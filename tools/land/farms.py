"""Builds data/scenario/1925/land.ron from tools/land/state_stats.csv.

Run from the repository root, after tools/land/aggregate.py:

    python3 -I tools/land/farms.py

It also writes tools/land/check.md: world totals and, for each farm kind, the states that rank
highest by suitability × the land the farm uses, next to the 1925 producing regions.

The rules are documented, with the same numbers, in data/scenario/1925/LAND.md.
"""

import csv
import math
import os

ROOT = os.getcwd()
STATS = os.path.join(ROOT, "tools", "land", "state_stats.csv")
OUT = os.path.join(ROOT, "data", "scenario", "1925", "land.ron")
CHECK = os.path.join(ROOT, "tools", "land", "check.md")

CLIMATES = [
    "tropical_rainforest", "tropical_monsoon", "tropical_savanna", "hot_desert", "cold_desert",
    "hot_steppe", "cold_steppe", "mediterranean", "humid_subtropical", "oceanic",
    "humid_continental", "subarctic", "tundra", "ice_cap",
]
FARMS = [
    "grain_farm", "rice_paddy", "maize_farm", "livestock_ranch", "dairy_farm", "cotton_plantation",
    "sugarcane_plantation", "sugar_beet_farm", "coffee_plantation", "tea_plantation",
    "cocoa_plantation", "rubber_plantation", "tobacco_plantation", "fruit_plantation",
    "fiber_plantation", "vineyard", "forestry", "fishery",
]
KOPPEN_TO_CLIMATE = {
    "Af": "tropical_rainforest", "Am": "tropical_monsoon", "Aw": "tropical_savanna",
    "BWh": "hot_desert", "BWk": "cold_desert", "BSh": "hot_steppe", "BSk": "cold_steppe",
    "Csa": "mediterranean", "Csb": "mediterranean", "Csc": "mediterranean",
    "Cwa": "humid_subtropical", "Cfa": "humid_subtropical",
    "Cfb": "oceanic", "Cfc": "oceanic", "Cwb": "oceanic", "Cwc": "oceanic",
    "Dfa": "humid_continental", "Dfb": "humid_continental", "Dwa": "humid_continental",
    "Dwb": "humid_continental", "Dsa": "humid_continental", "Dsb": "humid_continental",
    "Dfc": "subarctic", "Dfd": "subarctic", "Dwc": "subarctic", "Dwd": "subarctic",
    "Dsc": "subarctic", "Dsd": "subarctic", "ET": "tundra", "EF": "ice_cap",
}
# Tropical highlands: temperate (C) climates in a state whose centre is within this latitude.
HIGHLAND_LAT = 25.0

# Climate weights for each farm, over the 14 climates plus three derived zones that the 14 keys
# hide: `cont_hot` (hot-summer continental, Dfa/Dwa/Dsa, taken out of humid_continental),
# `highland_cool` (oceanic and mediterranean climates in the tropics: Cwb/Cfb/Csb on plateaus) and
# `highland_warm` (humid subtropical in the tropics: Cwa/Cfa on lower plateaus).
# The climate fit is the share-weighted sum of these weights.
WEIGHTS = {
    "grain_farm": dict(oceanic=0.9, humid_continental=1.0, cont_hot=0.9, cold_steppe=0.9,
                       mediterranean=0.8, humid_subtropical=0.6, hot_steppe=0.35, subarctic=0.3,
                       highland_cool=0.5, highland_warm=0.3, tropical_savanna=0.1, cold_desert=0.15,
                       hot_desert=0.05),
    "rice_paddy": dict(tropical_monsoon=0.9, tropical_rainforest=0.7, tropical_savanna=0.6,
                       humid_subtropical=0.6, highland_warm=0.5),
    "maize_farm": dict(humid_subtropical=1.0, cont_hot=1.0, humid_continental=0.45,
                       tropical_savanna=0.7, highland_warm=0.9, highland_cool=0.8,
                       tropical_monsoon=0.5, tropical_rainforest=0.3, mediterranean=0.45, oceanic=0.3,
                       hot_steppe=0.4, cold_steppe=0.3),
    "livestock_ranch": dict(cold_steppe=1.0, hot_steppe=0.9, cold_desert=0.5, hot_desert=0.25,
                            tropical_savanna=0.7, humid_subtropical=0.5, mediterranean=0.5,
                            oceanic=0.5, humid_continental=0.45, cont_hot=0.5, highland_cool=0.6,
                            highland_warm=0.6, subarctic=0.15, tundra=0.15, tropical_monsoon=0.3,
                            tropical_rainforest=0.1),
    "dairy_farm": dict(oceanic=1.0, humid_continental=0.8, cont_hot=0.6, subarctic=0.25,
                       humid_subtropical=0.3, mediterranean=0.25, cold_steppe=0.25,
                       highland_cool=0.4),
    "cotton_plantation": dict(humid_subtropical=0.75, cont_hot=0.25, tropical_savanna=0.7,
                              hot_steppe=0.55, tropical_monsoon=0.35, mediterranean=0.25,
                              highland_warm=0.5, tropical_rainforest=0.1),
    "sugarcane_plantation": dict(tropical_monsoon=1.0, tropical_savanna=0.9, tropical_rainforest=0.8,
                                 humid_subtropical=0.55, highland_warm=0.7, highland_cool=0.2,
                                 hot_steppe=0.3),
    "sugar_beet_farm": dict(humid_continental=1.0, cont_hot=0.5, oceanic=0.9, cold_steppe=0.4,
                            subarctic=0.15, mediterranean=0.2, humid_subtropical=0.2),
    "coffee_plantation": dict(highland_cool=1.0, highland_warm=0.85, tropical_savanna=0.25,
                              tropical_monsoon=0.25, tropical_rainforest=0.2),
    "tea_plantation": dict(humid_subtropical=0.7, highland_cool=0.9, highland_warm=0.8,
                           tropical_monsoon=0.6, tropical_rainforest=0.45, tropical_savanna=0.15,
                           oceanic=0.2),
    "cocoa_plantation": dict(tropical_rainforest=1.0, tropical_monsoon=0.8, tropical_savanna=0.3,
                             highland_warm=0.2),
    "rubber_plantation": dict(tropical_rainforest=1.0, tropical_monsoon=0.75, tropical_savanna=0.2),
    "tobacco_plantation": dict(humid_subtropical=1.0, cont_hot=0.5, humid_continental=0.25,
                               mediterranean=0.7, tropical_savanna=0.6, tropical_monsoon=0.6,
                               tropical_rainforest=0.4, highland_warm=0.7, highland_cool=0.4,
                               hot_steppe=0.35, cold_steppe=0.3, oceanic=0.25),
    "fruit_plantation": dict(tropical_rainforest=0.8, tropical_monsoon=0.9, tropical_savanna=0.6,
                             highland_warm=0.6, mediterranean=1.0, humid_subtropical=0.55),
    "fiber_plantation": dict(tropical_monsoon=0.6, tropical_savanna=0.45, hot_steppe=0.35,
                             humid_subtropical=0.45, humid_continental=0.4, cont_hot=0.3,
                             oceanic=0.25, tropical_rainforest=0.35, highland_cool=0.3,
                             highland_warm=0.4),
    "vineyard": dict(mediterranean=1.0, cold_steppe=0.25, hot_steppe=0.25, humid_subtropical=0.2,
                     cont_hot=0.15, humid_continental=0.1, oceanic=0.4),
    # Fishery: the land's climate stands for the sea's temperature (cold seas are richer).
    "fishery": dict(oceanic=0.9, subarctic=0.9, tundra=0.7, humid_continental=0.7, cont_hot=0.6,
                    ice_cap=0.3, humid_subtropical=0.6, mediterranean=0.5, cold_steppe=0.5,
                    cold_desert=0.5, hot_desert=0.45, hot_steppe=0.45, tropical_savanna=0.4,
                    tropical_monsoon=0.4, tropical_rainforest=0.4, highland_cool=0.6,
                    highland_warm=0.5),
}
# Cash crops: the climate gives the potential, scaled down by this factor, and the regions that
# grew the crop commercially in 1925 get their FLOOR on top. Without it, a crop that needs a
# precise rainfall, soil and market (tea on wet acid hills, coffee on frost-free plateaus) would
# spread over every state of its climate class, which Köppen cannot tell apart.
SCALE = {
    "coffee_plantation": 0.4, "tea_plantation": 0.4, "cocoa_plantation": 0.4,
    "rubber_plantation": 0.4, "cotton_plantation": 0.6, "sugarcane_plantation": 0.6,
    "sugar_beet_farm": 0.6, "tobacco_plantation": 0.7, "fruit_plantation": 0.35,
    "fiber_plantation": 0.6, "vineyard": 0.6,
}
# Irrigation: the share of cropland under irrigation in HYDE (Nile, Indus, Turkestan, Iraq,
# California) lets dry climates grow what rain alone would not. Bonus = weight × irrigated share
# of cropland × share of the state in dry climates (deserts and steppes).
IRRIGATION = {
    "grain_farm": 0.6, "rice_paddy": 0.5, "maize_farm": 0.3, "cotton_plantation": 1.0,
    "sugarcane_plantation": 0.6, "sugar_beet_farm": 0.5, "fruit_plantation": 0.5,
    "vineyard": 0.5, "tobacco_plantation": 0.2,
}
# Latitude limits (state centre, absolute degrees): frost and season length.
LAT_MAX = {
    "cotton_plantation": 42, "sugarcane_plantation": 33, "coffee_plantation": 25,
    "cocoa_plantation": 20, "rubber_plantation": 20, "tea_plantation": 43, "fruit_plantation": 42,
    "vineyard": 52, "rice_paddy": 50,
}
LAT_MIN = {"sugar_beet_farm": 30, "vineyard": 28}

