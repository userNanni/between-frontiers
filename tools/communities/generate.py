"""Builds data/scenario/1925/communities.ron from the sourced tables in tools/communities/spec/.

Run from the repository root:

    python3 -I tools/communities/generate.py

Each `spec/<region>.txt` line is

    STATE [STATE ...] | culture/religion SHARE, culture/religion ~, ... | SOURCE | FLAG

with `~` standing for the rest (1 minus the other shares) and FLAG one of C (census of
1915–1935), M (census for one dimension, the other inferred, or a nearby census adjusted) or E
(estimate). Relevance lines are

    !TAG | culture/religion | factor, factor | note

with `!TAG:C` for the country's colonies (plain `!TAG` is the metropole). Lines for the same
country and community from several files are merged.

The script checks every key against data/culture/*.ron and the map, rounds the shares to four
places (the largest absorbs the rounding), and writes tools/communities/check.md: the world and
country totals by community, with the state population approximated as the economy does it
until data/scenario/1925/population.ron exists (country population split by cropland plus 2 % of
the area). Method and sources: docs/population.pt-BR.md.
"""

import glob
import os
import re
from collections import defaultdict

ROOT = os.getcwd()
SPEC = os.path.join(ROOT, "tools", "communities", "spec")
OUT = os.path.join(ROOT, "data", "scenario", "1925", "communities.ron")
CHECK = os.path.join(ROOT, "tools", "communities", "check.md")

FLAGS = {"C": "Census", "M": "Mixed", "E": "Estimate"}
FACTORS = {
    "elite": "Elite",
    "organization": "Organization",
    "legal_status": "LegalStatus",
    "patron": "Patron",
}


def fail(msg):
    raise SystemExit(f"generate.py: {msg}")


def keys_of(path, field="key"):
    text = open(os.path.join(ROOT, path), encoding="utf-8").read()
    return re.findall(rf'\b{field}: "([^"]+)"', text)


def map_states():
    text = open(os.path.join(ROOT, "assets", "map", "provinces.ron"), encoding="utf-8").read()
    text = text[text.index("states:"):]
    out = {}
    for m in re.finditer(
        r'\(id: \d+, key: "([^"]+)", name: "(?:[^"\\]|\\.)*", owner: "(\w+)"'
        r'(?:, colony: (true|false))?, center: \([^)]*\), area_km2: ([\d.e+-]+)',
        text,
    ):
        out[m.group(1)] = (m.group(2), m.group(3) == "true", float(m.group(4)))
    return out


def countries():
    text = open(os.path.join(ROOT, "data", "scenario", "1925", "countries.ron"), encoding="utf-8").read()
    out = {}
    for block in re.split(r'\n        \(\n            tag: ', text)[1:]:
        tag = block[1:4]
        pop = int(re.search(r"\n            population: (\d+)", block).group(1))
        col = re.search(r"colonies: Some\(\(\s*population: (\d+)", block)
        out[tag] = (pop, int(col.group(1)) if col else 0)
    return out


def cropland():
    text = open(os.path.join(ROOT, "data", "scenario", "1925", "land.ron"), encoding="utf-8").read()
    return {
        m.group(1): float(m.group(2))
        for m in re.finditer(r'state: "([^"]+)",\n\s*climate: [^\n]*\n\s*cropland_km2: ([\d.]+)', text)
    }


def populations(states):
    """State population as the economy approximates it (econ::setup, no census file)."""
    pops, crop = countries(), cropland()
    weight = {k: crop.get(k, 0.0) + area * 0.02 + 1.0 for k, (_, _, area) in states.items()}
    groups = defaultdict(list)
    for k, (owner, colony, _) in states.items():
        groups[(owner, colony)].append(k)
    out = {}
    for (owner, colony), ks in groups.items():
        total = pops.get(owner, (0, 0))[1 if colony else 0]
        w = sum(weight[k] for k in ks)
        for k in ks:
            out[k] = total * weight[k] / w
    return out


def parse_pair(text, where):
    if "/" not in text:
        fail(f"{where}: '{text}' is not culture/religion")
    c, r = (x.strip() for x in text.split("/", 1))
    return c, r


