#!/usr/bin/env python3
"""Writes data/scenario/1925/forces.ron: armies, navies and air forces on 1925-01-01.

Every number below was read from the source named in its row (tools/war/SOURCES.md lists the
sources, their licences and how they were read). The only computed numbers are:
- sums of components the source lists (said in the row);
- the split of an air force by type when the source gives only its total: the stated shares of
  the air forces whose split is printed (AIR_SPLIT_FROM), applied by largest remainder; those
  rows carry `estimate`.

Usage: python3 tools/war/forces_1925.py  (from the repository root)
"""

from pathlib import Path

AYB = "LoN Armaments Year-Book 1924 (A.37.1924.IX)"
HANSARD_ARMIES = "Hansard HC Deb 18 June 1924 vol 174 cc2150-2W 'European Armies' (War Office, peace strengths 1924)"
HANSARD_AIR = "Hansard HC Deb 15 April 1930 vol 237 cc2755-6W 'Air Forces (International Comparisons)', 1925 column, first-line aircraft"
BRASSEY = "Brassey's Naval and Shipping Annual 1925, 'Ships of the Lesser Navies'"

# ---- Land: men under arms (army only; gendarmerie, police and reserves left out) ----------------
# (tag, type, men, station, source, estimate)
LAND = [
    ("USA", "infantry", 133264, "Metropole", f"{AYB}, United States, p.802: Regular Army 30 June 1924 (11,767 officers + 121,497 enlisted); National Guard and Organized Reserves left out", None),
    ("GBR", "infantry", 152592, "Metropole", f"{AYB}, Great Britain, pp.93-94: British troops 1924-25 excluding India (9,644 officers + 142,948 other ranks); Territorial Army and reserve left out", None),
    ("GBR", "garrison", 3343, "Colonies", f"{AYB}, Great Britain, pp.93-94: colonial and native troops on the British estimates 1924-25", None),
    ("RAJ", "infantry", 215162, "Metropole", f"{AYB}, India, p.270: fighting services 1922-23, sum of 4,467 British officers, 66,078 British other ranks and 144,617 Indian officers and other ranks (British troops in India count here, not in GBR)", None),
    ("FRA", "infantry", 637399, "Metropole", f"{AYB}, France, pp.442-443, Table I: grand total 744,556 (1923 budget) less the colonial forces 107,157; includes the Rhine army and Morocco", None),
    ("FRA", "garrison", 107157, "Colonies", f"{AYB}, France, pp.442-443, Table I: colonial forces (4,371 officers + 102,786 other ranks)", None),
    ("ITA", "infantry", 248000, "Metropole", f"{AYB}, Italy, p.516: budget strength (18,000 officers + 15,000 NCOs + 215,000 other ranks); Carabinieri left out", None),
    ("JAP", "infantry", 232993, "Metropole", f"{AYB}, Japan, p.567: budgetary effectives 1 Jan 1923 (16,879 officers + 216,114 other ranks), the 38,590 in Korea, Formosa and Kwantung included", None),
    ("SOV", "infantry", 530000, "Metropole", "WP 'Red Army' (history): strength on 1 Oct 1924", None),
    ("CHN", "infantry", 1600000, "Metropole", f"{AYB}, China, p.332: 100,000 officers + 1,500,000 soldiers in 110 divisions, the nominal sum of all (largely provincial) armies", None),
    ("GER", "infantry", 100000, "Metropole", f"{AYB}, Germany, p.474; {HANSARD_ARMIES}: 100,000", "Treaty of Versailles ceiling (art. 160), which both sources give as the strength"),
    ("POL", "infantry", 276375, "Metropole", f"{AYB}, Poland, p.645: 1923 budget (18,900 officers + 38,722 NCOs + 218,753 other ranks)", None),
    ("ESP", "infantry", 230500, "Metropole", f"{AYB}, Spain, p.690: May 1924 (19,000 officers + 211,500 other ranks), Morocco included; Guardia Civil left out", None),
    ("CZE", "infantry", 149877, "Metropole", f"{HANSARD_ARMIES}", None),
    ("ROM", "infantry", 146564, "Metropole", f"{AYB}, Roumania, p.669: budgetary, sum of 11,379 officers + 10,185 re-enlisted + 125,000 NCOs and men", None),
    ("YUG", "infantry", 130000, "Metropole", f"{HANSARD_ARMIES} (the Year-Book figure is illegible in the scan)", None),
    ("TUR", "infantry", 88000, "Metropole", f"{HANSARD_ARMIES}", None),
    ("BEL", "infantry", 80936, "Metropole", f"{AYB}, Belgium, p.47: budget effectives 1924 (5,410 officers + 10,845 NCOs + 63,181 other ranks + 1,500 employees); gendarmerie left out", None),
    ("NLD", "infantry", 163262, "Metropole", f"{HANSARD_ARMIES} (the Year-Book gives no total for the home army)", None),
    ("NLD", "garrison", 34330, "Colonies", f"{AYB}, Netherlands, pp.597-598: colonial armies (1,130 officers + 33,200 other ranks); East Indies gendarmerie left out", None),
    ("SWE", "infantry", 35946, "Metropole", f"{AYB}, Sweden, p.719: 1923 average, sum of 2,802 officers + 2,092 NCOs + 10,574 enlisted + 20,478 conscripts", None),
    ("ARG", "infantry", 28023, "Metropole", f"{AYB}, Argentina, p.16: 1923-24 budget, sum of the listed officers, NCOs, volunteers and 21,000 conscripts", None),
    ("BRA", "infantry", 45255, "Metropole", f"{AYB}, Brazil, p.67: 4,862 officers (establishment) + 40,393 NCOs and men (1924 budget)", None),
    ("CAN", "infantry", 3643, "Metropole", f"{AYB}, Canada, p.227: Permanent Force 1 Jan 1923 (428 officers + 3,215 other ranks); militia left out", None),
    ("NZL", "infantry", 3989, "Metropole", f"{AYB}, New Zealand, pp.286-287: Permanent and Territorial, sum of the components (the scan's total reads 13,989)", None),
    ("AUT", "infantry", 21500, "Metropole", f"{HANSARD_ARMIES}: actual strength, under the 30,000 of Saint-Germain", None),
    ("HUN", "infantry", 35000, "Metropole", f"{HANSARD_ARMIES}", "Treaty of Trianon ceiling, which the source gives as the strength"),
    ("BUL", "infantry", 20000, "Metropole", f"{AYB}, Bulgaria, p.316; {HANSARD_ARMIES}", "Treaty of Neuilly ceiling, which both sources give as the strength"),
    ("GRE", "infantry", 68565, "Metropole", f"{AYB}, Greece, p.488: budgetary effectives (5,558 officers + 63,007 other ranks); gendarmerie left out", None),
    ("POR", "infantry", 40000, "Metropole", f"{HANSARD_ARMIES}", None),
    ("DEN", "infantry", 11061, "Metropole", f"{AYB}, Denmark, p.373: budgetary, sum of the listed officers, NCOs and 8,625 other ranks", None),
    ("NOR", "infantry", 16000, "Metropole", f"{HANSARD_ARMIES}", None),
    ("FIN", "infantry", 26207, "Metropole", f"{AYB}, Finland, p.392: army without the navy and the air force", None),
    ("CHL", "infantry", 18253, "Metropole", f"{AYB}, Chile, p.328: effectives, sum of officers, NCOs, men and 8,690 conscripts", None),
    ("EST", "infantry", 20100, "Metropole", f"{AYB}, Esthonia, p.387: budgetary 1923 (1,600 officers + 18,500 other ranks)", None),
    ("LAT", "infantry", 19156, "Metropole", f"{AYB}, Latvia, p.578: budgetary 1923 (1,824 officers + 17,332 other ranks)", None),
    ("LTU", "infantry", 15000, "Metropole", f"{HANSARD_ARMIES}", None),
    ("ALB", "infantry", 7557, "Metropole", f"{AYB}, Albania, p.11: budgetary 1923 (355 officers + 7,202 other ranks); gendarmerie left out", None),
    ("LUX", "garrison", 296, "Metropole", f"{AYB}, Luxemburg, p.580: 7 officers + 289 other ranks", None),
    ("PRS", "infantry", 40000, "Metropole", "K. Farrokh, 'Organization of the Iranian Army in 1921-1941' (kavehfarrokh.com): 40,000 in 1926", "1926 figure taken for 1925"),
    ("IRQ", "infantry", 5772, "Metropole", "WP 'Iraqi Army' (history): 5,772 in 1924; British-officered Levies left out", None),
    ("IRL", "infantry", 17000, "Metropole", "RTE News 'Army mutiny 1924' (2024-03-06): demobilisation target of March 1924", "planned establishment after the 1924 demobilisation, taken as strength"),
    ("MEX", "infantry", 80000, "Metropole", "WP 'Mexican Army' (post-revolution): more than 80,000 under arms in 1920", "1920 figure, the latest found, taken for 1925; the army shrank in the 1920s"),
]

