"""Converts the costs of the technology tree from one unit of research effort to another.

The costs in data/research/technologies.ron are written in the research points of a reference
run. When the rule that makes research points changes (head count before, laboratories that cost
after, docs/research.pt-BR.md, "Pesquisa que custa"), the tree has to be rewritten in the new
unit, or every date moves. This script does that from two 100-year runs with the same tree:

    factor(Y) = sum of the leader's points from 1925 to Y in the new run
              / the same in the reference run

where the leader of a year is the country with the most points that year. Each technology's
cost becomes cost x factor(its historical breakthrough year); technologies of 1925 keep theirs.
The factor comes from cumulative sums, so it moves smoothly from year to year, without steps by
era. When the factor falls (the new unit's effort slows down against the old one), a
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


def factors(reference, new):
    years = sorted(set(reference) & set(new))
    out, cum_ref, cum_new = {}, 0.0, 0.0
    for y in years:
        cum_ref += reference[y]
        cum_new += new[y]
        if cum_ref > 0 and cum_new > 0:
            out[y] = cum_new / cum_ref
    return out


def factor_at(f, year):
    if year in f:
        return f[year]
    known = sorted(f)
    if year < known[0]:
        return f[known[0]]
    return f[known[-1]]


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if len(args) != 2:
        sys.exit(__doc__)
    f = factors(leader_points(args[0]), leader_points(args[1]))
    for y in range(1930, 2026, 5):
        if y in f:
            print(f"{y}: factor {f[y]:.3f}")
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
        new = max(1, round(cost * factor_at(f, year))) if era > 0 else cost
        techs[m.group("key")] = {"era": era, "cost": new, "requires": requires}
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