# Regional overrides. FLOOR: the state grew the crop on a commercial scale around 1925, and the
# climate rule alone underrates it (Köppen classes are coarse; the Gold Coast cocoa belt maps as
# savanna, Turkestan's irrigated cotton as desert). The value is the state's suitability at least.
# The 1925 producing regions come from standard agricultural histories (see LAND.md).
FLOOR = {
    "grain_farm": {},
    "rice_paddy": {
        "IND-BENGAL": 0.95, "BGD-DACCA": 0.95, "BGD-RAJSHAHI": 0.95, "BGD-CHITTAGONG": 0.9,
        "IND-BIHAR": 0.85, "IND-ORISSA": 0.9, "IND-MADRAS": 0.85, "IND-ANDHRA": 0.85,
        "IND-ASSAM": 0.85, "IND-MALABAR": 0.8, "IND-CHHATTISGARH": 0.7, "IND-UNITED-PROVINCES": 0.6,
        "MMR-LOWER": 0.95, "MMR-UPPER": 0.7, "THA-CENTRAL-SIAM": 0.95, "THA-ISAN": 0.8,
        "THA-NORTHERN-SIAM": 0.8, "VNM": 0.95, "VNM-TONKIN": 0.95, "VNM-ANNAM": 0.85, "KHM": 0.85,
        "LAO": 0.7, "CHN-JS": 0.9, "CHN-ZJ": 0.9, "CHN-AH": 0.85, "CHN-JX": 0.9, "CHN-HN": 0.95,
        "CHN-HB": 0.9, "CHN-GD": 0.95, "CHN-GX": 0.9, "CHN-FJ": 0.85, "CHN-SC": 0.9, "CHN-GZ": 0.8,
        "CHN-YN": 0.75, "TWN": 0.9, "KOR-JEONNAM": 0.9, "KOR-JEONBUK": 0.9, "KOR-CHUNGNAM": 0.85,
        "KOR-GYEONGNAM": 0.85, "KOR-GYEONGBUK": 0.8, "KOR-GYEONGGI": 0.8, "KOR-CHUNGBUK": 0.75,
        "PRK-HWANGHAE": 0.7, "PRK-PYEONGNAM": 0.6, "PRK-PYEONGBUK": 0.6, "JPN-HOKKAIDO": 0.5,
        "IDN-WEST-JAVA": 0.95, "IDN-MIDDEN-JAVA": 0.95, "IDN-OOST-JAVA": 0.95, "IDN-DJOKJAKARTA": 0.95,
        "IDN-BANTAM": 0.9, "IDN-BATAVIA": 0.9, "IDN-BALI-LOMBOK": 0.9, "PHL-CENTRAL-LUZON": 0.95,
        "PHL-NORTHERN-LUZON": 0.8, "PHL-SOUTHERN-LUZON": 0.8, "PHL-WESTERN-VISAYAS": 0.75,
        "LKA-WESTERN-PROVINCE": 0.7, "MDG": 0.8, "ITA-PIEMONTE": 0.7, "ITA-LOMBARDIA": 0.7,
        "ESP-VALENCIA": 0.7, "EGY-WESTERN-DELTA": 0.7, "EGY-EASTERN-DELTA": 0.7, "USA-LA": 0.8,
        "USA-AR": 0.7, "USA-TX": 0.3, "USA-CA": 0.5, "BRA-RS": 0.6, "IRN-CASPIAN": 0.8, "GUY": 0.7,
        "SUR": 0.6, "SLE": 0.6, "GIN": 0.5, "IRQ-BASRA": 0.5, "PAK-SIND": 0.5,
    },
    "maize_farm": {
        "ARG-LITORAL": 0.9, "ARG-BUENOS-AIRES": 0.85, "ARG-CORDOBA": 0.85, "ROU-MUNTENIA": 0.9,
        "ROU-OLTENIA": 0.9, "ROU-BANAT": 0.85, "ROU-MOLDOVA": 0.75, "MDA": 0.8, "SRB-VOJVODINA": 0.9,
        "SRB": 0.85, "HRV": 0.8, "HUN-GREAT-SOUTHERN-PLAIN": 0.85, "HUN-NORTHERN-GREAT-PLAIN": 0.8,
        "BGR-SEVERNA": 0.8, "ITA-LOMBARDIA": 0.8, "ITA-VENETO": 0.85, "ITA-EMILIA": 0.75,
        "ITA-PIEMONTE": 0.7, "UKR-51": 0.7, "UKR-65": 0.65, "UKR-48": 0.65, "RUS-KUBAN": 0.7,
        "MEX-JAL": 0.9, "MEX-MEX": 0.85, "MEX-GUA": 0.85, "MEX-MIC": 0.85, "MEX-PUE": 0.85,
        "ZAF-TRANSVAAL": 0.85, "ZAF-ORANGE": 0.8, "ZWE": 0.6,
    },
    "livestock_ranch": {
        "ARG-BUENOS-AIRES": 0.95, "ARG-PATAGONIA": 0.8, "URY": 0.95, "BRA-RS": 0.9, "PRY": 0.7,
        "AUS-NSW": 0.95, "AUS-QLD": 0.9, "NZL-SOUTH-ISLAND": 0.9, "NZL-NORTH-ISLAND": 0.9,
        "FLK": 0.9, "MNG": 0.9,
    },
    "dairy_farm": {
        "DNK-JYLLAND": 0.95, "DNK-SJAELLAND": 0.9, "NLD-FR": 0.95, "NLD-ZH": 0.9, "NLD-NH": 0.9,
        "NZL-NORTH-ISLAND": 0.95, "NZL-SOUTH-ISLAND": 0.8, "USA-WI": 0.95, "USA-MN": 0.85,
        "USA-NY": 0.9, "USA-VT": 0.9, "CHE-MITTELLAND": 0.95, "CHE-ZENTRALSCHWEIZ": 0.9,
        "IRL-MUNSTER": 0.9, "IRL-LEINSTER": 0.8, "CAN-ON": 0.8, "CAN-QC": 0.8, "AUS-VIC": 0.8,
        "SWE-GOTALAND": 0.8, "FIN-VAASA": 0.7, "RUS-VLG": 0.7,
    },
    "cotton_plantation": {
        "USA-TX": 0.95, "USA-MS": 0.95, "USA-GA": 0.9, "USA-AL": 0.9, "USA-SC": 0.9, "USA-AR": 0.9,
        "USA-NC": 0.75, "USA-LA": 0.85, "USA-OK": 0.85, "USA-TN": 0.75, "USA-AZ": 0.6, "USA-CA": 0.45,
        "EGY-WESTERN-DELTA": 0.95, "EGY-EASTERN-DELTA": 0.95, "EGY-UPPER": 0.85, "EGY-CAIRO": 0.8,
        "SDN-BLUE-NILE": 0.85, "SDN-KASSALA": 0.5, "IND-BOMBAY": 0.9, "IND-GUJARAT": 0.85,
        "IND-HYDERABAD": 0.85, "IND-CHHATTISGARH": 0.7, "IND-CENTRAL-INDIA": 0.7, "IND-MADRAS": 0.7,
        "IND-PUNJAB": 0.6, "PAK-PUNJAB": 0.75, "PAK-SIND": 0.6, "UZB-FERGANA": 0.95,
        "UZB-TASHKENT": 0.85, "UZB-SAMARKAND": 0.8, "UZB-BUKHARA": 0.8, "UZB-KHOREZM": 0.85,
        "TKM": 0.75, "TJK": 0.75, "KAZ-SOUTH": 0.5, "AZE-AZARBAYCAN": 0.55, "CHN-JS": 0.8,
        "CHN-HB": 0.8, "CHN-SD": 0.7, "CHN-HA": 0.7, "CHN-ZHILI": 0.7, "CHN-SN": 0.6,
        "BRA-PE": 0.7, "BRA-PB": 0.8, "BRA-RN": 0.8, "BRA-CE": 0.7, "BRA-SP": 0.5,
        "PER-CENTRO": 0.8, "PER-NORTE": 0.65, "MEX-COA": 0.7, "MEX-DUR": 0.55, "MEX-BCN": 0.6,
        "UGA-EASTERN-PROVINCE": 0.85, "UGA-BUGANDA": 0.7, "UGA-NORTHERN-PROVINCE": 0.6,
        "NGA-KANO-SOKOTO": 0.5, "TUR-AKDENIZ": 0.75, "ARG-CHACO": 0.7, "IRQ-BAGHDAD": 0.4,
        "IRN-KHORASAN": 0.4,
    },
    "sugarcane_plantation": {
        "CUB-CAMAGUEY": 0.95, "CUB-ORIENTE": 0.95, "CUB-SANTA-CLARA": 0.95, "CUB-MATANZAS": 0.95,
        "CUB-HABANA": 0.9, "CUB-PINAR-DEL-RIO": 0.7, "IDN-OOST-JAVA": 0.95, "IDN-MIDDEN-JAVA": 0.9,
        "IDN-DJOKJAKARTA": 0.9, "IDN-WEST-JAVA": 0.6, "IND-UNITED-PROVINCES": 0.8, "IND-BIHAR": 0.7,
        "IND-PUNJAB": 0.6, "PAK-PUNJAB": 0.5, "USA-HI": 0.95, "USA-LA": 0.8, "PRI": 0.9, "DOM": 0.85,
        "PHL-WESTERN-VISAYAS": 0.9, "PHL-CENTRAL-LUZON": 0.8, "TWN": 0.85, "MUS": 0.95, "FRA-RE": 0.9,
        "BRA-PE": 0.9, "BRA-AL": 0.85, "BRA-SE": 0.7, "BRA-RJ": 0.7, "BRA-BA": 0.6, "BRA-SP": 0.5,
        "PER-NORTE": 0.85, "PER-CENTRO": 0.6, "ZAF-NATAL": 0.85, "AUS-QLD": 0.85,
        "ARG-NOROESTE": 0.8, "JAM": 0.6, "GUY": 0.85, "BRB": 0.9, "TTO": 0.8, "FJI": 0.85,
        "EGY-UPPER": 0.6, "MEX-MOR": 0.6, "MEX-VER": 0.6, "COL-PACIFICO": 0.5, "FRA-GUADELOUPE": 0.85,
        "FRA-MQ": 0.85, "GBR-LEEWARD": 0.7, "GBR-WINDWARD": 0.5, "SUR": 0.5, "VIR": 0.5,
    },
    "sugar_beet_farm": {
        "CZE-CECHY": 0.95, "CZE-MORAVA": 0.95, "DEU-ST": 0.95, "DEU-NI": 0.9, "GER-NIEDERSCHLESIEN": 0.85,
        "FRA-NORD": 0.95, "FRA-PICARDIE": 0.95, "FRA-CHAMPAGNE": 0.85, "FRA-ILE-DE-FRANCE": 0.85,
        "BEL-WHT": 0.9, "BEL-BRABANT": 0.9, "NLD-GR": 0.85, "NLD-ZE": 0.85, "POL-POZNAN": 0.9,
        "POL-POMORZE": 0.85, "POL-WARSZAWA": 0.8, "POL-LUBLIN": 0.8, "UKR-KYIV": 0.95, "UKR-05": 0.95,
        "UKR-71": 0.9, "UKR-53": 0.85, "UKR-63": 0.85, "UKR-59": 0.85, "UKR-68": 0.85,
        "RUS-KRS": 0.85, "RUS-VOR": 0.8, "ITA-EMILIA": 0.8, "ITA-VENETO": 0.75, "AUT-3": 0.85,
        "HUN-WESTERN-TRANSDANUBIA": 0.8, "SWE-GOTALAND": 0.85, "DNK-SJAELLAND": 0.85,
        "USA-CO": 0.85, "USA-MI": 0.8, "USA-UT": 0.8, "USA-ID": 0.75, "USA-NE": 0.75, "USA-CA": 0.6,
        "GBR-EAST-ANGLIA": 0.7,
    },
    "coffee_plantation": {
        "BRA-SP": 0.95, "BRA-MG": 0.9, "BRA-ES": 0.85, "BRA-RJ": 0.8, "BRA-PR": 0.6, "BRA-BA": 0.5,
        "COL-ANTIOQUIA": 0.9, "COL-CUNDINAMARCA": 0.85, "COL-TOLIMA": 0.85, "COL-SANTANDER": 0.85,
        "COL-PACIFICO": 0.8, "VEN": 0.8, "GTM": 0.9, "SLV": 0.95, "CRI": 0.9, "NIC": 0.8, "HND": 0.6,
        "MEX-VER": 0.85, "MEX-CHP": 0.85, "MEX-OAX": 0.7, "HTI": 0.85, "DOM": 0.6, "PRI": 0.75,
        "JAM": 0.5, "CUB-ORIENTE": 0.5, "ECU": 0.6, "IDN-OOST-JAVA": 0.85, "IDN-MIDDEN-JAVA": 0.75,
        "IDN-WEST-JAVA": 0.75, "IDN-ZUID-SUMATRA": 0.75, "IDN-CELEBES": 0.5, "IDN-BALI-LOMBOK": 0.5,
        "IND-MYSORE": 0.85, "IND-MALABAR": 0.7, "IND-MADRAS": 0.5, "KEN": 0.9,
        "UGA-BUGANDA": 0.85, "UGA-EASTERN-PROVINCE": 0.7, "TZA": 0.75, "ETH-SIDAMO": 0.9,
        "ETH-HARAR": 0.85, "ETH-WEST": 0.6, "YEM": 0.85, "AGO": 0.65, "MDG": 0.5, "CIV": 0.4,
        "BEL-RUANDA-URUNDI": 0.5, "COD-ORIENTALE": 0.4, "NCL": 0.4, "VNM-ANNAM": 0.4,
    },
    "tea_plantation": {
        "IND-ASSAM": 0.95, "IND-BENGAL": 0.7, "BGD-RAJSHAHI": 0.7, "BGD-CHITTAGONG": 0.5,
        "IND-MALABAR": 0.8, "IND-MADRAS": 0.7, "IND-MYSORE": 0.4, "IND-KUMAON": 0.5,
        "IND-HILL-STATES": 0.4, "IND-TRIPURA": 0.5, "LKA-CENTRAL-PROVINCE": 0.95,
        "LKA-UVA-PROVINCE": 0.95, "LKA-SABARAGAMUWA-PROVINCE": 0.85, "LKA-SOUTHERN-PROVINCE": 0.6,
        "LKA-WESTERN-PROVINCE": 0.5, "IDN-WEST-JAVA": 0.9, "IDN-MIDDEN-JAVA": 0.6,
        "IDN-OOSTKUST": 0.7, "CHN-FJ": 0.9, "CHN-ZJ": 0.9, "CHN-AH": 0.9, "CHN-JX": 0.85,
        "CHN-HN": 0.85, "CHN-HB": 0.8, "CHN-YN": 0.7, "CHN-SC": 0.7, "CHN-GD": 0.6, "CHN-GX": 0.5,
        "CHN-GZ": 0.5, "TWN": 0.85, "JPN-TOKAI": 0.9, "JPN-KINKI": 0.7, "JPN-MINAMI-KYUSHU": 0.6,
        "JPN-KITA-KYUSHU": 0.6, "GEO-AJARIA": 0.7, "GEO": 0.5, "GEO-ABKHAZIA": 0.5, "MWI": 0.7,
        "KEN": 0.5, "IRN-CASPIAN": 0.4, "VNM-ANNAM": 0.5, "VNM-TONKIN": 0.5, "MMR-SHAN": 0.5,
    },
    "cocoa_plantation": {
        "GHA-COLONY": 0.95, "GHA-ASHANTI": 0.95, "GHA-TOGOLAND": 0.6, "NGA-WESTERN": 0.9,
        "NGA-EASTERN": 0.5, "CIV": 0.7, "CMR": 0.6, "CMR-BRITISH": 0.6, "GNQ": 0.85, "STP": 0.95,
        "TGO": 0.5, "BRA-BA": 0.95, "ECU": 0.9, "VEN": 0.7, "TTO": 0.85, "DOM": 0.7,
        "GBR-WINDWARD": 0.8, "LKA-CENTRAL-PROVINCE": 0.4,
    },
    "rubber_plantation": {
        "MYS-FMS": 0.95, "MYS-UMS": 0.9, "MYS-STRAITS": 0.9, "IDN-OOSTKUST": 0.95, "IDN-ATJEH": 0.7,
        "IDN-ZUID-SUMATRA": 0.8, "IDN-MIDDEN-SUMATRA": 0.7, "IDN-WEST-JAVA": 0.8,
        "IDN-MIDDEN-JAVA": 0.6, "IDN-OOST-JAVA": 0.6, "IDN-WEST-BORNEO": 0.7, "IDN-OOST-BORNEO": 0.6,
        "MYS-SARAWAK": 0.7, "MYS-NORTH-BORNEO": 0.6, "LKA-WESTERN-PROVINCE": 0.85,
        "LKA-SABARAGAMUWA-PROVINCE": 0.9, "LKA-SOUTHERN-PROVINCE": 0.7, "LKA-CENTRAL-PROVINCE": 0.5,
        "VNM": 0.75, "KHM": 0.6, "THA-SOUTHERN-SIAM": 0.7, "MMR-LOWER": 0.5, "LBR": 0.6,
        "IND-MALABAR": 0.6, "BRA-PA": 0.6, "BRA-AM": 0.6, "BRA-AC": 0.6, "BOL": 0.4,
        "PER-ORIENTE": 0.4, "COD-EQUATEUR": 0.4,
    },
    "tobacco_plantation": {
        "USA-KY": 0.95, "USA-NC": 0.95, "USA-VA": 0.95, "USA-SC": 0.8, "USA-TN": 0.85, "USA-CT": 0.8,
        "USA-WI": 0.6, "USA-PA": 0.7, "CUB-PINAR-DEL-RIO": 0.95, "IDN-OOSTKUST": 0.9,
        "IDN-OOST-JAVA": 0.8, "PHL-NORTHERN-LUZON": 0.8, "GRC-MAKEDONIA": 0.9, "GRC-THRAKI": 0.9,
        "TUR-EGE": 0.85, "TUR-DOGU-KARADENIZ": 0.6, "BGR-YUZHNA": 0.9, "BRA-BA": 0.7, "BRA-RS": 0.7,
        "SYR-ALAOUITES": 0.6, "RUS-KRYM": 0.6, "RUS-KUBAN": 0.6, "IND-ANDHRA": 0.7, "IND-BIHAR": 0.6,
        "ZWE": 0.6, "MWI": 0.6, "DZA-ALGER": 0.6, "HUN-GREAT-SOUTHERN-PLAIN": 0.6,
    },
    "fruit_plantation": {
        "HND": 0.9, "CRI": 0.85, "GTM": 0.7, "PAN": 0.8, "JAM": 0.9, "COL-CARIBE": 0.8, "NIC": 0.5,
        "CUB-ORIENTE": 0.6, "ESP-CANARIAS": 0.9, "USA-CA": 0.95, "USA-FL": 0.9, "ESP-VALENCIA": 0.95,
        "ITA-SICILIA": 0.9, "ITA-CALABRIA": 0.7, "GBR-PALESTINE": 0.8, "BRA-RJ": 0.5, "BRA-SP": 0.5,
        "FRA-GUADELOUPE": 0.7, "FRA-MQ": 0.7, "ZAF-CAPE": 0.5,
    },
    "fiber_plantation": {
        # Jute
        "IND-BENGAL": 0.9, "BGD-DACCA": 0.95, "BGD-RAJSHAHI": 0.9, "BGD-CHITTAGONG": 0.6,
        "IND-BIHAR": 0.7, "IND-ASSAM": 0.7, "IND-ORISSA": 0.5,
        # Sisal and henequen
        "TZA": 0.9, "KEN": 0.7, "KEN-COAST": 0.8, "MEX-YUC": 0.95, "MEX-CAM": 0.6, "MOZ": 0.5,
        # Abacá (Manila hemp)
        "PHL-EASTERN-VISAYAS": 0.9, "PHL-SOUTHERN-LUZON": 0.85, "PHL-MINDANAO-AND-SULU": 0.8,
        # Flax and hemp
        "RUS-ORL": 0.6, "RUS-KRS": 0.5, "RUS-SMO": 0.6, "RUS-PSK": 0.7, "RUS-TVE": 0.6, "BLR": 0.6,
        "LTU": 0.6, "LVA": 0.6, "BEL-VWV": 0.7, "BEL-VOV": 0.6, "ITA-CAMPANIA": 0.7, "ITA-EMILIA": 0.7,
        "SRB-VOJVODINA": 0.6, "NZL-NORTH-ISLAND": 0.5,
    },
    "vineyard": {
        "FRA-LANGUEDOC": 0.95, "FRA-GUYENNE": 0.95, "FRA-BOURGOGNE": 0.85, "FRA-CHAMPAGNE": 0.85,
        "FRA-PROVENCE": 0.8, "FRA-RHONE-ALPES": 0.7, "FRA-POITOU-CHARENTES": 0.75,
        "FRA-PAYS-DE-LA-LOIRE": 0.6, "FRA-ALSACE": 0.8, "FRA-GASCOGNE": 0.6, "ITA-PIEMONTE": 0.9,
        "ITA-TOSCANA": 0.9, "ITA-PUGLIE": 0.9, "ITA-SICILIA": 0.85, "ITA-VENETO": 0.8,
        "ITA-CAMPANIA": 0.8, "ITA-EMILIA": 0.7, "ITA-LAZIO": 0.7, "ITA-ABRUZZI-E-MOLISE": 0.7,
        "ESP-CASTILLA-LA-NUEVA": 0.9, "ESP-CATALUNA": 0.9, "ESP-VALENCIA": 0.8,
        "ESP-ANDALUCIA-OCCIDENTAL": 0.8, "ESP-CASTILLA-LA-VIEJA": 0.7, "ESP-ARAGON": 0.6,
        "ESP-NAVARRA": 0.6, "PRT-NORTE": 0.9, "PRT-CENTRO": 0.7, "PRT-LISBOA": 0.7,
        "PRT-MADEIRA": 0.6, "DZA-ORAN": 0.95, "DZA-ALGER": 0.9, "DZA-CONSTANTINE": 0.6, "TUN": 0.5,
        "ROU-MUNTENIA": 0.7, "ROU-MOLDOVA": 0.7, "MDA": 0.7, "HUN-NORTHERN-HUNGARY": 0.7,
        "HUN-GREAT-SOUTHERN-PLAIN": 0.6, "GRC-PELOPONNISOS": 0.8, "SRB": 0.6, "HRV-DALMACIJA": 0.7,
        "BGR-YUZHNA": 0.5, "DEU-RP": 0.8, "DEU-BW": 0.6, "AUT-3": 0.6, "RUS-KRYM": 0.6, "GEO": 0.7,
        "TUR-EGE": 0.6, "GBR-CYPRUS": 0.5, "USA-CA": 0.8, "ARG-CUYO": 0.9, "CHL-CENTRO": 0.9,
        "ZAF-CAPE": 0.7, "AUS-SA": 0.75, "AUS-VIC": 0.5,
    },
    "forestry": {},
    "fishery": {
        "NOR-NORD-NORGE": 1.0, "NOR-VESTLANDET": 1.0, "NOR-TRONDELAG": 0.9, "NOR-SORLANDET": 0.8,
        "ISL": 1.0, "FRO": 1.0, "CAN-NL": 1.0, "CAN-NS": 0.95, "SPM": 0.95, "CAN-NB": 0.8,
        "CAN-PE": 0.8, "CAN-QC": 0.6, "CAN-BC": 0.9, "USA-AK": 0.95, "USA-WA": 0.8, "USA-OR": 0.7,
        "USA-MA": 0.85, "USA-ME": 0.8, "USA-CA": 0.75, "JPN-HOKKAIDO": 1.0, "RUS-KARAFUTO": 0.9,
        "RUS-KAM": 0.95, "RUS-MAGADAN": 0.7, "RUS-PRI": 0.8, "RUS-SAK": 0.75, "RUS-MUR": 0.85,
        "RUS-AST": 0.9, "KAZ-WEST": 0.6, "GBR-EAST-OF-SCOTLAND": 0.95, "GBR-SCOTTISH-HIGHLANDS": 0.9,
        "GBR-YORKSHIRE": 0.9, "GBR-EAST-ANGLIA": 0.85, "GBR-NORTH-EAST-ENGLAND": 0.8,
        "GBR-DEVON-CORNWALL": 0.8, "NLD-ZH": 0.85, "NLD-NH": 0.8, "DEU-HB": 0.8, "DNK-JYLLAND": 0.85,
        "SWE-GOTALAND": 0.8, "PRT-NORTE": 0.85, "PRT-CENTRO": 0.85, "PRT-LISBOA": 0.85,
        "PRT-ALGARVE": 0.85, "ESP-GALICIA": 0.95, "ESP-VASCONGADAS": 0.9, "ESP-ASTURIAS": 0.8,
        "ESP-CANARIAS": 0.8, "FRA-BRETAGNE": 0.95, "FRA-NORMANDIE": 0.75, "FRA-NORD": 0.85,
        "PER-NORTE": 0.75, "PER-CENTRO": 0.75, "PER-SUR": 0.7, "CHL-NORTE": 0.7, "NAM": 0.75,
        "ZAF-CAPE": 0.8, "MAR-RABAT": 0.65, "ESP-SAHARA": 0.65, "MRT": 0.65, "AGO": 0.6,
        "CHN-ZJ": 0.8, "CHN-FJ": 0.7, "CHN-SD": 0.7, "CHN-GD": 0.7, "KOR-GYEONGNAM": 0.8,
        "KOR-JEONNAM": 0.75, "NZL-SOUTH-ISLAND": 0.6, "GRL": 0.5,
    },
}
# Japan led the world's catch in the 1920s: every coastal Japanese state gets this floor.
FISHERY_JAPAN = 0.85
# CAP: the climate fits, but the crop belongs to another part of the world in this period (tea
# outside Asia, Africa and the Caucasus was a curiosity) or the coast is ice-bound most of the
# year (Hudson Bay, the Siberian Arctic). Applied after the floors' regions are excluded.
CAP = {
    "tea_plantation": ("americas_europe_oceania", 0.3),
    # Cocoa was an Atlantic crop: West and Central Africa and the Americas.
    "cocoa_plantation": ("outside_atlantic", 0.03),
    # Farmers of China, Korea, Japan and South-East Asia kept no dairy herds (lactose
    # intolerance, no tradition); the pastoral west of China did.
    "dairy_farm": ("east_asia_farming", 0.1),
    "fishery": ("frozen_coasts", 0.3),
}
EAST_ASIA = ("CHN-", "JPN-", "KOR-", "PRK-", "TWN", "VNM", "KHM", "LAO", "THA-", "MMR-", "MYS-",
             "IDN-", "PHL-", "HKG", "MAC")