def parse_spec():
    compositions, relevance = {}, []
    files = sorted(glob.glob(os.path.join(SPEC, "*.txt")))
    if not files:
        fail(f"no spec in {SPEC}")
    for path in files:
        name = os.path.basename(path)
        for n, raw in enumerate(open(path, encoding="utf-8"), 1):
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            where = f"{name}:{n}"
            parts = [p.strip() for p in line.split("|")]
            if line.startswith("!"):
                if len(parts) != 4:
                    fail(f"{where}: relevance needs 4 fields")
                c, r = parse_pair(parts[1], where)
                factors = [f.strip() for f in parts[2].split(",") if f.strip()]
                for f in factors:
                    if f not in FACTORS:
                        fail(f"{where}: unknown factor {f}")
                tag, _, colonies = parts[0][1:].strip().partition(":")
                if colonies not in ("", "C"):
                    fail(f"{where}: scope {parts[0]}")
                relevance.append((tag, colonies == "C", c, r, factors, parts[3], where))
                continue
            if len(parts) != 4:
                fail(f"{where}: composition needs 4 fields")
            scope, comp, source, flag = parts
            if flag not in FLAGS:
                fail(f"{where}: flag {flag}")
            shares, rest = [], None
            for entry in (e.strip() for e in comp.split(",")):
                pair, _, value = entry.rpartition(" ")
                c, r = parse_pair(pair, where)
                if value == "~":
                    if rest is not None:
                        fail(f"{where}: two '~'")
                    rest = (c, r)
                    continue
                shares.append([c, r, float(value)])
            explicit = sum(s[2] for s in shares)
            if rest is not None:
                if explicit >= 1.0:
                    fail(f"{where}: shares sum to {explicit:.4f} before '~'")
                shares.append([rest[0], rest[1], 1.0 - explicit])
            elif abs(explicit - 1.0) > 0.005:
                fail(f"{where}: shares sum to {explicit:.4f}")
            seen = set()
            for c, r, _ in shares:
                if (c, r) in seen:
                    fail(f"{where}: {c}/{r} twice")
                seen.add((c, r))
            for state in scope.split():
                if state in compositions:
                    fail(f"{where}: {state} already given in {compositions[state][3]}")
                compositions[state] = ([list(s) for s in shares], FLAGS[flag], source, where)
    return compositions, relevance


def normalise(shares):
    total = sum(s[2] for s in shares)
    for s in shares:
        s[2] = round(s[2] / total, 4)
    shares.sort(key=lambda s: (-s[2], s[0], s[1]))
    shares[0][2] = round(shares[0][2] + 1.0 - sum(s[2] for s in shares), 4)
    return [s for s in shares if s[2] > 0]


