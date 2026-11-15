# SPDX-License-Identifier: AGPL-3.0-or-later
"""Generate data/scenario/1925/deposits.ron from the curated deposit table.

Run from the repository root (needs Pillow to read the 16-bit province texture):

    python3 -I tools/deposits/generate.py            # write data/scenario/1925/deposits.ron
    python3 -I tools/deposits/generate.py --check    # fail if the RON on disk is out of date
    python3 -I tools/deposits/generate.py --report   # also print where each row landed and the totals
    python3 -I tools/deposits/generate.py --where LON LAT   # which state a point falls in

Every row of `tools/deposits/deposits_1925.csv` is one deposit, district or national aggregate
with a representative longitude and latitude. The point is projected with the map's Miller
projection (the same formula as `bf-client::world::project`) and looked up in
`assets/map/provinces.png` (16-bit grayscale province ids); the province gives the state through
`assets/map/provinces.ron`. A point that falls in the sea (offshore fields, coastal mines) takes the
nearest province within `SEARCH_RADIUS` pixels. The `state` column, when filled, overrides the
lookup (offshore fields whose nearest coast belongs to another country, disputed borders); the
script still reports the looked-up state so that an override that disagrees is visible.

Rows that land in the same state with the same resource are merged when both were known by 1925
(output and potential summed, size the largest, notes joined); a later discovery keeps its own
entry so that it can stay hidden until its year.
"""

from __future__ import annotations

import argparse
import csv
import math
import re
import sys
from collections import defaultdict
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
CSV_PATH = ROOT / "tools/deposits/deposits_1925.csv"
MAP_RON = ROOT / "assets/map/provinces.ron"
MAP_PNG = ROOT / "assets/map/provinces.png"
OUT = ROOT / "data/scenario/1925/deposits.ron"
SCENARIO_START = 1925
SEARCH_RADIUS = 60  # pixels, about 150 km at the equator

# Order and units must match `bf_sim::resources::RESOURCES`.
UNITS = [
    ("coal", "Mt raw coal (hard coal and lignite)"),
    ("iron_ore", "Mt gross ore"),
    ("copper", "kt Cu content"),
    ("bauxite", "kt gross ore"),
    ("oil", "Mt crude"),
    ("natural_gas", "bcm marketed gas"),
    ("gold", "t Au"),
    ("lead_zinc", "kt Pb + Zn content"),
    ("tin", "kt Sn content"),
    ("nickel", "kt Ni content"),
    ("manganese", "kt gross ore"),
    ("chromium", "kt chromite gross"),
    ("tungsten", "t WO3 content"),
    ("sulfur", "kt S content (native, pyrite and recovered)"),
    ("phosphate", "kt phosphate rock gross"),
    ("potash", "kt K2O"),
    ("uranium", "t U content"),
    ("rare_earths", "kt REO"),
]
RESOURCE_ORDER = {key: i for i, (key, _) in enumerate(UNITS)}

# 1925 world production used as the yardstick in DEPOSITS.md (same units as UNITS).
REFERENCE_1925 = {
    "coal": 1370.0,
    "iron_ore": 165.0,
    "copper": 1420.0,
    "bauxite": 1380.0,
    "oil": 146.0,
    "natural_gas": 35.0,
    "gold": 590.0,
    "lead_zinc": 2800.0,
    "tin": 145.0,
    "nickel": 37.0,
    "manganese": 2800.0,
    "chromium": 300.0,
    "tungsten": 6500.0,
    "sulfur": 4400.0,
    "phosphate": 9500.0,
    "potash": 1600.0,
    "uranium": 0.0,
    "rare_earths": 0.0,
}


def load_map():
    text = MAP_RON.read_text(encoding="utf-8")
    meta = {
        "width": int(re.search(r"width:\s*(\d+)", text).group(1)),
        "height": int(re.search(r"height:\s*(\d+)", text).group(1)),
        "projection": re.search(r'projection:\s*"([^"]+)"', text).group(1),
        "lat_top": float(re.search(r"lat_top:\s*(-?[\d.]+)", text).group(1)),
        "lat_bottom": float(re.search(r"lat_bottom:\s*(-?[\d.]+)", text).group(1)),
    }
    states_at = text.index("    states: [")
    province_state = {
        int(m.group(1)): int(m.group(2))
        for m in re.finditer(r"\(id: (\d+), key: \"[^\"]*\".*?state: (\d+)\)", text[:states_at])
    }
    states = {}
    for m in re.finditer(
        r"\(id: (\d+), key: \"([^\"]+)\", name: \"((?:[^\"\\]|\\.)*)\", owner: \"([^\"]+)\"",
        text[states_at:],
    ):
        states[int(m.group(1))] = (m.group(2), m.group(3), m.group(4))
    return meta, province_state, states