# ---- Sea: hulls by type, with the total tonnage when the source gives it ------------------------
# (tag, type, ships, tonnage or None, source, estimate)
def ayb_navy(country, page):
    return f"{AYB}, {country}, Navy, p.{page}: ships in the fleet, 1924 (active and reserve not told apart)"

SEA = [
    ("USA", "battleship", 18, 525850, ayb_navy("United States", 810), None),
    ("USA", "carrier", 1, 19300, ayb_navy("United States", 810), None),
    ("USA", "cruiser", 30, 240840, ayb_navy("United States", 810), None),
    ("USA", "coastal", 30, 29650, ayb_navy("United States", 810) + "; coast-defence ships and monitors", None),
    ("USA", "destroyer", 295, 340940, ayb_navy("United States", 810), None),
    ("USA", "submarine", 126, 86721, ayb_navy("United States", 810), None),
    ("GBR", "battleship", 22, None, ayb_navy("Great Britain", 105) + "; battleships and battle cruisers", None),
    ("GBR", "carrier", 4, None, ayb_navy("Great Britain", 105), None),
    ("GBR", "cruiser", 44, None, ayb_navy("Great Britain", 105), None),
    ("GBR", "coastal", 2, None, ayb_navy("Great Britain", 105), None),
    ("GBR", "destroyer", 172, None, ayb_navy("Great Britain", 105), None),
    ("GBR", "submarine", 59, None, ayb_navy("Great Britain", 105), None),
    ("JAP", "battleship", 10, 301320, ayb_navy("Japan", 569), None),
    ("JAP", "carrier", 2, 15375, ayb_navy("Japan", 569), None),
    ("JAP", "cruiser", 14, 60160, ayb_navy("Japan", 569), None),
    ("JAP", "coastal", 12, 82026, ayb_navy("Japan", 569), None),
    ("JAP", "destroyer", 111, 78983, ayb_navy("Japan", 569), None),
    ("JAP", "submarine", 37, 24377, ayb_navy("Japan", 569), None),
    ("FRA", "battleship", 9, 195720, ayb_navy("France", 458), None),
    ("FRA", "cruiser", 18, 163958, ayb_navy("France", 458), None),
    ("FRA", "destroyer", 78, 45915, ayb_navy("France", 458) + "; destroyers and torpedo boats", None),
    ("FRA", "submarine", 48, 30057, ayb_navy("France", 458), None),
    ("ITA", "battleship", 5, 109900, ayb_navy("Italy", 542), None),
    ("ITA", "coastal", 11, 31200, ayb_navy("Italy", 542), None),
    ("ITA", "cruiser", 15, 80600, ayb_navy("Italy", 542), None),
    ("ITA", "destroyer", 135, 64200, ayb_navy("Italy", 542) + "; destroyers and torpedo boats", None),
    ("ITA", "submarine", 43, 18100, ayb_navy("Italy", 542), None),
    ("SOV", "battleship", 3, 70110, ayb_navy("USSR", 782), None),
    ("SOV", "cruiser", 2, 13550, ayb_navy("USSR", 782), None),
    ("SOV", "destroyer", 15, 15210, ayb_navy("USSR", 782) + "; torpedo boats and destroyers", None),
    ("SOV", "submarine", 9, 5450, ayb_navy("USSR", 782), None),
    ("GER", "battleship", 8, None, "WP 'Reichsmarine', list of ships: counted in service on 1925-01-01 (6 pre-dreadnoughts + 2 in reserve, within Versailles art. 181)", None),
    ("GER", "cruiser", 8, None, "WP 'Reichsmarine', list of ships: counted in service on 1925-01-01 (6 light cruisers + 2 in reserve)", None),
    ("ESP", "battleship", 3, 47100, ayb_navy("Spain", 700), None),
    ("ESP", "carrier", 1, None, ayb_navy("Spain", 700) + "; attached to the flying school", None),
    ("ESP", "cruiser", 5, 33138, ayb_navy("Spain", 700), None),
    ("ESP", "destroyer", 28, 6762, ayb_navy("Spain", 700) + "; destroyers and torpedo boats", None),
    ("ESP", "submarine", 7, 3008, ayb_navy("Spain", 700), None),
    ("ARG", "battleship", 2, 55880, ayb_navy("Argentina", 17), None),
    ("ARG", "coastal", 2, 4600, ayb_navy("Argentina", 17), None),
    ("ARG", "cruiser", 6, 34880, ayb_navy("Argentina", 17), None),
    ("ARG", "destroyer", 12, 5300, ayb_navy("Argentina", 17) + "; destroyers and torpedo boats", None),
    ("BRA", "battleship", 2, None, ayb_navy("Brazil", 71), None),
    ("BRA", "coastal", 1, None, ayb_navy("Brazil", 71), None),
    ("BRA", "cruiser", 3, None, ayb_navy("Brazil", 71), None),
    ("BRA", "destroyer", 11, None, ayb_navy("Brazil", 71), None),
    ("BRA", "submarine", 3, None, ayb_navy("Brazil", 71), None),
    ("CHL", "battleship", 1, 28000, ayb_navy("Chile", 329), None),
    ("CHL", "cruiser", 5, 27800, ayb_navy("Chile", 329), None),
    ("CHL", "destroyer", 12, 10300, ayb_navy("Chile", 329), None),
    ("CHL", "submarine", 6, 2200, ayb_navy("Chile", 329), None),
    ("NLD", "coastal", 4, 21910, ayb_navy("Netherlands", 599) + "; 1 Jan 1924, colonies included", None),
    ("NLD", "cruiser", 2, 14100, ayb_navy("Netherlands", 599), None),
    ("NLD", "destroyer", 17, 7150, ayb_navy("Netherlands", 599) + "; destroyers and torpedo boats", None),
    ("NLD", "submarine", 18, 6810, ayb_navy("Netherlands", 599), None),
    ("SWE", "coastal", 12, 42213, ayb_navy("Sweden", 720) + "; coast-defence ships and monitors", None),
    ("SWE", "cruiser", 1, 4940, ayb_navy("Sweden", 720), None),
    ("SWE", "destroyer", 39, 6216, ayb_navy("Sweden", 720) + "; torpedo boats and destroyers", None),
    ("SWE", "submarine", 16, 5100, ayb_navy("Sweden", 720), None),
    ("AUS", "cruiser", 6, 29830, ayb_navy("Australia", 207), None),
    ("AUS", "destroyer", 12, 11245, ayb_navy("Australia", 207), None),
    ("AUS", "submarine", 6, 7200, ayb_navy("Australia", 207), None),
    ("CAN", "cruiser", 1, 3500, ayb_navy("Canada", 230), None),
    ("CAN", "destroyer", 2, 2008, ayb_navy("Canada", 230), None),
    ("CAN", "submarine", 2, 728, ayb_navy("Canada", 230), None),
    ("DEN", "coastal", 8, 23183, ayb_navy("Denmark", 376) + "; coast-defence ships and monitors", None),
    ("DEN", "destroyer", 23, 3760, ayb_navy("Denmark", 376) + "; torpedo boats", None),
    ("DEN", "submarine", 14, 2810, ayb_navy("Denmark", 376), None),
    ("NOR", "coastal", 4, 16000, ayb_navy("Norway", 619), None),
    ("NOR", "destroyer", 32, 6000, ayb_navy("Norway", 619) + "; torpedo boats and destroyers", None),
    ("NOR", "submarine", 4, 1400, ayb_navy("Norway", 619), None),
    ("FIN", "coastal", 4, 1524, ayb_navy("Finland", 393) + "; coastguard vessels", None),
    ("FIN", "destroyer", 5, 800, ayb_navy("Finland", 393) + "; torpedo boats", None),
    ("GRE", "battleship", 3, 41300, ayb_navy("Greece", 489), None),
    ("GRE", "cruiser", 1, 2930, ayb_navy("Greece", 489), None),
    ("GRE", "destroyer", 17, 8250, ayb_navy("Greece", 489), None),
    ("POL", "destroyer", 6, 2125, ayb_navy("Poland", 650) + "; torpedo boats and destroyers", None),
    ("YUG", "destroyer", 12, 2800, ayb_navy("Kingdom of the Serbs, Croats and Slovenes", 671) + "; torpedo boats", None),
    ("ROM", "coastal", 7, None, f"{AYB}, Roumania, pp.663-664: Danube Division, river monitors", None),
    ("ROM", "destroyer", 8, None, f"{AYB}, Roumania, pp.663-664: Maritime Division, 2 destroyers + 6 torpedo boats", None),
    ("CHN", "cruiser", 6, 18500, ayb_navy("China", 333), None),
    ("CHN", "destroyer", 11, 1860, ayb_navy("China", 333) + "; destroyers and torpedo boats", None),
    ("TUR", "battleship", 2, None, "WP 'Turkish Naval Forces' (Republic, 1923), citing Langensiepen & Gueleryuez p.59: list of 29 Oct 1923, both needing repair", None),
    ("TUR", "cruiser", 4, None, "WP 'Turkish Naval Forces' (Republic, 1923): list of 29 Oct 1923, 2 active and 2 out of service", None),
    ("TUR", "destroyer", 11, None, "WP 'Turkish Naval Forces' (Republic, 1923): list of 29 Oct 1923, 5 destroyers and 6 torpedo boats, all but one out of service", None),
    ("POR", "cruiser", 2, None, f"{BRASSEY}, pp.422-423: counted (Adamastor, Sao Gabriel; Almirante Reis dismantled)", None),
    ("POR", "destroyer", 4, None, f"{BRASSEY}, pp.422-423: counted", None),
    ("POR", "submarine", 4, None, f"{BRASSEY}, pp.422-423: counted", None),
    ("SIA", "destroyer", 3, None, f"{BRASSEY}, p.425: counted", None),
    ("URY", "cruiser", 2, None, f"{BRASSEY}, p.425: counted (Montevideo, Uruguay)", None),
    ("URY", "destroyer", 1, None, f"{BRASSEY}, p.425: counted; a torpedo boat", None),
]

