"""Downloads the historical reference series and writes them, trimmed to 1900-2025 and to
modern-border countries, to data/history/. The raw files stay in data/raw/history/ (gitignored).

Run from the repository root: python3 tools/history/fetch.py [--offline] [--only NAME]
`--offline` rebuilds data/history/ from the raw files already downloaded; `--only fdi` downloads
and rebuilds only that file (the names are the build_* steps in `main`).

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
    # IBGE / Ministério do Trabalho, "Movimento da População: Estatística Migratória de 1952"
    # (1955): table 8, immigrants into Brazil by nationality, 1884-1952.
    "brazil_immigration": ("https://biblioteca.ibge.gov.br/visualizacao/livros/liv84438.pdf", "ibge_liv84438.pdf"),
    # IBGE, Recenseamento Geral do Brasil de 1940, Sinopse (special edition of 1947): foreign
    # nationals by nationality, note 7 of the table for the whole country.
    "brazil_census_1940": ("https://biblioteca.ibge.gov.br/visualizacao/periodicos/314/cd_1940_dadosgerais_edespecial_1947.pdf", "ibge_census1940_sinopse1947.pdf"),
    # US DHS, Yearbook of Immigration Statistics 2023, table 1 (lawful permanent residents).
    "usa_immigration": ("https://ohss.dhs.gov/topics/immigration/yearbook/2023/table1", "dhs_yearbook2023_table1.html"),
}
OWID_CHARTS = {
    "population": "population",
    "population_wpp": "population-with-un-projections",
    "urban": "long-term-urban-population-region",
    "literacy": "cross-country-literacy-rates",
    "births": "crude-birth-rate",
    "deaths": "crude-death-rate",
    "life_expectancy": "life-expectancy",
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
# Foreign direct investment (World Bank WDI from the IMF balance of payments, with UNCTAD where the
# IMF has no series): output column -> indicator. Net flows, current US$, from 1970.
FDI = {
    "fdi_inflow_usd": "BX.KLT.DINV.CD.WD",
    "fdi_outflow_usd": "BM.KLT.DINV.CD.WD",
    "fdi_inflow_gdp": "BX.KLT.DINV.WD.GD.ZS",
}


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


def download_wdi(indicators):
    os.makedirs(RAW, exist_ok=True)
    for ind in indicators.values():
        print("download", ind)
        fetch(WDI_URL.format(ind), os.path.join(RAW, f"wdi_{ind}.json"))


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
    download_wdi(WDI)
    download_wdi(FDI)


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


def build_vital():
    """Births and deaths per thousand (UN WPP, 1950-) and life expectancy at birth (long run)."""
    columns = {
        "births": ("crude-birth-rate", "birth_rate__sex_all__age_all__variant_estimates"),
        "deaths": ("crude-death-rate", "death_rate__sex_all__age_all__variant_estimates"),
        "life_expectancy": ("life-expectancy", "life_expectancy_0"),
    }
    rows = {}
    for k, (slug, column) in columns.items():
        for code, year, r in owid_rows(slug):
            if r.get(column):
                rows.setdefault((code, year), {})[k] = round(float(r[column]), 2)
    write(
        "vital.csv",
        ["iso3", "year", "births", "deaths", "life_expectancy"],
        [
            (c, y, v.get("births", ""), v.get("deaths", ""), v.get("life_expectancy", ""))
            for (c, y), v in rows.items()
        ],
        "Births and deaths per 1,000 (UN WPP 2024, 1950-) and life expectancy at birth (Riley; Zijdeman et al.; HMD; UN WPP), via Our World in Data. See SOURCES.md.",
    )


# Brazil's table 8 columns after the total, and the modern country each nationality is.
BRAZIL_ORIGINS = [("DEU", "germans"), ("ESP", "spaniards"), ("ITA", "italians"), ("JPN", "japanese"),
                  ("PRT", "portuguese"), ("RUS", "russians"), ("other", "others")]


def brazil_rows(text, first=1925, last=1952):
    """Rows of table 8 read from the PDF text. The scan has OCR noise (I for 1, O for 0); a
    year is kept only when its seven nationalities add up to its total, so nothing guessed
    enters the file."""
    import re

    fix = str.maketrans({"I": "1", "l": "1", "i": "1", "O": "0", "o": "0", "S": "5", "s": "5", "!": "1"})
    rows = {}
    for line in text.splitlines():
        m = re.match(r"^\s*([I1l][89I][0-9I][0-9I])\s*[\. ·•'o]*(.*)$", line)
        if not m:
            continue
        year = int(m.group(1).translate(fix))
        if not first <= year <= last:
            continue
        rest = re.sub(r"^[\s\.·•'o]+", "", m.group(2))
        vals = []
        for g in [g for g in re.split(r"\s{2,}", rest.strip()) if g]:
            if re.fullmatch(r"-+", g.strip()):
                vals.append(0)
                continue
            t = g.strip().translate(fix).replace(" ", "")
            if not re.fullmatch(r"\d+", t):
                vals = None
                break
            vals.append(int(t))
        if vals and len(vals) == 8 and sum(vals[1:]) == vals[0]:
            rows[year] = vals
    return rows


# Nationalities of note 7 of the 1940 census and the modern country of each.
BRAZIL_1940 = {"portuguesa": "PRT", "italiana": "ITA", "espanhola": "ESP", "japonesa": "JPN",
               "alema": "DEU", "siria": "SYR"}


def brazil_1940(text):
    """Foreign nationals in Brazil in 1940 from the national note 7 ("Nacionalidades
    predominantes"), the first in the volume. The scan's OCR splits words and numbers with
    spaces; a nationality is kept only when it is one of BRAZIL_1940 and both its counts read
    as numbers."""
    import re
    import unicodedata

    start = text.find("Nacionalidades predominantes")
    if start < 0:
        return {}
    chunk = text[start:start + 600]
    end = chunk.find("Discrimina")
    chunk = chunk[:end if end > 0 else None].replace("\n", " ")
    out = {}
    for m in re.finditer(r"([A-Za-zÀ-ÿ ]+?)\s*,\s*([\d ]+?)\s+homens\s+[ce]\s+([\d ]+?)\s+mulh", chunk):
        name = unicodedata.normalize("NFKD", m.group(1).replace(" ", "").lower())
        name = "".join(c for c in name if not unicodedata.combining(c)).split(":")[-1]
        name = name.replace("predominantes", "")
        if name in BRAZIL_1940:
            men, women = (int(x.replace(" ", "")) for x in (m.group(2), m.group(3)))
            out[BRAZIL_1940[name]] = men + women
    return out


def build_immigration():
    import html
    import re
    import subprocess

    out = []
    pdf = os.path.join(RAW, "ibge_liv84438.pdf")
    if os.path.exists(pdf):
        text = subprocess.run(["pdftotext", "-layout", pdf, "-"], capture_output=True, text=True, check=True).stdout
        for year, vals in sorted(brazil_rows(text).items()):
            for (iso, _), v in zip(BRAZIL_ORIGINS, vals[1:]):
                out.append(("BRA", year, iso, v))
    census = os.path.join(RAW, "ibge_census1940_sinopse1947.pdf")
    if os.path.exists(census):
        text = subprocess.run(["pdftotext", "-layout", census, "-"], capture_output=True, text=True, check=True).stdout
        for iso, n in sorted(brazil_1940(text).items()):
            # A stock, not arrivals: `year` 1940 and origin prefixed with "stock:".
            out.append(("BRA", 1940, "stock:" + iso, n))
    page = os.path.join(RAW, "dhs_yearbook2023_table1.html")
    if os.path.exists(page):
        with open(page, encoding="utf-8") as f:
            cells = [html.unescape(re.sub("<[^>]+>", "", c)).strip()
                     for c in re.findall(r"<t[dh][^>]*>(.*?)</t[dh]>", f.read(), re.S)]
        for a, b in zip(cells, cells[1:]):
            if re.fullmatch(r"\d{4}", a) and re.fullmatch(r"[\d,]+", b) and FIRST <= int(a) <= LAST:
                out.append(("USA", int(a), "all", int(b.replace(",", ""))))
    write(
        "immigration.csv",
        ["destination", "year", "origin", "arrivals"],
        out,
        "Immigrants arriving: Brazil by nationality (IBGE/MTIC, Estatistica Migratoria de 1952, table 8; years whose columns add up to the total) and USA (DHS Yearbook 2023, table 1). See SOURCES.md.",
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


def wdi_rows(indicators, digits=3):
    import json

    rows = []
    keep = countries()
    for name, ind in indicators.items():
        with open(os.path.join(RAW, f"wdi_{ind}.json"), encoding="utf-8") as f:
            data = json.load(f)
        for r in data[1] or []:
            code = r.get("countryiso3code") or ""
            if code in keep and r.get("value") is not None:
                rows.append((code, int(r["date"]), name, num(r["value"], 0 if name.endswith("_usd") else digits)))
    return rows


def build_wdi():
    write(
        "wdi.csv",
        ["iso3", "year", "series", "value"],
        wdi_rows(WDI),
        "World Bank, World Development Indicators (ILO modelled employment, IMF CPI and government finance), CC BY 4.0. Percent. See SOURCES.md.",
    )


def build_fdi():
    write(
        "fdi.csv",
        ["iso3", "year", "series", "value"],
        wdi_rows(FDI),
        "Foreign direct investment, net inflows and outflows (current US$) and net inflows (% of GDP): World Bank WDI after IMF BoP and UNCTAD, CC BY 4.0. See SOURCES.md.",
    )


def main():
    if "--only" in sys.argv:
        only = sys.argv[sys.argv.index("--only") + 1]
        if only != "fdi":
            sys.exit(f"--only: unknown file {only!r} (known: fdi)")
        if "--offline" not in sys.argv:
            download_wdi(FDI)
        if not os.path.exists(os.path.join(RAW, "population.csv")):
            fetch(OWID.format("population"), os.path.join(RAW, "population.csv"))
        build_fdi()
        return
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
    build_vital()
    build_immigration()
    build_production()
    build_wdi()
    build_fdi()


if __name__ == "__main__":
    main()