def project(meta, lon, lat):
    """lon/lat -> texture pixel; same formula as bf-client::world::project."""

    def y_of(lat_deg):
        phi = math.radians(lat_deg)
        if meta["projection"] == "miller":
            return 1.25 * math.log(math.tan(math.pi / 4 + 0.4 * phi))
        return phi

    top, bottom = y_of(meta["lat_top"]), y_of(meta["lat_bottom"])
    x = (lon + 180.0) / 360.0 * meta["width"]
    y = (top - y_of(lat)) / (top - bottom) * meta["height"]
    return x, y


class Locator:
    def __init__(self):
        self.meta, self.province_state, self.states = load_map()
        self.image = Image.open(MAP_PNG)
        self.pixels = self.image.load()
        self.key_of = {sid: s[0] for sid, s in self.states.items()}

    def province_at(self, x, y):
        w, h = self.meta["width"], self.meta["height"]
        if not 0 <= y < h:
            return 0
        return self.pixels[int(x) % w, int(y)]

    def locate(self, lon, lat):
        """State key and distance in pixels to the nearest province pixel."""
        x, y = project(self.meta, lon, lat)
        best = None
        for r in range(0, SEARCH_RADIUS + 1):
            for dx in range(-r, r + 1):
                for dy in (-r, r) if abs(dx) != r else range(-r, r + 1):
                    pid = self.province_at(x + dx, y + dy)
                    if pid:
                        d = math.hypot(dx, dy)
                        if best is None or d < best[0]:
                            best = (d, pid)
            if best is not None and best[0] <= r:
                break
        if best is None:
            return None, None
        state_id = self.province_state.get(best[1])
        return self.key_of.get(state_id), best[0]


def read_rows():
    with CSV_PATH.open(newline="", encoding="utf-8") as f:
        rows = [r for r in csv.DictReader(f) if r["resource"] and not r["resource"].startswith("#")]
    for r in rows:
        r["lon"], r["lat"] = float(r["lon"]), float(r["lat"])
        r["size"] = int(r["size"])
        r["discovered"] = int(r["discovered"])
        r["output_1925"] = float(r["output_1925"])
        r["potential"] = float(r["potential"])
    return rows


def validate(rows):
    errors = []
    for n, r in enumerate(rows, start=2):
        where = f"line {n} ({r['deposit']})"
        if r["resource"] not in RESOURCE_ORDER:
            errors.append(f"{where}: unknown resource {r['resource']!r}")
        if not 1 <= r["size"] <= 5:
            errors.append(f"{where}: size {r['size']} outside 1..=5")
        if r["discovered"] > SCENARIO_START and r["output_1925"] != 0:
            errors.append(f"{where}: discovered {r['discovered']} but output_1925 > 0")
        if r["potential"] < r["output_1925"]:
            errors.append(f"{where}: potential below output_1925")
        if not r["source"]:
            errors.append(f"{where}: no source tag")
    return errors


def fmt(value):
    text = f"{value:.3f}".rstrip("0")
    return text + "0" if text.endswith(".") else text


