"""Downloads the historical reference series and writes them, trimmed to 1900-2025 and to
modern-border countries, to data/history/. The raw files stay in data/raw/history/ (gitignored).

Run from the repository root: python3 tools/history/fetch.py [--offline]
`--offline` rebuilds data/history/ from the raw files already downloaded.

Every source, its licence and its attribution are listed in data/history/SOURCES.md; this script
is the only way numbers enter data/history/.
"""

import csv
import io
import os
import sys
import urllib.request
import zipfile

ROOT = os.path.normpath(os.path.join(os.path.dirname(__file__), "..", ".."))
RAW = os.path.join(ROOT, "data", "raw", "history")
OUT = os.path.join(ROOT, "data", "history")
FIRST, LAST = 1900, 2025

OWID = "https://ourworldindata.org/grapher/{}.csv?v=1&csvType=full&useColumnShortNames=true"
OWID_META = "https://ourworldindata.org/grapher/{}.metadata.json?v=1&csvType=full&useColumnShortNames=true"

# name -> (url, raw file)
DOWNLOADS = {
    "maddison": ("https://dataverse.nl/api/access/datafile/421302", "mpd2023_web.xlsx"),
}
OWID_CHARTS = {
    "population": "population",
    "population_wpp": "population-with-un-projections",
    "urban": "long-term-urban-population-region",
    "literacy": "cross-country-literacy-rates",
}
# Physical production: output file -> (chart, query, column, unit). Chart views of OWID.
PRODUCTION = {
    "coal": ("fossil-fuels", "fuel=coal&metric=production&per_capita=total", "coal_production_twh", "TWh"),
    "crude_oil": ("fossil-fuels", "fuel=oil&metric=production&per_capita=total", "oil_production_twh", "TWh"),
    "natural_gas": ("fossil-fuels", "fuel=gas&metric=production&per_capita=total", "gas_production_twh", "TWh"),
    "electricity": ("electricity-mix", "frequency=annual&metric=generation&source=total", "total_generation__twh", "TWh"),
    "steel": ("minerals", "measure=absolute&metric=production&mineral=steel&type=processing__crude", "production_steel_processing__crude_tonnes", "t"),
    "cereals": ("cereal-production", "", "cereals__00001717__production__005510__tonnes", "t"),
}
# World Development Indicators (World Bank API, no key): output column -> indicator.
WDI = {
    "employment_agriculture": "SL.AGR.EMPL.ZS",
    "employment_industry": "SL.IND.EMPL.ZS",
    "employment_services": "SL.SRV.EMPL.ZS",
    "inflation": "FP.CPI.TOTL.ZG",
    "revenue": "GC.REV.XGRT.GD.ZS",
    "expense": "GC.XPN.TOTL.GD.ZS",
    "debt": "GC.DOD.TOTL.GD.ZS",
}
WDI_URL = "https://api.worldbank.org/v2/country/all/indicator/{}?format=json&per_page=20000&date=1960:2025"


def fetch(url, path, tries=4):
    req = urllib.request.Request(url, headers={"User-Agent": "between-frontiers-history/1.0"})
    for attempt in range(tries):
        try:
            with urllib.request.urlopen(req, timeout=300) as r, open(path + ".part", "wb") as f:
                f.write(r.read())
            os.replace(path + ".part", path)
            return
        except OSError as e:
            if attempt == tries - 1:
                raise
            print("  retry after", e)


def download():
    os.makedirs(RAW, exist_ok=True)
    for name, (url, file) in DOWNLOADS.items():
        print("download", name)
        fetch(url, os.path.join(RAW, file))
    for name, slug in OWID_CHARTS.items():
        print("download", slug)
        fetch(OWID.format(slug), os.path.join(RAW, slug + ".csv"))
        fetch(OWID_META.format(slug), os.path.join(RAW, slug + ".metadata.json"))
    for name, (slug, query, _col, _unit) in PRODUCTION.items():
        print("download", name)
        url = OWID.format(slug) + ("&" + query if query else "")
        fetch(url, os.path.join(RAW, f"production_{name}.csv"))
        fetch(OWID_META.format(slug) + ("&" + query if query else ""), os.path.join(RAW, f"production_{name}.metadata.json"))
    for name, ind in WDI.items():
        print("download", ind)
        fetch(WDI_URL.format(ind), os.path.join(RAW, f"wdi_{ind}.json"))


def owid_rows(slug):
    with open(os.path.join(RAW, slug if slug.endswith(".csv") else slug + ".csv"), newline="", encoding="utf-8") as f:
        for r in csv.DictReader(f):
            code = r.get("code") or ""
            # Countries only: OWID's regions and aggregates have no code or an OWID_ code.
            if len(code) != 3:
                continue
            year = int(r["year"])
            if FIRST <= year <= LAST:
                yield code, year, r


def write(name, header, rows, note):
    path = os.path.join(OUT, name)
    with open(path, "w", newline="", encoding="utf-8") as f:
        f.write(f"# {note}\n")
        w = csv.writer(f, lineterminator="\n")
        w.writerow(header)
        for row in sorted(rows):
            w.writerow(row)
    print("wrote", os.path.relpath(path, ROOT), len(rows), "rows")


