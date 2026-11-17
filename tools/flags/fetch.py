# SPDX-License-Identifier: AGPL-3.0-or-later
"""Re-download and re-render the 1925 country flags from Wikimedia Commons.

Run from the repository root:

    python3 -I tools/flags/fetch.py            # render assets/flags/<TAG>.png from the pinned revisions
    python3 -I tools/flags/fetch.py --pin      # record the current revision of every file (sha1 column)

Every row of `tools/flags/flags_1925.csv` (the scenario) and `tools/flags/flags_new.csv` (the
countries that may be born later) names a Commons file and the SHA-1 of the revision that
was reviewed. The script looks that exact revision up in the file history, so a later edit on
Commons never changes the output silently: a missing revision is an error until someone reviews
the new one and pins it again. Licences are checked on every run; only public domain, CC0, CC BY
and CC BY-SA (2.0 or later) pass, since the art ships under CC BY-SA 4.0.

The SVG is rendered by librsvg through ImageMagick (`magick`), stretched to 192x128 (3:2) the way
a strategy-game flag is shown, except rows marked `fit` (non-rectangular flags), which keep their
proportions on a transparent canvas. If `oxipng` is on the PATH (or passed with `--oxipng`), the
PNGs are recompressed losslessly. `assets/flags/ATTRIBUTION.md` is rewritten from the Commons
metadata on every run.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import html
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.parse
import urllib.request
from pathlib import Path

API = "https://commons.wikimedia.org/w/api.php"
USER_AGENT = (
    "between-frontiers-flag-fetch/0.1 (open-source game; https://github.com/userNanni/between-frontiers)"
)
WIDTH, HEIGHT = 192, 128
ROOT = Path(__file__).resolve().parents[2]
LISTS = ROOT / "tools" / "flags"
OUT_DIR = ROOT / "assets" / "flags"
FIELDS = ["tag", "commons_file", "sha1", "render", "description", "note"]

# Licences whose terms let the file ship inside CC BY-SA 4.0 game art.
LICENCE_OK = re.compile(
    r"^(public domain|pd\b.*|cc0( 1\.0)?|cc by(-sa)? (2\.0|2\.5|3\.0|4\.0)( [a-z-]+)?|cc by 1\.0)$",
    re.IGNORECASE,
)


def http_get(url: str, *, retries: int = 4) -> bytes:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    for attempt in range(retries):
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                return response.read()
        except Exception:  # network hiccups and 429s: back off and retry
            if attempt == retries - 1:
                raise
            time.sleep(2 ** (attempt + 1))
    raise AssertionError("unreachable")


def api(**params: str) -> dict:
    params = {**params, "format": "json", "formatversion": "2"}
    return json.loads(http_get(API + "?" + urllib.parse.urlencode(params)))


def plain(text: str) -> str:
    """Strip the HTML Commons puts in extmetadata."""
    text = re.sub(r"<[^>]+>", " ", text or "")
    return re.sub(r"\s+", " ", html.unescape(text)).strip()


def file_history(title: str) -> tuple[list[dict], dict]:
    """All revisions of a file (newest first) and the extmetadata of the current one."""
    data = api(
        action="query",
        prop="imageinfo",
        titles="File:" + title,
        iiprop="url|sha1|timestamp|extmetadata",
        iilimit="50",
    )
    page = data["query"]["pages"][0]
    if page.get("missing") or "imageinfo" not in page:
        raise SystemExit(f"Commons file not found: {title}")
    revisions = page["imageinfo"]
    return revisions, revisions[0].get("extmetadata", {})


def licence_of(meta: dict) -> tuple[str, str]:
    name = plain(meta.get("LicenseShortName", {}).get("value", ""))
    url = plain(meta.get("LicenseUrl", {}).get("value", ""))
    if not url and name.lower().startswith(("public domain", "pd")):
        url = "https://creativecommons.org/publicdomain/mark/1.0/"
    return name, url


def author_of(meta: dict) -> str:
    author = plain(meta.get("Artist", {}).get("value", ""))
    return author or "Not stated (see the file page)"


LENGTH = re.compile(r"^\s*([0-9.]+(?:[eE][+-]?[0-9]+)?)\s*(px|pt|pc|mm|cm|in)?\s*$")
UNIT_PX = {None: 1.0, "px": 1.0, "pt": 4 / 3, "pc": 16.0, "mm": 96 / 25.4, "cm": 96 / 2.54, "in": 96.0}


def to_px(value: str) -> float:
    match = LENGTH.match(value)
    if not match:
        raise ValueError(f"unsupported SVG length {value!r}")
    return float(match.group(1)) * UNIT_PX[match.group(2)]


def set_attr(tag: str, name: str, value: str) -> str:
    pattern = re.compile(r"(\s)" + re.escape(name) + r"\s*=\s*(\"[^\"]*\"|'[^']*')")
    if pattern.search(tag):
        return pattern.sub(lambda m: f'{m.group(1)}{name}="{value}"', tag, count=1)
    return tag[:4] + f' {name}="{value}"' + tag[4:]


def get_attr(tag: str, name: str) -> str | None:
    match = re.search(r"\s" + re.escape(name) + r"\s*=\s*(\"([^\"]*)\"|'([^']*)')", tag)
    if not match:
        return None
    return match.group(2) if match.group(2) is not None else match.group(3)


def frame_svg(svg: str, mode: str) -> str:
    """Give the root element a 192x128 viewport; stretch the drawing into it or fit it inside."""
    match = re.search(r"<svg\b[^>]*>", svg)
    if not match:
        raise ValueError("no <svg> root element")
    tag = match.group(0)
    if get_attr(tag, "viewBox") is None:
        width, height = get_attr(tag, "width"), get_attr(tag, "height")
        if not width or not height:
            raise ValueError("SVG without viewBox nor width/height")
        tag = set_attr(tag, "viewBox", f"0 0 {to_px(width):g} {to_px(height):g}")
    tag = set_attr(tag, "width", str(WIDTH))
    tag = set_attr(tag, "height", str(HEIGHT))
    tag = set_attr(tag, "preserveAspectRatio", "none" if mode == "stretch" else "xMidYMid meet")
    return svg[: match.start()] + tag + svg[match.end() :]


def render(svg_path: Path, png_path: Path, oxipng: str | None) -> None:
    subprocess.run(
        [
            "magick",
            "-background", "none",
            "-density", "96",
            f"rsvg:{svg_path}",
            "-resize", f"{WIDTH}x{HEIGHT}!",
            "-colorspace", "sRGB",
            "-strip",
            "-define", "png:exclude-chunks=date,time",
            "-define", "png:compression-level=9",
            f"PNG32:{png_path}",
        ],
        check=True,
    )
    if oxipng:
        subprocess.run([oxipng, "-q", "-o", "max", "--strip", "all", str(png_path)], check=True)


def list_paths() -> list[Path]:
    """Every list: `flags_1925.csv` (the scenario) first, then the others (`flags_new.csv`, the
    countries born after 1925, data/borders/candidates.ron) in name order."""
    paths = sorted(LISTS.glob("flags_*.csv"))
    return sorted(paths, key=lambda p: p.name != "flags_1925.csv")


def read_rows() -> list[dict]:
    rows = []
    for path in list_paths():
        with path.open(newline="", encoding="utf-8") as handle:
            rows += [{**row, "_list": path} for row in csv.DictReader(handle)]
    return rows


def write_rows(rows: list[dict]) -> None:
    for path in list_paths():
        with path.open("w", newline="", encoding="utf-8") as handle:
            writer = csv.DictWriter(handle, fieldnames=FIELDS, lineterminator="\n", extrasaction="ignore")
            writer.writeheader()
            writer.writerows(r for r in rows if r["_list"] == path)


def md_cell(text: str) -> str:
    return text.replace("|", "\\|").replace("\n", " ")


def write_attribution(records: list[dict]) -> None:
    lines = [
        "# Flag attribution",
        "",
        "Flags on 1 January 1925 for the countries of the 1925 scenario, and the flags at birth of the",
        "countries that may be born later (`data/borders/candidates.ron`), rendered to 192x128 PNG",
        "from SVG files on Wikimedia Commons by `tools/flags/fetch.py` (lists in",
        "`tools/flags/flags_1925.csv` and `tools/flags/flags_new.csv`, which pin the reviewed",
        "revision of each file by SHA-1).",
        "Most designs are in the public domain; the drawings below under CC BY or CC BY-SA are used",
        "under those licences, and the rendered PNGs ship under CC BY-SA 4.0 with the rest of the",
        "game art. Rows marked `fit` keep their own proportions; all others are stretched to 3:2.",
        "",
        "| Tag | Flag | Commons file | Author | Licence |",
        "|---|---|---|---|---|",
    ]
    for r in records:
        page = "https://commons.wikimedia.org/wiki/File:" + urllib.parse.quote(r["commons_file"].replace(" ", "_"))
        licence = f"[{md_cell(r['licence'])}]({r['licence_url']})" if r["licence_url"] else md_cell(r["licence"])
        flag = md_cell(r["description"]) + (f" ({md_cell(r['note'])})" if r["note"] else "")
        lines.append(
            f"| {r['tag']} | {flag} | [{md_cell(r['commons_file'])}]({page}) | {md_cell(r['author'])} | {licence} |"
        )
    lines.append("")
    (OUT_DIR / "ATTRIBUTION.md").write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--pin", action="store_true", help="record the current revision SHA-1 of every file")
    parser.add_argument("--only", nargs="*", help="render only these tags (the attribution still covers every list)")
    parser.add_argument("--oxipng", default=shutil.which("oxipng"), help="path to oxipng (optional)")
    parser.add_argument("--keep-svg", type=Path, help="also keep the downloaded SVGs in this directory")
    args = parser.parse_args()
    if shutil.which("magick") is None:
        raise SystemExit("ImageMagick 7 (`magick`, with the librsvg delegate) is required")

    rows = read_rows()
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    records, rejected = [], []
    with tempfile.TemporaryDirectory(prefix="bf-flags-") as scratch:
        for row in rows:
            revisions, meta = file_history(row["commons_file"])
            licence, licence_url = licence_of(meta)
            if not LICENCE_OK.match(licence):
                rejected.append(f"{row['tag']}: {row['commons_file']} ({licence or 'no licence'})")
                continue
            if args.pin:
                row["sha1"] = revisions[0]["sha1"]
            wanted = [rev for rev in revisions if rev.get("sha1") == row["sha1"]]
            if not wanted:
                raise SystemExit(
                    f"{row['tag']}: pinned revision {row['sha1'] or '(none)'} of {row['commons_file']} "
                    "is not in the file history; review the current file and run with --pin"
                )
            records.append({**row, "licence": licence, "licence_url": licence_url, "author": author_of(meta)})
            if args.only and row["tag"] not in args.only:
                continue
            data = http_get(wanted[0]["url"])
            if hashlib.sha1(data).hexdigest() != row["sha1"]:
                raise SystemExit(f"{row['tag']}: downloaded bytes do not match the pinned SHA-1")
            if args.keep_svg:
                args.keep_svg.mkdir(parents=True, exist_ok=True)
                (args.keep_svg / f"{row['tag']}.svg").write_bytes(data)
            svg_path = Path(scratch) / f"{row['tag']}.svg"
            svg_path.write_text(frame_svg(data.decode("utf-8-sig"), row["render"]), encoding="utf-8")
            render(svg_path, OUT_DIR / f"{row['tag']}.png", args.oxipng)
            print(f"{row['tag']}: {row['commons_file']} [{licence}]", file=sys.stderr)
            time.sleep(0.2)  # be gentle with the API

    if rejected:
        print("Rejected for licence:\n  " + "\n  ".join(rejected), file=sys.stderr)
        return 1
    if args.pin:
        write_rows(rows)
    write_attribution(records)
    return 0


if __name__ == "__main__":
    sys.exit(main())