PASTORAL_CHINA = {"CHN-NM", "CHN-XZ", "CHN-XJ", "CHN-QH", "CHN-GS"}
FROZEN_COASTS = {"CAN-ON", "CAN-MB", "CAN-NT", "RUS-SA", "RUS-KRASNOYARSK", "RUS-TYUMEN", "RUS-KO"}
MIN_LISTED = 0.05


def clamp01(x):
    return max(0.0, min(1.0, x))


def region(row, name):
    lon, lat, key = float(row["lon"]), float(row["lat"]), row["state"]
    if name == "americas_europe_oceania":
        americas = lon < -30
        europe = lat > 35 and -30 < lon < 40 and not key.startswith("GEO")
        oceania = lon > 110 and lat < -10
        return americas or europe or oceania
    if name == "frozen_coasts":
        return key in FROZEN_COASTS
    if name == "outside_atlantic":
        return lon > 40 or lon < -100
    if name == "east_asia_farming":
        return key.startswith(EAST_ASIA) and key not in PASTORAL_CHINA
    raise ValueError(name)


def zones(row):
    """Climate shares (14 keys) and the zone shares the rules use."""
    lat = abs(float(row["lat"]))
    climate = {c: 0.0 for c in CLIMATES}
    for k, c in KOPPEN_TO_CLIMATE.items():
        climate[c] += float(row[k])
    z = dict(climate)
    z["cont_hot"] = float(row["Dfa"]) + float(row["Dwa"]) + float(row["Dsa"])
    z["humid_continental"] -= z["cont_hot"]
    z["highland_cool"] = z["highland_warm"] = 0.0
    if lat < HIGHLAND_LAT:
        z["highland_cool"] = z["oceanic"] + z["mediterranean"]
        z["highland_warm"] = z["humid_subtropical"]
        z["oceanic"] = z["mediterranean"] = z["humid_subtropical"] = 0.0
    return climate, z