def num(x, digits):
    v = float(x)
    return int(round(v)) if digits == 0 else round(v, digits)


def build_population():
    rows = {}
    for code, year, r in owid_rows("population"):
        if r["population_historical"]:
            rows[(code, year)] = num(r["population_historical"], 0)
    # UN WPP 2024 medium variant for the years after the last estimate.
    for code, year, r in owid_rows("population-with-un-projections"):
        v = r.get("population__sex_all__age_all__variant_medium__projected")
        if v and (code, year) not in rows:
            rows[(code, year)] = num(v, 0)
    write(
        "population.csv",
        ["iso3", "year", "population"],
        [(c, y, v) for (c, y), v in rows.items()],
        "OWID population (HYDE 3.3, Gapminder v7, UN WPP 2024; WPP medium variant after 2023). See SOURCES.md.",
    )


def build_maddison():
    import openpyxl  # only needed here

    wb = openpyxl.load_workbook(os.path.join(RAW, "mpd2023_web.xlsx"), read_only=True)
    rows = []
    for code, _name, _region, year, gdppc, pop in list(wb["Full data"].iter_rows(values_only=True))[1:]:
        if year is None or not FIRST <= int(year) <= LAST:
            continue
        if gdppc is None and pop is None:
            continue
        # Maddison's SDN is Sudan before the 2011 split; keep it apart from OWID's SDN.
        if code == "SDN":
            code = "SDN_FORMER"
        rows.append(
            (
                code,
                int(year),
                "" if gdppc is None else round(float(gdppc), 1),
                "" if pop is None else int(round(float(pop) * 1000)),
            )
        )
    write(
        "maddison.csv",
        ["code", "year", "gdppc_2011usd", "population"],
        rows,
        "Maddison Project Database 2023 (Bolt and van Zanden 2024), CC BY 4.0. SUN, CSK, YUG and SDN_FORMER are former states. See SOURCES.md.",
    )


def build_share(slug, column, out, note, scale=0.01):
    rows = []
    for code, year, r in owid_rows(slug):
        if r.get(column):
            rows.append((code, year, round(float(r[column]) * scale, 4)))
    write(out, ["iso3", "year", "share"], rows, note)


# Series whose terms forbid passing them on: used from data/raw/history/ on this machine only,
# never written to data/history/ (see SOURCES.md, "Left out").
LOCAL_ONLY = {"steel"}


def countries():
    """Codes of countries and territories: those of the population series, plus Kosovo and the
    Channel Islands, which the World Bank lists apart. Regional and income aggregates drop out."""
    with open(os.path.join(RAW, "population.csv"), newline="", encoding="utf-8") as f:
        codes = {r["code"] for r in csv.DictReader(f) if len(r.get("code") or "") == 3}
    return codes | {"XKX", "CHI"}


def build_production():
    keep = countries()
    rows, local = [], []
    for name, (_slug, _query, col, unit) in PRODUCTION.items():
        for code, year, r in owid_rows(f"production_{name}.csv"):
            if r.get(col) and code in keep:
                row = (code, year, name, round(float(r[col]), 3), unit)
                (local if name in LOCAL_ONLY else rows).append(row)
    if local:
        global OUT
        out, OUT = OUT, RAW
        write("production_local.csv", ["iso3", "year", "good", "value", "unit"], local,
              "Local use only; not redistributable. See data/history/SOURCES.md.")
        OUT = out
    write(
        "production.csv",
        ["iso3", "year", "good", "value", "unit"],
        rows,
        "Production by country: OWID fossil-fuels, electricity-mix and cereal-production (FAOSTAT). See SOURCES.md.",
    )


def build_wdi():
    import json

    rows = []
    keep = countries()
    for name, ind in WDI.items():
        with open(os.path.join(RAW, f"wdi_{ind}.json"), encoding="utf-8") as f:
            data = json.load(f)
        for r in data[1] or []:
            code = r.get("countryiso3code") or ""
            if code in keep and r.get("value") is not None:
                rows.append((code, int(r["date"]), name, round(float(r["value"]), 3)))
    write(
        "wdi.csv",
        ["iso3", "year", "series", "value"],
        rows,
        "World Bank, World Development Indicators (ILO modelled employment, IMF CPI and government finance), CC BY 4.0. Percent. See SOURCES.md.",
    )


def main():
    if "--offline" not in sys.argv:
        download()
    os.makedirs(OUT, exist_ok=True)
    build_population()
    build_maddison()
    build_share(
        "long-term-urban-population-region",
        "urbc_c_share",
        "urban.csv",
        "Share of the population in urban areas, HYDE 3.5 via Our World in Data. See SOURCES.md.",
    )
    build_share(
        "cross-country-literacy-rates",
        "literacy_rate",
        "literacy.csv",
        "Adult literacy rate, Our World in Data (UNESCO, van Zanden et al., World Bank). See SOURCES.md.",
    )
    build_production()
    build_wdi()


if __name__ == "__main__":
    main()
