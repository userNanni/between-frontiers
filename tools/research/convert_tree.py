"""Converts the costs of the technology tree from one unit of research effort to another.

The costs in data/research/technologies.ron are written in the research points of a reference
run. When the rule that makes research points changes (head count before, laboratories that cost
after, docs/research.pt-BR.md, "Pesquisa que custa"), the tree has to be rewritten in the new
unit, or every date moves. This script does that from two 100-year runs with the same tree:

    factor = sum of the leader's points over the technology's own window in the new run
           / the same in the reference run

where the leader of a year is the country with the most points that year, and the window of a
technology runs from the historical year of its latest requirement to its own historical year,
held to 5-15 years: a technology is paid with the research of the years before it, not with
the stock since 1925. Each technology's cost becomes cost x its factor; technologies of 1925
keep theirs. When the factor falls (the new unit's effort slows down against the old one), a
technology could end up cheaper than one of its requirements; the tree's rule that every
technology costs at least 10 % more than each requirement is then restored by raising it, in
the order of the requirements, and the script says how many it raised. Run it again when
research changes again (the federal R&D of the Cold War, phase 5).

    python3 tools/research/convert_tree.py REFERENCE_HISTORY.json NEW_HISTORY.json [--write]

The histories are `bf-bench --game --history-out` files of the same seed, both with the tree
as it stands. Without `--write` it only prints the factors and the new cost ranges by era.
"""

import json
import os
import re
import sys

ROOT = os.path.normpath(os.path.join(os.path.dirname(__file__), "..", ".."))
TREE = os.path.join(ROOT, "data", "research", "technologies.ron")


def leader_points(path):
    """Year -> research points of that year's leader."""
    with open(path, encoding="utf-8") as f:
        h = json.load(f)
    years = h["years"]
    out = {}
    for i, y in enumerate(years):
        best = 0.0
        for c in h["countries"]:
            pts = c.get("research_points") or []
            if i < len(pts):
                best = max(best, pts[i])
        out[y] = best
    return out


WINDOW = (5, 15)


def window_factor(reference, new, year, years):
    """Leader's points in the `years` before `year` (inclusive), new over reference. Years past
    the end of the runs use the last window the runs have."""
    last = max(set(reference) & set(new))
    end = min(year, last)
    span = range(end - years + 1, end + 1)
    ref = sum(reference.get(y, 0.0) for y in span)
    nw = sum(new.get(y, 0.0) for y in span)
    return nw / ref if ref > 0 and nw > 0 else 1.0


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if len(args) != 2:
        sys.exit(__doc__)
    reference, new_points = leader_points(args[0]), leader_points(args[1])
    for y in range(1935, 2026, 10):
        print(f"{y}: factor over 10 years {window_factor(reference, new_points, y, 10):.3f}")
    with open(TREE, encoding="utf-8") as fh:
        text = fh.read()
    block = re.compile(r"\(\s*key:\s*\"(?P<key>[^\"]+)\".*?\n\s*\),", re.S)
    matches = list(block.finditer(text))
    techs = {}
    for m in matches:
        body = m.group(0)
        era = int(re.search(r"era:\s*(\d+)", body).group(1))
        year = int(re.search(r"demonstrated:\s*(-?\d+)", body).group(1))
        cost = int(re.search(r"cost:\s*(\d+)", body).group(1))
        req = re.search(r"requires:\s*\[([^\]]*)\]", body)
        requires = re.findall(r"\"([^\"]+)\"", req.group(1)) if req else []
        techs[m.group("key")] = {"era": era, "cost": cost, "year": year, "requires": requires}
    windows = []
    for t in techs.values():
        if t["era"] == 0:
            continue
        latest = max((techs[r]["year"] for r in t["requires"]), default=t["year"] - 10)
        years = min(max(t["year"] - latest, WINDOW[0]), WINDOW[1])
        windows.append(years)
        factor = window_factor(reference, new_points, t["year"], years)
        t["cost"] = max(1, round(t["cost"] * factor))
    print(f"windows: {min(windows)}-{max(windows)} years, mean {sum(windows) / len(windows):.1f}")
    # Every technology at least 10 % above each requirement, requirements first.
    raised, done = 0, set()

    def settle(k):
        nonlocal raised
        if k in done:
            return
        t = techs[k]
        for r in t["requires"]:
            settle(r)
        floor = max((-(-techs[r]["cost"] * 11 // 10) for r in t["requires"]), default=0)
        if t["era"] > 0 and t["cost"] < floor:
            t["cost"] = floor
            raised += 1
        done.add(k)

    for k in techs:
        settle(k)
    print(f"raised to 10 % above a requirement: {raised} of {len(techs)}")
    ranges = {}
    pieces, last = [], 0
    for m in matches:
        body = m.group(0)
        t = techs[m.group("key")]
        cost_m = re.search(r"cost:\s*(\d+)", body)
        body = body[: cost_m.start(1)] + str(t["cost"]) + body[cost_m.end(1):]
        lo, hi = ranges.get(t["era"], (t["cost"], t["cost"]))
        ranges[t["era"]] = (min(lo, t["cost"]), max(hi, t["cost"]))
        pieces.append(text[last : m.start()])
        pieces.append(body)
        last = m.end()
    pieces.append(text[last:])
    for era in sorted(ranges):
        print(f"era {era}: costs {ranges[era][0]}-{ranges[era][1]}")
    if "--write" in sys.argv:
        with open(TREE, "w", encoding="utf-8") as fh:
            fh.write("".join(pieces))
        print("wrote", os.path.relpath(TREE, ROOT))


if __name__ == "__main__":
    main()