def suitability(row):
    climate, z = zones(row)
    lat = abs(float(row["lat"]))
    area = max(float(row["area_km2"]), 1e-6)
    crop = float(row["cropland_km2"])
    irr = float(row["tot_irri_km2"]) / crop if crop > 0 else 0.0
    rice = float(row["tot_rice_km2"]) / crop if crop > 0 else 0.0
    dry = z["hot_desert"] + z["cold_desert"] + z["hot_steppe"] + z["cold_steppe"]
    grazing_share = float(row["grazing_km2"]) / area
    pasture_share = float(row["pasture_km2"]) / area
    forest_share = float(row["forest_km2"]) / area
    coast = row["coast"] == "1"
    # A state with no farmland at all in HYDE (a rock, a reef, a fortress town) farms nothing.
    farmed = crop + float(row["grazing_km2"]) > 0
    out = {}
    for farm in FARMS:
        if not farmed and farm not in ("forestry", "fishery"):
            out[farm] = 0.0
            continue
        if farm == "forestry":
            s = 1.4 * forest_share
        else:
            s = sum(w * z[k] for k, w in WEIGHTS[farm].items())
            s += IRRIGATION.get(farm, 0.0) * irr * dry
            s *= SCALE.get(farm, 1.0)
            if farm == "rice_paddy":
                # HYDE's rice area: where rice was in fact grown. A dry-season tropical state with
                # almost no rice in HYDE (the Deccan) keeps half its climate fit.
                if rice > 0.02:
                    s = max(s * 0.8, 0.3 + 2.0 * rice)
                elif z["tropical_savanna"] + z["hot_steppe"] > 0.5:
                    s *= 0.5
            if farm == "livestock_ranch":
                s = 0.55 * s + 0.6 * min(1.0, grazing_share)
            if farm == "dairy_farm":
                s = 0.7 * s + 0.3 * min(1.0, 2.0 * pasture_share)
            if farm == "vineyard" and lat > 50.5:
                s *= 0.3
            if farm == "fishery":
                if not coast:
                    out[farm] = 0.0
                    continue
                # Coast length against the state's size: islands and long shores fish more.
                reach = float(row["coast_km"]) / (2.0 * math.sqrt(area))
                s *= clamp01(0.4 + reach)
                if row["state"].startswith("JPN-"):
                    s = max(s, FISHERY_JAPAN)
        if lat > LAT_MAX.get(farm, 90) or lat < LAT_MIN.get(farm, -1):
            s = 0.0
        cap = CAP.get(farm)
        if cap and region(row, cap[0]):
            s = min(s, cap[1])
        s = max(s, FLOOR.get(farm, {}).get(row["state"], 0.0))
        out[farm] = clamp01(s)
    return climate, out