def build(rows, locator, report):
    states = {s[0] for s in locator.states.values()}
    merged = defaultdict(list)  # state -> list of deposit dicts
    problems = []
    for r in rows:
        found, dist = locator.locate(r["lon"], r["lat"])
        state = r["state"] or found
        if state is None:
            problems.append(f"{r['deposit']}: no province within {SEARCH_RADIUS} px; set `state`")
            continue
        if state not in states:
            problems.append(f"{r['deposit']}: unknown state {state!r}")
            continue
        if report:
            flag = ""
            if r["state"] and r["state"] != found:
                flag = f"  (override; lookup gave {found})"
            elif dist:
                flag = f"  (nearest land {dist:.0f} px)"
            print(f"{r['resource']:12} {state:28} {r['deposit']}{flag}")
        known = r["discovered"] <= SCENARIO_START
        for d in merged[state]:
            if d["resource"] == r["resource"] and known and d["discovered"] <= SCENARIO_START:
                d["size"] = max(d["size"], r["size"])
                d["discovered"] = min(d["discovered"], r["discovered"])
                d["output_1925"] += r["output_1925"]
                d["potential"] += r["potential"]
                d["notes"].append(f"{r['deposit']} ({r['source']})")
                break
            if d["resource"] == r["resource"] and d["discovered"] == r["discovered"] and not known:
                d["size"] = max(d["size"], r["size"])
                d["potential"] += r["potential"]
                d["notes"].append(f"{r['deposit']} ({r['source']})")
                break
        else:
            merged[state].append(
                {
                    "resource": r["resource"],
                    "size": r["size"],
                    "discovered": r["discovered"],
                    "output_1925": r["output_1925"],
                    "potential": r["potential"],
                    "notes": [f"{r['deposit']} ({r['source']})"],
                }
            )
    return merged, problems


def render(merged):
    out = [
        "// Generated by `python3 -I tools/deposits/generate.py` from tools/deposits/deposits_1925.csv;",
        "// do not edit by hand. Sources and method: data/scenario/1925/DEPOSITS.md.",
        "// Format: bf_sim::resources::DepositsFile. Data licence: CC BY-SA 4.0.",
        "(",
        "    units: [",
    ]
    for key, unit in UNITS:
        out.append(f'        ("{key}", "{unit}"),')
    out.append("    ],")
    out.append("    states: [")
    for state in sorted(merged):
        out.append("        (")
        out.append(f'            state: "{state}",')
        out.append("            deposits: [")
        deposits = sorted(
            merged[state], key=lambda d: (RESOURCE_ORDER[d["resource"]], d["discovered"])
        )
        for d in deposits:
            note = "; ".join(d["notes"]).replace("\\", "\\\\").replace('"', '\\"')
            out.append(
                f'                (resource: "{d["resource"]}", size: {d["size"]}, '
                f'discovered: {d["discovered"]}, output_1925: {fmt(d["output_1925"])}, '
                f'potential: {fmt(d["potential"])}, note: "{note}"),'
            )
        out.append("            ],")
        out.append("        ),")
    out.append("    ],")
    out.append(")")
    return "\n".join(out) + "\n"


def print_totals(merged):
    count = defaultdict(int)
    hidden = defaultdict(int)
    total = defaultdict(float)
    by_state = defaultdict(lambda: defaultdict(float))
    for state, deposits in merged.items():
        for d in deposits:
            count[d["resource"]] += 1
            hidden[d["resource"]] += d["discovered"] > SCENARIO_START
            total[d["resource"]] += d["output_1925"]
            by_state[d["resource"]][state] += d["output_1925"]
    print("\n| Recurso | Unidade | Depósitos | Ocultos | Produção 1925 | Referência |")
    print("|---|---|---:|---:|---:|---:|")
    for key, unit in UNITS:
        print(
            f"| {key} | {unit} | {count[key]} | {hidden[key]} | {total[key]:,.1f} | "
            f"{REFERENCE_1925[key]:,.0f} |"
        )
    print()
    for key, _ in UNITS:
        top = sorted(by_state[key].items(), key=lambda kv: -kv[1])[:5]
        print(f"{key}: " + ", ".join(f"{s} {v:,.1f}" for s, v in top if v))


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--check", action="store_true", help="fail if the RON is out of date")
    parser.add_argument("--report", action="store_true", help="print placements and totals")
    parser.add_argument("--where", nargs=2, type=float, metavar=("LON", "LAT"))
    args = parser.parse_args()

    locator = Locator()
    if args.where:
        state, dist = locator.locate(*args.where)
        print(state, f"{dist:.0f} px" if dist is not None else "")
        return 0

    rows = read_rows()
    errors = validate(rows)
    merged, problems = build(rows, locator, args.report)
    errors += problems
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    text = render(merged)
    if args.report:
        print_totals(merged)
    if args.check:
        if OUT.read_text(encoding="utf-8") != text:
            print(f"{OUT.relative_to(ROOT)} is out of date; run the generator", file=sys.stderr)
            return 1
        return 0
    OUT.write_text(text, encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)} ({sum(len(v) for v in merged.values())} deposits)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