def ron_str(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def main():
    cultures = set(keys_of("data/culture/cultures.ron"))
    religions = set(keys_of("data/culture/religions.ron"))
    states = map_states()
    if len(states) != 870:
        fail(f"read {len(states)} states from the map")
    compositions, relevance = parse_spec()

    missing = sorted(set(states) - set(compositions))
    extra = sorted(set(compositions) - set(states))
    if missing or extra:
        fail(f"states without composition: {missing}; not on the map: {extra}")
    communities = set()
    for state, (shares, _, _, where) in compositions.items():
        for c, r, _ in shares:
            if c not in cultures:
                fail(f"{where}: culture {c} not in data/culture/cultures.ron")
            if r not in religions:
                fail(f"{where}: religion {r} not in data/culture/religions.ron")
            communities.add((c, r))
    owners = {o for o, _, _ in states.values()}
    lives = defaultdict(set)
    for state, (shares, _, _, _) in compositions.items():
        owner, colony, _ = states[state]
        for c, r, _ in shares:
            lives[(owner, colony)].add((c, r))
    for tag, colonies, c, r, _, _, where in relevance:
        if tag not in owners:
            fail(f"{where}: country {tag} owns no state")
        if (c, r) not in lives[(tag, colonies)]:
            fail(f"{where}: {c}/{r} lives in no state of {tag}{' colonies' if colonies else ''}")
    # The same colonial power appears in several regions: merge its lines.
    merged = {}
    for tag, colonies, c, r, factors, note, where in relevance:
        key = (tag, colonies, c, r)
        if key in merged:
            old = merged[key]
            merged[key] = (old[0] + [f for f in factors if f not in old[0]], old[1] + "; " + note)
        else:
            merged[key] = (list(factors), note)
    relevance = [(*k, f, n, None) for k, (f, n) in merged.items()]

    out = [
        "// Generated by tools/communities/generate.py from tools/communities/spec/*.txt; do not edit",
        "// by hand. Sources and method: docs/population.pt-BR.md. Format:",
        "// bf_sim::communities::CommunitiesFile. Licence: CC BY-SA 4.0.",
        "(",
        "    communities: [",
    ]
    for c, r in sorted(communities):
        out.append(f'        ("{c}", "{r}"),')
    out += ["    ],", "    relevance: ["]
    for tag, colonies, c, r, factors, note, _ in sorted(relevance):
        fs = ", ".join(FACTORS[f] for f in factors)
        col = "colonies: true, " if colonies else ""
        out.append(
            f'        (country: "{tag}", {col}culture: "{c}", religion: "{r}", factors: [{fs}], '
            f"note: {ron_str(note)}),"
        )
    out += ["    ],", "    states: ["]
    final = {}
    for state in sorted(compositions):
        shares, basis, source, _ = compositions[state]
        shares = normalise(shares)
        final[state] = shares
        body = ", ".join(f'("{c}", "{r}", {v})' for c, r, v in shares)
        out.append(
            f'        (state: "{state}", shares: [{body}], basis: {basis}, source: {ron_str(source)}),'
        )
    out += ["    ],", ")", ""]
    with open(OUT, "w", encoding="utf-8") as f:
        f.write("\n".join(out))

    # Report: world and country totals with the approximate state populations.
    pops = populations(states)
    world = defaultdict(float)
    by_country = defaultdict(lambda: defaultdict(float))
    basis_pop = defaultdict(float)
    for state, shares in final.items():
        owner, colony, _ = states[state]
        p = pops[state]
        basis_pop[compositions[state][1]] += p
        for c, r, v in shares:
            world[(c, r)] += p * v
            by_country[(owner, colony)][(c, r)] += p * v
    total = sum(pops.values())
    lines = [
        "# Comunidades de 1925: conferência",
        "",
        "Gerado por `tools/communities/generate.py`. População do estado aproximada como faz a",
        "economia sem `population.ron` (população do país repartida pela lavoura mais 2 % da área).",
        "",
        f"Estados: {len(final)}. Comunidades: {len(communities)}. Culturas usadas: "
        f"{len({c for c, _ in communities})} de {len(cultures)}. Religiões usadas: "
        f"{len({r for _, r in communities})} de {len(religions)}. População: {total / 1e6:,.0f} milhões.",
        "",
        "## Base dos números, por população",
        "",
        "| Base | Milhões | % |",
        "|---|---:|---:|",
    ]
    for b in ("Census", "Mixed", "Estimate"):
        lines.append(f"| {b} | {basis_pop[b] / 1e6:,.0f} | {100 * basis_pop[b] / total:.1f} |")
    lines += ["", "## As 60 maiores comunidades do mundo", "", "| Comunidade | Milhões |", "|---|---:|"]
    for (c, r), p in sorted(world.items(), key=lambda x: -x[1])[:60]:
        lines.append(f"| {c} / {r} | {p / 1e6:,.1f} |")
    by_rel = defaultdict(float)
    for (_, r), p in world.items():
        by_rel[r] += p
    lines += ["", "## Religiões no mundo", "", "| Religião | Milhões | % |", "|---|---:|---:|"]
    for r, p in sorted(by_rel.items(), key=lambda x: -x[1]):
        lines.append(f"| {r} | {p / 1e6:,.1f} | {100 * p / total:.1f} |")
    lines += ["", "## Países: as três maiores comunidades", "", "| País | Comunidades (%) |", "|---|---|"]
    for (owner, colony), m in sorted(by_country.items()):
        t = sum(m.values()) or 1.0
        top = sorted(m.items(), key=lambda x: -x[1])[:3]
        label = f"{owner} (colônias)" if colony else owner
        lines.append(f"| {label} | " + "; ".join(f"{c}/{r} {100 * p / t:.1f}" for (c, r), p in top) + " |")
    with open(CHECK, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print(f"{len(final)} states, {len(communities)} communities, {len(relevance)} relevance lines")


if __name__ == "__main__":
    main()