def fmt(x):
    s = f"{x:.3f}".rstrip("0").rstrip(".")
    return s if "." in s else s + ".0"


def rounded_shares(climate):
    """Shares rounded to 3 decimals, tiny ones dropped, the rest rescaled to sum 1."""
    kept = {k: v for k, v in climate.items() if v >= 0.0005}
    total = sum(kept.values())
    shares = {k: round(v / total, 3) for k, v in kept.items()}
    shares = {k: v for k, v in shares.items() if v > 0}
    # Put the rounding remainder on the largest share.
    top = max(shares, key=shares.get)
    shares[top] = round(shares[top] + 1.0 - sum(shares.values()), 3)
    return [(k, shares[k]) for k in CLIMATES if k in shares]


def main():
    rows = list(csv.DictReader(open(STATS, encoding="utf-8")))
    keys = {r["state"] for r in rows}
    for farm, floors in FLOOR.items():
        missing = set(floors) - keys
        assert not missing, f"{farm}: unknown states {sorted(missing)}"
    lines = [
        "// Generated by tools/land/farms.py from tools/land/state_stats.csv; do not edit by hand.",
        "// Sources and method: data/scenario/1925/LAND.md. Format: bf_sim::resources::LandFile.",
        "(",
        "    states: [",
    ]
    results = []
    for r in rows:
        climate, farms = suitability(r)
        area = float(r["area_km2"])
        crop = float(r["cropland_km2"])
        past = float(r["grazing_km2"])
        # Keep the use within the state's area (coastline differences in tiny states).
        if crop + past > area:
            f = area / (crop + past)
            crop, past = crop * f, past * f
        forest = min(float(r["forest_km2"]), area)
        listed = [(k, round(v, 2)) for k, v in farms.items() if v >= MIN_LISTED]
        results.append((r, farms, crop, past, forest))
        clim = ", ".join(f'("{k}", {fmt(v)})' for k, v in rounded_shares(climate))
        farm_s = ", ".join(f'("{k}", {fmt(v)})' for k, v in listed)
        lines += [
            "        (",
            f'            state: "{r["state"]}",',
            f"            climate: [{clim}],",
            f"            cropland_km2: {crop:.1f},",
            f"            pasture_km2: {past:.1f},",
            f"            forest_km2: {forest:.1f},",
            f"            coast: {'true' if r['coast'] == '1' else 'false'},",
            f"            farms: [{farm_s}],",
            "        ),",
        ]
    lines += ["    ],", ")", ""]
    with open(OUT, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    write_check(results)
    print("wrote", OUT, "and", CHECK)


# 1925 producing regions, for the check (states that should rank high).
PRODUCERS = {
    "grain_farm": "USA Plains (KS, ND, NE, OK, MN, WA), Canadian Prairies, Ukraine and the Volga, Punjab and United Provinces, Argentina (BA, Córdoba, Litoral), France, Italy, Australia (NSW, VIC), North China",
    "rice_paddy": "Bengal, Madras, Bihar and Orissa, Lower Burma, Siam, Cochinchina and Tonkin, Yangtze and South China, Java, Japan, Korea, Taiwan, Philippines",
    "maize_farm": "US Corn Belt (IA, IL, NE, MO, IN, OH, KS), Argentina (Litoral, BA, Córdoba), Romania, Yugoslavia, Brazil, Mexico, Italy, Hungary, South Africa, Ukraine south",
    "livestock_ranch": "Australia, Argentina and Uruguay, US West (TX, MT, WY), Kazakhstan, Mongolia, South Africa, Brazil, New Zealand, Patagonia",
    "dairy_farm": "Denmark, Netherlands, New Zealand, Wisconsin and New York, Switzerland, Ireland, Great Britain, Germany, Ontario and Québec, Sweden",
    "cotton_plantation": "US Cotton Belt (TX, MS, GA, AL, SC, AR, OK), India (Bombay Deccan, Berar, Hyderabad, Gujarat, Madras, Punjab), Egypt, China (Jiangsu, Hubei, Shandong, Zhili), Soviet Turkestan (Fergana), Brazil Northeast, Peru, Sudan Gezira, Uganda",
    "sugarcane_plantation": "Cuba, Java, India (United Provinces, Bihar, Punjab), Hawaii, Philippines (Negros), Brazil (Pernambuco), Taiwan, Puerto Rico, Australia (Queensland), Peru, Mauritius, Louisiana",
    "sugar_beet_farm": "Germany (Saxony, Lower Saxony, Silesia), Czechoslovakia, France (Nord, Picardie), Ukraine (Kyiv, Vinnytsia, Kharkiv), Poland, Belgium, Netherlands, USA (CO, MI, UT), Italy (Emilia), Hungary",
    "coffee_plantation": "Brazil (São Paulo, Minas Gerais, Espírito Santo, Rio), Colombia, Venezuela, Central America (El Salvador, Guatemala, Costa Rica), Mexico, Java and Sumatra, Haiti, India (Mysore), Kenya and Uganda, Ethiopia and Yemen",
    "tea_plantation": "China (Fujian, Zhejiang, Anhui, Hunan, Jiangxi), India (Assam, Bengal, Madras, Travancore), Ceylon (Central, Uva), Java and Sumatra, Japan (Shizuoka), Taiwan",
    "cocoa_plantation": "Gold Coast, Brazil (Bahia), Nigeria, Ecuador, São Tomé, Dominican Republic, Trinidad, Venezuela, Côte d'Ivoire, Fernando Po",
    "rubber_plantation": "Malaya, Sumatra, Java, Ceylon, Borneo, Cochinchina, Siam, Brazil (Amazon, wild), Burma, India (Travancore)",
    "tobacco_plantation": "USA (KY, NC, VA, TN, SC), India, China, Dutch East Indies, USSR, Japan, Greece, Turkey, Bulgaria, Philippines, Brazil, Cuba",
    "fruit_plantation": "Bananas: Central America (Honduras, Costa Rica, Guatemala, Panama), Jamaica, Colombia, Canaries; citrus: California, Florida, Valencia, Sicily, Palestine",
    "fiber_plantation": "Jute: Bengal and East Bengal; sisal and henequen: Yucatán, Tanganyika, Kenya; abacá: Philippines; flax and hemp: Russia, Baltic, Belgium, Italy",
    "vineyard": "France (Languedoc, Gironde, Burgundy, Champagne), Italy, Spain (La Mancha, Catalonia), Portugal, Algeria, Argentina (Mendoza), Romania, Hungary, Greece, Chile, California",
    "forestry": "Russia and Siberia, Canada, Finland and Sweden, USA (Pacific NW, South, Lake States), Brazil and the Amazon, Congo Basin, Borneo",
    "fishery": "Japan, Great Britain, Norway, USA (New England, Pacific), Newfoundland and the Maritimes, Iceland, Russia (Caspian, Far East), Spain and Portugal, China, Canada (BC)",
}
# Which land each farm works, for the ranking: cropland, grazing land, forest or coastline.
RANK_BASE = {"livestock_ranch": "grazing", "dairy_farm": "managed pasture", "forestry": "forest", "fishery": "suitability"}


def write_check(results):
    tot_crop = sum(x[2] for x in results)
    tot_past = sum(x[3] for x in results)
    tot_forest = sum(x[4] for x in results)
    out = [
        "# Land check (generated by tools/land/farms.py)",
        "",
        f"World, 870 states: cropland {tot_crop / 1e6:.2f} Mkm², pasture and rangeland {tot_past / 1e6:.2f} Mkm², forest {tot_forest / 1e6:.2f} Mkm².",
        "",
        "Top 10 states by suitability × the land the farm uses (cropland unless noted).",
        "",
    ]
    for farm in FARMS:
        base = RANK_BASE.get(farm, "cropland")

        def weight(x):
            r, farms, crop, past, forest = x
            v = {"cropland": crop, "grazing": past, "managed pasture": float(r["pasture_km2"]),
                 "forest": forest, "suitability": 1.0}[base]
            # As written to land.ron: below MIN_LISTED the farm is not listed.
            suit = farms[farm] if farms[farm] >= MIN_LISTED else 0.0
            return (suit * v, float(r["coast_km"]))

        top = sorted(results, key=weight, reverse=True)[:10]
        names = ", ".join(f"{x[0]['state']} ({x[1][farm]:.2f})" for x in top)
        out += [f"## {farm} (× {base})", "", f"- Top: {names}", f"- 1925 producers: {PRODUCERS[farm]}", ""]
    with open(CHECK, "w", encoding="utf-8") as f:
        f.write("\n".join(out))


if __name__ == "__main__":
    main()