# ---- Air: aircraft by type ----------------------------------------------------------------------
# Split printed by the source: (tag, fighter, attack (bombing), patrol (reconnaissance, scouts),
# naval_aviation, source). Sources counting fighting / bombing / reconnaissance machines.
AIR_SPLIT = [
    ("GBR", 117, 241, 219, 0, f"{AYB}, Great Britain, pp.103-104: RAF establishment 1924-25 (117 fighting, 241 bombing, 219 reconnaissance; 22 miscellaneous left out)"),
    ("JAP", 112, 0, 240, 196, f"{AYB}, Japan, p.565: army 112 fighting + 240 reconnaissance, navy 196; spares included, training left out"),
    ("CZE", 94, 0, 134, 0, f"{AYB}, Czechoslovakia, p.344: 94 fighting + 134 scouts, training left out"),
    ("ESP", 51, 0, 344, 0, f"{AYB}, Spain, p.687: 51 fighting + 344 scouts, colonies included, training left out"),
    ("SWE", 25, 0, 40, 0, f"{AYB}, Sweden, p.717: army 25 fighting + 25 scouts, navy 15 (counted as patrol)"),
    ("CHE", 60, 0, 150, 0, f"{AYB}, Switzerland, p.742: 60 fighting + 150 scouts, training left out"),
    ("EST", 13, 18, 26, 0, f"{AYB}, Esthonia, p.387: 13 fighting + 18 bombing + 26 reconnaissance"),
    ("LAT", 8, 0, 11, 0, f"{AYB}, Latvia, p.578: 8 fighting + 11 reconnaissance"),
    ("YUG", 6, 0, 10, 0, f"{AYB}, Kingdom of the Serbs, Croats and Slovenes, p.670: 6 fighting + 10 reconnaissance"),
    ("NLD", 24, 0, 74, 0, f"{AYB}, Netherlands, pp.588, 598, 599: home 72 (split not printed), East Indies 6 fighting + 24 reconnaissance, navy 18 fighting + 50 scouting; home 72 counted below"),
]
# Shares used to split the totals below: fighters, bombers and reconnaissance of the forces above
# whose split covers all three kinds or two of them (GBR, JAP army, CZE, ESP, SWE, CHE, EST, LAT,
# YUG), computed in `shares()`.
AIR_SPLIT_FROM = ["GBR", "JAP", "CZE", "ESP", "SWE", "CHE", "EST", "LAT", "YUG"]

