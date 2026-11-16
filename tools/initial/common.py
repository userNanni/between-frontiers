"""Shared readers for the 1925 initial-conditions tools (plain regex, no RON library)."""

import os
import re

ROOT = os.getcwd()


def read_states():
    """The 870 states of assets/map/provinces.ron: key, name, owner, colony flag, in file order."""
    text = open(os.path.join(ROOT, "assets/map/provinces.ron"), encoding="utf-8").read()
    part = text.split("\n    states: [")[1]
    out = []
    for m in re.finditer(
        r'\(id: (\d+), key: "([^"]+)", name: "((?:[^"\\]|\\.)*)", owner: "([^"]+)", colony: (\w+)', part
    ):
        out.append(dict(key=m.group(2), name=m.group(3), owner=m.group(4), colony=m.group(5) == "true"))
    assert len(out) == 870, len(out)
    return out


def _opt(s):
    return None if s == "None" else float(re.match(r"Some\(([-\d.]+)\)", s).group(1))


def read_countries():
    """tag -> dict(population, urbanization, gdp_per_capita, colonies=None|dict(...)), file order."""
    text = open(os.path.join(ROOT, "data/scenario/1925/countries.ron"), encoding="utf-8").read()
    out = {}
    blocks = re.split(r"\n        \(\n            tag: ", text)[1:]
    for b in blocks:
        tag = re.match(r'"([A-Z]+)"', b).group(1)
        d = dict(
            population=int(re.search(r"\n            population: (\d+)", b).group(1)),
            gdp_per_capita=int(re.search(r"\n            gdp_per_capita: (\d+)", b).group(1)),
            urbanization=_opt(re.search(r"\n            urbanization: (None|Some\([\d.]+\))", b).group(1)),
            status=re.search(r"\n            status: ([^\n]+),", b).group(1),
            colonies=None,
        )
        c = re.search(r"colonies: Some\(\((.*?)\)\),", b, re.S)
        if c:
            cb = c.group(1)
            d["colonies"] = dict(
                population=int(re.search(r"population: (\d+)", cb).group(1)),
                gdp_per_capita=int(re.search(r"gdp_per_capita: (\d+)", cb).group(1)),
                urbanization=_opt(re.search(r"urbanization: (None|Some\([\d.]+\))", cb).group(1)),
            )
        out[tag] = d
    assert len(out) == 83, len(out)
    return out