# Totals only: (tag, aircraft, source)
AIR_TOTAL = [
    ("FRA", 1280, HANSARD_AIR),
    ("USA", 750, HANSARD_AIR + "; army and navy"),
    ("ITA", 600, HANSARD_AIR),
    ("NLD", 72, f"{AYB}, Netherlands, p.588: home air force, bombing, fighting and scouting, training left out"),
    ("FIN", 69, f"{AYB}, Finland, p.390: aircraft approved at admission to the League (1920)"),
    ("CHL", 32, f"{AYB}, Chile, p.328: aircraft for training or war"),
    ("CAN", 29, f"{AYB}, Canada, p.227: machines on 1 Jan 1923"),
    ("RAJ", 72, f"{AYB}, India, p.252: six squadrons of twelve aeroplanes on a peace basis"),
]
# Single-type forces: (tag, type, aircraft, source)
AIR_SINGLE = [
    ("DEN", "patrol", 26, f"{AYB}, Denmark, p.368: naval reconnaissance aircraft (the army had school machines only)"),
    ("NZL", "patrol", 4, f"{AYB}, New Zealand, pp.286-287: air force equipment (9 more lent to civil companies left out)"),
    ("ALB", "patrol", 3, f"{AYB}, Albania, p.11: reconnaissance"),
]

ORDER = [
    "USA", "GBR", "RAJ", "CAN", "AUS", "NZL", "FRA", "GER", "ITA", "JAP", "SOV", "CHN", "POL", "ESP",
    "CZE", "ROM", "YUG", "TUR", "BEL", "NLD", "SWE", "CHE", "AUT", "HUN", "BUL", "GRE", "POR", "DEN",
    "NOR", "FIN", "EST", "LAT", "LTU", "ALB", "LUX", "ARG", "BRA", "CHL", "URY", "MEX", "PRS", "IRQ",
    "IRL", "SIA",
]


def shares():
    f = b = r = 0
    for tag, fi, bo, re_, _, _ in AIR_SPLIT:
        if tag in AIR_SPLIT_FROM:
            f, b, r = f + fi, b + bo, r + re_
    return f, b, r


def split(total, weights):
    s = sum(weights)
    raw = [total * w for w in weights]
    out = [x // s for x in raw]
    left = total - sum(out)
    order = sorted(range(len(weights)), key=lambda i: (-(raw[i] % s), i))
    for i in order[:left]:
        out[i] += 1
    return out


def ron_str(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def main():
    rows = {}

    def add(tag, row):
        rows.setdefault(tag, []).append(row)

    for tag, ty, men, station, source, est in LAND:
        add(tag, (ty, men, station, None, source, est))
    for tag, ty, n, tonnage, source, est in SEA:
        add(tag, (ty, n, "Metropole", tonnage, source, est))
    for tag, fi, bo, re_, na, source in AIR_SPLIT:
        for ty, n in (("fighter", fi), ("attack", bo), ("patrol", re_), ("naval_aviation", na)):
            if n:
                add(tag, (ty, n, "Metropole", None, source, None))
    f, b, r = shares()
    method = (f"split by the shares of fighters, bombers and reconnaissance aircraft in the air forces "
              f"whose split the Year-Book prints ({', '.join(AIR_SPLIT_FROM)}): {f}:{b}:{r}")
    for tag, total, source in AIR_TOTAL:
        nf, nb, nr = split(total, [f, b, r])
        for ty, n in (("fighter", nf), ("attack", nb), ("patrol", nr)):
            if n:
                existing = [x for x in rows.get(tag, []) if x[0] == ty]
                if existing:
                    # Add to a row of the same type (Netherlands: home and East Indies).
                    i = rows[tag].index(existing[0])
                    t, c, st, ton, src, est = existing[0]
                    rows[tag][i] = (t, c + n, st, ton, src + f"; plus {n} of the {source}", method)
                else:
                    add(tag, (ty, n, "Metropole", None, f"{source}: total {total}", method))
    for tag, ty, n, source in AIR_SINGLE:
        add(tag, (ty, n, "Metropole", None, source, None))

    missing = sorted(set(rows) - set(ORDER))
    assert not missing, missing
    out = [
        "// Generated by tools/war/forces_1925.py; do not edit by hand. Sources: tools/war/SOURCES.md.",
        "// Between Frontiers: armies, navies and air forces on 1925-01-01 (phase 5, not wired into the",
        "// simulation yet). Contract: crates/bf-sim/src/war/forces.rs (ForcesFile); catalogue in",
        "// data/military/units.ron. Licence: CC BY-SA 4.0.",
        "//",
        "// count: men (land), ships (sea), aircraft (air). Land: the army only (gendarmerie, police,",
        "// militia and reserves left out); in 1925 the whole army counts as `infantry`, its guns and",
        "// horses inside the element's equipment, until a source splits it by arm. Sea: ships in the",
        "// fleet, active and reserve; `tonnage` is the source's total for the row. The figures are of",
        "// 1923-1924, the nearest to the start date the sources print; rows whose number the source",
        "// does not state carry `estimate`.",
        "(",
        '    start_date: "1925-01-01",',
        "    countries: [",
    ]
    for tag in ORDER:
        if tag not in rows:
            continue
        out.append(f'        (tag: "{tag}", rows: [')
        for ty, n, station, tonnage, source, est in rows[tag]:
            parts = [f'type: "{ty}"', f"count: {n}"]
            if station != "Metropole":
                parts.append(f"station: {station}")
            if tonnage is not None:
                parts.append(f"tonnage: Some({tonnage})")
            parts.append(f"source: {ron_str(source)}")
            if est:
                parts.append(f"estimate: Some({ron_str(est)})")
            out.append("            (" + ", ".join(parts) + "),")
        out.append("        ]),")
    out += ["    ],", ")", ""]
    Path("data/scenario/1925/forces.ron").write_text("\n".join(out))


if __name__ == "__main__":
    main()
