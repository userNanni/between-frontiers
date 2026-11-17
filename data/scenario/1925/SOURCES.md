# 1925 scenario: country data sources

Data in `countries.ron`, `assets/i18n/*/countries.ftl` and `assets/i18n/*/governments.ftl` is
licensed CC BY-SA 4.0, except `countries.ron`, which is **CC BY-NC-SA 4.0** (non-commercial)
because its urban share comes from HYDE 3.5 (owner's decision of 2026-11-17; the list of
HYDE-derived files is in `LAND.md`, "Licença do HYDE"). This file records where every number
comes from and how it was built.

## Rule: what a tag counts

A tag's `population`, `gdp_per_capita`, `literacy`, `life_expectancy` and `urbanization` are the
**metropole's**: the territory legally part of the state on 1925-01-01. Colonies under direct rule
are provinces of the owner with `colony: true` in `assets/map/provinces.ron` (Victoria 3's
unincorporated states); their people are summed in the tag's `colonies` (population,
population-weighted GDP per capita and social indicators) and **never** in the national figures.
Concretely:

- Which provinces are metropole and which are colonies is data: the `metropoles` section of
  `province_owners.ron`, with every judgement call (Algeria, Alaska and Hawaii, Korea, the Faroes,
  the Crown dependencies) explained in `docs/map-pipeline.pt-BR.md`, "Metrópole e colônias".
- Colonies, protectorates and League mandates **without a tag of their own** are colonies of the
  power that administers them (Nigeria, Malaya, Palestine and Bahrain of GBR; Indochina, Algeria,
  Morocco and Syria of FRA; the Dutch East Indies of NLD; Korea, Taiwan and Kwantung of JAP; the
  Philippines and Puerto Rico of USA; Papua and New Guinea of AUS; South West Africa of SAF; Western
  Samoa of NZL).
- Subjects with a tag (dominions, RAJ, mandates and protectorates in the tag list, MGL, TUV) count
  only their own territory and are **not** added to the overlord. RAJ has no colonies: Burma was a
  province of British India until 1937 and is part of RAJ's own figures.
- `colonies` is `Some` exactly for the 13 owners with colonial provinces (GBR, FRA, BEL, NLD, ESP,
  POR, ITA, DEN, USA, JAP, SAF, AUS, NZL), and metropole plus colonies add up to the empire totals
  of the component tables below; `crates/bf-sim/tests/countries_1925.rs` checks both.
- In the component tables the metropole rows are named under each heading; every other row is a
  colony. Where the province map assigns a territory differently (the Canal Zone is inside Panama
  on the map; Aden town is counted in RAJ by the Census of India), the tables say exactly which pieces were counted where, so the numbers can
  be moved.

## Sources

| Source | Licence | Used for |
|---|---|---|
| Maddison Project Database 2023 (Bolt & van Zanden), `mpd2023_web.xlsx`, Dataverse doi:10.34894/INZBF2 | CC BY 4.0 | GDP per capita (2011 international dollars) and population, mid-year |
| Our World in Data, "Population" (HYDE 3.3, Gapminder v7, UN WPP 2024), `population.csv` | CC BY 4.0 | Population of modern-border components without a Maddison series (Gapminder v7 for 1800–1949) |
| Official censuses of the period (listed per row) | Official statistics; only the published totals are used, as facts | Population of 1925 territories that differ from modern borders |
| Wikidata | CC0 | Capital coordinates (city centre, rounded to 4 decimals) |
| E. Frankema and M. Jerven, African Population Database v4.1 (African Economic History Network) | Licence not checked; only published totals are cited, as facts | Population of Ethiopia |
| Buck (1937), Gamble (1921, 1954), Rawski (1979) and the census figures quoted under "China's literacy" | Published figures cited as facts | Literacy of China |

No non-commercial, GPL or copyrighted-book source was used (see `docs/architecture.pt-BR.md`,
"Fontes de dados"). The raw downloads live in `data/raw/` (gitignored); to rebuild them, fetch
`https://dataverse.nl/api/access/datafile/421302` (Maddison xlsx) and
`https://ourworldindata.org/grapher/population.csv?v=1&csvType=full&useColumnShortNames=true`.

Attribution:

- Bolt, J. and van Zanden, J. L. (2024), "Maddison style estimates of the evolution of the world
  economy: A new 2023 update", *Journal of Economic Surveys*, 1–41. Maddison Project Database,
  version 2023, CC BY 4.0.
- Our World in Data (2024), "Population", based on HYDE (2023), Gapminder (2022) and UN World
  Population Prospects (2024), CC BY 4.0.

## Method

1. **Date.** Maddison and OWID populations are mid-year. The value for 1925-01-01 is the mean of
   mid-1924 and mid-1925. Census-based rows interpolate linearly between the two census dates around
   1925 (decimal years in the table), or project back from one census at the stated annual rate.
2. **GDP per capita** is the Maddison value for the calendar year 1925. When 1925 is missing, it is
   interpolated geometrically between the nearest Maddison years, which are named in the row
   (`MPD interp 1913-1950` means 1913 and 1950 benchmarks, mostly Pamuk 2006 for the Middle East).
3. **Order of preference for population:** census of the 1925 territory when the territory differs
   from modern borders or Maddison lacks the series; then Maddison annual; then OWID annual. Two
   deliberate exceptions: Afghanistan uses the Maddison 1913–1950 interpolation because the OWID
   series jumps from 6.6 M (1913) to 10.2 M (1920) and back to 6.6 M (1930); Ethiopia follows the
   Frankema–Jerven reconstruction (see below) because the OWID series is flat at 17.48 M from 1917
   to 1948, a placeholder rather than an estimate.
4. **Composite tags.** Population is the sum of the components; GDP per capita is the
   population-weighted mean of the components.
5. **Proxies** (all marked `est`, `proxy` or with a three-letter code in the GDP column):
   - `proxy` = 800 $: sub-Saharan colonies without a Maddison series. It is the rounded mean of the
     Maddison values for Nigeria, Malawi, Zambia and Zimbabwe (1925) and Kenya and Uganda (1926), ≈ 790.
   - A three-letter code (`ITA`, `DEU`, `IND`, `CHN`…) means the component takes that country's
     Maddison value (microstates, leased territories, enclaves).
   - `est` means a reasoned estimate stated in the notes below.
6. **Rounding.** `population` is rounded to the thousand (to the hundred below 100 000);
   `gdp_per_capita` to the dollar.

## Judgement calls

### Status

- **EGY — Sovereign.** Egypt has been a formally independent kingdom since 28 February 1922. The four
  points Britain reserved (imperial communications, defence, foreign interests and minorities, the
  Sudan) and the garrison make it a client in practice, but there is no protectorate treaty in force
  in 1925; the British leverage belongs in diplomacy, not in `Status`. The Sudan counts under GBR:
  after the murder of Sir Lee Stack (November 1924) Egyptian troops were expelled and the
  condominium was British-run in fact.
- **DNZ — Sovereign, `free_city`.** A Free City under League of Nations protection with Poland in
  charge of foreign relations and customs; no tag is its overlord in the game's sense (the League is
  not a country), so it is sovereign with the `free_city` government.
- **SAA — Sovereign, `free_city`.** Governed by the League's Governing Commission until the 1935
  plebiscite, with the coal mines owned by France. Same reasoning as Danzig.
- **HAI — Sovereign.** Under United States occupation (1915–1934) with a client president; kept
  sovereign as the tag list requires, the occupation is a matter for scripted events.
- **HEJ** on 1925-01-01 holds Jeddah, Medina and Yanbu under King Ali; Mecca and Taif fell to Nejd in
  October 1924. Capital Jeddah. Population split below.

### Government on 1925-01-01

- **ITA — `parliamentary_monarchy`.** Mussolini is prime minister of a coalition and the Aventine
  opposition still sits outside parliament; the dictatorship begins with the speech of 3 January 1925
  and the laws of 1925–26. The switch to `one_party_state` should be an early scripted event.
- **TUR — `parliamentary_republic`.** The 1924 constitution is an assembly-government republic and the
  opposition Progressive Republican Party is legal from November 1924 until June 1925. The turn to a
  one-party state is a 1925 event.
- **ALB — `military_dictatorship`.** Ahmet Zogu retook Tirana by force on 24 December 1924; the
  republic and his presidency come in January 1925.
- **CHL — `military_dictatorship`.** Junta since the September 1924 coup (another one on 23 January
  1925).
- **CHN — `military_dictatorship`.** Beiyang government under Duan Qirui, installed after Feng
  Yuxiang's Beijing coup of November 1924; the warlords rule the provinces.
- **VEN — `military_dictatorship`** (Gómez). **ESP — `military_dictatorship`** (Primo de Rivera).
- **PRS — `constitutional_monarchy`.** Qajar shah abroad and Reza Khan prime minister and commander,
  but still through the Majlis; the dynasty falls in October 1925.
- **HUN, ROM, YUG, BUL — `constitutional_monarchy`**: the crown (or regent, in Hungary) keeps real
  power over cabinets. **JAP — `constitutional_monarchy`** (Meiji constitution, Taishō party
  cabinets).
- **AFG, NEP, SIA, ETH, BHU, TRJ, HEJ, NEJ, OMA, KUW — `absolute_monarchy`.** Nepal is ruled by the
  hereditary Rana prime ministers with the king as figurehead; the key that fits best is still
  absolute monarchy. Transjordan has no constitution until 1928.
- **TIB, YEM — `theocracy`** (Dalai Lama; Zaydi imamate).
- **AND — `constitutional_monarchy`**: a co-principality under the French head of state and the
  Bishop of Urgell, with the General Council; none of the keys fits exactly.
- **CHE — `parliamentary_republic`**: the directorial system is closest to it among the keys.
- **FIN — `parliamentary_republic`** despite the strong 1919 presidency.

### Capitals

- **NOR — Oslo**: Kristiania was renamed Oslo with effect from 1 January 1925.
- **MGL — Ulan Bator**: Urga was renamed on 29 October 1924.
- **TUV — Khem-Beldyr** (renamed Kyzyl in 1926); coordinates of Kyzyl.
- **BOL — La Paz**, seat of government (Sucre is the constitutional capital).
- **NLD — Amsterdam**, constitutional capital (the government sits in The Hague).
- **SAF — Pretoria**, administrative capital (parliament in Cape Town).
- **AUS — Melbourne**, seat of government until Canberra in 1927.
- **RAJ — Delhi**, capital since 1912 (New Delhi is inaugurated in 1931).
- **LTU — Kaunas**, provisional capital (Vilnius is Polish).
- **BHU — Punakha**, the old capital and winter seat.
- **HEJ — Jeddah** (see Status). **CHN — Peking** in English, the 1925 usage.

### Colours

Iconic colours where tradition exists (GBR red, FRA blue, GER grey, SOV dark red, USA steel blue,
BRA green, ITA green, ESP gold, NLD orange). Dominions, the Raj, the mandates and the British
protectorates use warm reds, roses and salmons related to GBR, and MGL and TUV muted reds and plums
related to SOV. **IRL is green** instead: it shares a land border with GBR and a red tint would not
read apart. Every pair of land neighbours (colonial borders included, e.g. FRA–LBR, GBR–SIA) was
checked for a CIE76 ΔE of at least 25 in Lab; no colour repeats and none is within ΔE 25 of a
parchment background (232, 220, 190).

## Notes on specific estimates

- **SOV**: 1926 census, 17 December 1926, 147 027 915 in the 1926 borders, projected back 1.96
  years at 2.2 %/year. GDP per capita: Maddison "Former USSR" (1991 borders) 1925.
- **GER**: census of 16 June 1925, 62 410 619 without the Saar, projected back at 0.6 %/year.
- **POL**: censuses of 30 September 1921 (27.18 M, including Upper Silesia and the Wilno region) and
  9 December 1931 (32.11 M). GDP per capita: Maddison (modern borders) interpolated 1920–1929; the
  eastern voivodeships were poorer, so this probably overstates it a little.
- **ITA**: Maddison (modern borders) plus about 650 000 people in the parts of the Julian March,
  Fiume, Zara and Lagosta that are now Slovenia and Croatia (rough estimate: Istria and the Slovene
  Littoral ≈ 0.6 M, Fiume and Zara ≈ 0.07 M).
- **GRE**: Maddison minus the Dodecanese (≈ 120 000, Italian). **TUR**: Maddison minus the Sanjak of
  Alexandretta (≈ 220 000, French Syria).
- **CHN**: Maddison China (modern PRC without Taiwan) minus Tibet (1.0 M, estimate), the Kwantung
  Leased Territory (0.85 M, JAP, estimate), Weihaiwei (157 000, GBR) and Guangzhouwan (200 000,
  FRA). All three leases are carved out of the Chinese provinces on the map. GDP per capita
  interpolated between 1913 and 1929.
- **GBR, Weihaiwei**: censuses of 1911 (147 177) and 1921 (154 416), carried on at their 0.48 %/year
  for 3.5 years: 157 000. The lease ran until 1 October 1930.
- **FRA, Guangzhouwan**: linear between the colony's figures of 1911 (189 000) and 1931 (206 000):
  200 000.
- **GBR, Crown dependencies** (metropole since they hold British subjects with the same rights;
  see `docs/map-pipeline.pt-BR.md`): 141 000. Jersey from the censuses of 19 June 1921 (49 701) and
  26 April 1931 (50 462), 49 970 on 1925-01-01; Guernsey with Herm and Jethou from the same censuses
  (38 315 and 40 643), 39 150; Sark 611 and 571, about 600; Alderney about 1 500 (no 1921 or 1931
  total was found; about 1 400 people were evacuated in June 1940); the Isle of Man about 50 000
  residents. The Manx count of 19 June 1921 (60 284) took everyone present in the holiday season,
  visitors included, so it is not used; the April 1931 census counted about 49 300, and its first
  returns reported a fall of only 726 against the comparable 1921 figure, so the resident population
  stayed close to 50 000. GDP per capita and social indicators as the United Kingdom.
- **TIB**: 1.0 M for the territory ruled from Lhasa (Ü-Tsang, western Kham, Ngari); estimates of the
  period range from about 1 M for this area to 3–4 M for all ethnic Tibet. GDP per capita 700 $,
  as Mongolia and Bhutan, a little below Nepal (847).
- **MGL**: linear between the counts of 1918 (647 500) and 1935 (738 200). GDP per capita 700 $
  (Maddison has only 1950: 677). **TUV**: about 60 000 Tuvans and 10 000 Russian settlers; GDP as
  Mongolia.
- **ETH**: 13.3 M (13 311 000) for the Ethiopian Empire without Italian Eritrea. Ethiopia had no
  census before 1984, so every figure before 1950 is a back-projection from the UN benchmark: UN
  *World Population Prospects 2024* puts Ethiopia at 17.67 M in 1950 (Eritrea separate, 0.81 M). The
  value is the mean of 1924 (13.245 M) and 1925 (13.377 M) in the African Population Database of
  E. Frankema and M. Jerven (v4.1, African Economic History Network; method in "Writing History
  Backwards or Sideways", *Economic History Review* 67(4), 2014, 907–931), which carries the WPP
  2024 value back with decade-specific growth rates (12.73 M in 1920, 14.06 M in 1930, 15.23 M in
  1940; about 1.1 %/year from 1925 to 1950). Only these published totals are used, as facts.
  Earlier Gapminder versions agree within about 0.6 M (v3: 12.96 M, v5: 12.72 M for 1925); a
  plain 1 %/year back from 17.67 M gives 13.7 M, the previous value here (14.0 M) came from a rounded
  18 M for 1950.
  - **OWID's 17.5 M is not a 1925 estimate, nor a 1940 one.** The OWID series (Gapminder v7 before
    1950) is 17 476 313 in every year from 1917 to 1948 and drifts *down* from 17.86 M in 1890: it
    inherits Clio Infra's total population, which repeats 18.43 M (the WPP 2010 value for 1950) in
    every decade back to 1850, and joins WPP 2024 in 1950. It implies zero growth for three decades.
  - Contemporary guesses were not counts and ran low: *The Statesman's Year-Book* 1919, "over 8
    millions"; in 1935, 10–15 M in British accounts and "probably twelve millions" from Ethiopian
    officials. They are consistent with about 13 M.
  - GDP per capita 600 $ (Maddison has only 1950: 622, Ethiopia with Eritrea).
- **HEJ / NEJ**: OWID Saudi Arabia split 30/70. Hejaz without Mecca and Taif ≈ 0.73 M; Nejd with
  al-Hasa, Ha'il, Asir and the occupied Mecca ≈ 1.70 M. GDP per capita 1 000 $ (Hejaz, pilgrimage
  trade) and 900 $ (Nejd), near the Maddison 1913 value for Saudi Arabia (956); the 1950 value
  already reflects oil.
- **YEM**: OWID Yemen × 0.80; the other 20 % is Aden (≈ 50 000, inside the Census of India and so in
  RAJ) and the Aden Protectorate (GBR). GDP per capita 900 $ (estimate, pre-oil Arabia).
- **OMA, KUW**: OWID population; GDP per capita 900 $ and 1 200 $ (pearling and trade), estimates.
- **AFG**: 1 000 $ (Maddison 1950: 1 156). **BHU**: 700 $. **NEP**: Maddison interpolation 847 $;
  population from the censuses of 1920 (5.574 M) and 1930 (5.533 M).
- **LUX**: 8 000 $, about 8 % above Belgium (customs union since 1921, steel economy); Maddison has
  only 1950 (13 378, 1.54 × Belgium, after the post-war steel boom).
- **LTU, LAT, EST**: GDP per capita 2 200, 3 400 and 3 200 $ (estimates relative to Finland's 3 711:
  Latvia and Estonia close to it, Lithuania clearly poorer and agrarian). Populations from the
  Lithuanian census of 17 September 1923 (2 028 971, plus Klaipėda's 141 645 in 1925), the Latvian
  census of 10 February 1925 (1 844 805) and the Estonian censuses of 1922 and 1934.
- **NFL**: censuses of 1921 (263 033 incl. Labrador) and 1935 (289 588). Maddison's Canada adds
  Newfoundland only from 1949, so CAN is Maddison unchanged. GDP per capita 3 500 $ (estimate, about
  half of Canada, a fishing economy in debt).
- **HAI, DOM, PRY**: GDP per capita 1 100, 1 300 and 2 000 $ (estimates; Maddison has only 1950).
- **SAA**: between the 1922 estimate (≈ 712 000) and the census of 19 July 1927 (770 030). GDP per
  capita as Germany. **DNZ**: censuses of 1 November 1923 (366 730) and 18 August 1929 (407 517);
  GDP per capita as Germany.
- **EGY**: censuses of 6 March 1917 (12 750 918) and 19 February 1927 (14 217 864); OWID's 13.4 M is
  below both lines.
- **SAF**: censuses of 3 May 1921 (6 928 580) and 5 May 1936 (9 589 898), plus South West Africa
  (OWID Namibia). GDP per capita of South West Africa 1 200 $ (estimate).
- **JAP**: Korea from the census of 1 October 1925 (19 522 945) projected back at 1.5 %/year, GDP per
  capita the Maddison mean of South and North Korea; Karafuto from the 1925 census; Kwantung 0.85 M
  and the South Seas Mandate 56 000 are estimates, with GDP per capita 2 500 $ and 800 $.
- **RAJ**: Maddison "India" before 1947 is undivided British India with the princely states (Census
  of India totals, which include Aden and exclude French and Portuguese India); plus Burma.
- **Small colonies** missing from OWID in 1925 (Réunion, Guadeloupe, Martinique, French Guiana,
  New Caledonia, Curaçao, Macau, Bermuda, the Faroes, Greenland, Guam and
  others) carry rounded figures from the censuses of the 1920s; they add up to about 1.4 M worldwide.
- **Splits of modern countries** (all estimates): Somalia (British Somaliland 340 000, Jubaland
  120 000, which passes from Kenya to Italy on 29 June 1925, rest Italian); Cameroon (southern
  British Cameroons 350 000, rest French); Morocco (Spanish zone 750 000, Ifni 20 000, Tangier
  stays in the French zone until its statute takes effect on 1 June 1925); New Hebrides half French
  and half British.

## Doubts left open

1. Ethiopia's population is a back-projection either way (13.3 M here, after Frankema and Jerven;
   OWID's flat 17.5 M is a placeholder, see the notes). The Frankema–Jerven database licence was
   not checked; only its published totals are cited.
2. The census totals were transcribed from the published reports as cited above and checked for
   consistency against Maddison, but not yet against the original tables one by one.
3. Colonial figures are one aggregate per owner; per-province values (for instance Algeria's
   European settlers against the interior) are left for later. The component tables make that easy.
4. The Rif Republic controlled most of the Spanish zone of Morocco in early 1925; here the zone is
   counted in ESP.
5. The Isle of Man's resident population (about 50 000) is inferred from the 1931 count and the
   reported change since 1921, not read from the 1921 report's table of residents; Alderney's
   1 500 is an estimate.

## Summary

World total: **1 948 031 000** (1 689 041 000 in the tags' own figures, 258 990 000 in colonies). OWID's world for 1 January 1925 is ≈ 1 970 M, but its
modern-border series do not always agree with the censuses: for example OWID India + Pakistan +
Bangladesh is 365 M in 1925, while the Census of India line (1921: 305.7 M, 1931: 338.2 M, both
without Burma) and Maddison give 318 M.

| Tag | Name | Population | GDP pc (2011$) | Colonies population | Colonies GDP pc | Status | Estimated part |
|---|---|---:|---:|---:|---:|---|---|
| GBR | United Kingdom | 45 128 000 | 8 199 | 72 687 000 | 1 079 | Sovereign | population; GDP pc |
| IRL | Irish Free State | 2 995 000 | 4 101 | — | — | Dominion of GBR | population; GDP pc |
| FRA | France | 40 460 000 | 6 641 | 65 960 000 | 1 194 | Sovereign | population; GDP pc |
| BEL | Belgium | 7 743 000 | 7 438 | 11 530 000 | 800 | Sovereign | population; GDP pc |
| NLD | Netherlands | 7 315 000 | 8 019 | 58 003 000 | 1 497 | Sovereign | population; GDP pc |
| LUX | Luxembourg | 282 000 | 8 000 | — | — | Sovereign | GDP pc |
| CHE | Switzerland | 3 903 000 | 8 385 | — | — | Sovereign | population; GDP pc |
| ESP | Spain | 22 180 000 | 3 720 | 923 000 | 1 310 | Sovereign | population; GDP pc |
| POR | Portugal | 6 339 000 | 2 305 | 8 704 000 | 843 | Sovereign | population; GDP pc |
| ITA | Italy | 39 638 000 | 4 580 | 2 651 000 | 1 026 | Sovereign | population; GDP pc |
| SMR | San Marino | 12 900 | 4 580 | — | — | Sovereign | GDP pc |
| GER | Germany | 62 238 000 | 5 630 | — | — | Sovereign | GDP pc |
| SAA | Saar Basin Territory | 740 000 | 5 630 | — | — | Sovereign | population; GDP pc |
| DNZ | Free City of Danzig | 375 000 | 5 630 | — | — | Sovereign | GDP pc |
| AUT | Austria | 6 572 000 | 5 367 | — | — | Sovereign | population; GDP pc |
| HUN | Hungary | 8 266 000 | 3 633 | — | — | Sovereign | population; GDP pc |
| CZE | Czechoslovakia | 14 052 000 | 4 154 | — | — | Sovereign | — |
| POL | Poland | 28 752 000 | 3 202 | — | — | Sovereign | — |
| LTU | Lithuania | 2 197 000 | 2 200 | — | — | Sovereign | GDP pc |
| LAT | Latvia | 1 845 000 | 3 400 | — | — | Sovereign | GDP pc |
| EST | Estonia | 1 111 000 | 3 200 | — | — | Sovereign | GDP pc |
| FIN | Finland | 3 288 000 | 3 711 | — | — | Sovereign | population; GDP pc |
| SWE | Sweden | 6 033 000 | 5 488 | — | — | Sovereign | population; GDP pc |
| NOR | Norway | 2 738 000 | 4 733 | — | — | Sovereign | population; GDP pc |
| DEN | Denmark | 3 429 000 | 6 956 | 14 000 | 800 | Sovereign | population; GDP pc |
| ISL | Iceland | 102 000 | 3 037 | — | — | Sovereign | GDP pc |
| SOV | Soviet Union | 140 889 000 | 1 776 | — | — | Sovereign | — |
| ROM | Romania | 16 710 000 | 1 395 | — | — | Sovereign | GDP pc |
| YUG | Kingdom of the Serbs, Croats and Slovenes | 12 736 000 | 1 758 | — | — | Sovereign | GDP pc |
| BUL | Bulgaria | 5 268 000 | 1 949 | — | — | Sovereign | GDP pc |
| ALB | Albania | 954 000 | 1 428 | — | — | Sovereign | population |
| GRE | Greece | 5 859 000 | 3 411 | — | — | Sovereign | GDP pc |
| TUR | Turkey | 13 794 000 | 1 199 | — | — | Sovereign | GDP pc |
| LIE | Liechtenstein | 12 100 | 5 367 | — | — | Sovereign | GDP pc |
| MCO | Monaco | 24 700 | 6 641 | — | — | Sovereign | GDP pc |
| AND | Andorra | 5 300 | 3 720 | — | — | Sovereign | GDP pc |
| USA | United States | 115 421 000 | 11 150 | 13 179 000 | 2 051 | Sovereign | population; GDP pc |
| CAN | Canada | 9 472 000 | 6 918 | — | — | Dominion of GBR | population; GDP pc |
| NFL | Newfoundland | 270 000 | 3 500 | — | — | Dominion of GBR | GDP pc |
| MEX | Mexico | 15 732 000 | 2 640 | — | — | Sovereign | population; GDP pc |
| GUA | Guatemala | 1 674 000 | 2 327 | — | — | Sovereign | population; GDP pc |
| HON | Honduras | 835 000 | 2 042 | — | — | Sovereign | population; GDP pc |
| SAL | El Salvador | 1 285 000 | 1 476 | — | — | Sovereign | population; GDP pc |
| NIC | Nicaragua | 660 000 | 2 316 | — | — | Sovereign | population; GDP pc |
| CRI | Costa Rica | 455 000 | 2 654 | — | — | Sovereign | population; GDP pc |
| PAN | Panama | 496 000 | 2 514 | — | — | Sovereign | population; GDP pc |
| CUB | Cuba | 3 388 000 | 2 560 | — | — | Sovereign | population; GDP pc |
| HAI | Haiti | 2 254 000 | 1 100 | — | — | Sovereign | population; GDP pc |
| DOM | Dominican Republic | 1 047 000 | 1 300 | — | — | Sovereign | GDP pc |
| COL | Colombia | 6 972 000 | 2 000 | — | — | Sovereign | population; GDP pc |
| VEN | Venezuela | 3 096 000 | 2 289 | — | — | Sovereign | population; GDP pc |
| ECU | Ecuador | 1 858 000 | 1 765 | — | — | Sovereign | population; GDP pc |
| PER | Peru | 5 031 000 | 2 455 | — | — | Sovereign | population; GDP pc |
| BOL | Bolivia | 2 250 000 | 2 144 | — | — | Sovereign | population; GDP pc |
| CHL | Chile | 3 944 000 | 4 903 | — | — | Sovereign | population; GDP pc |
| ARG | Argentina | 10 206 000 | 6 247 | — | — | Sovereign | population; GDP pc |
| URY | Uruguay | 1 516 000 | 4 208 | — | — | Sovereign | population; GDP pc |
| PRY | Paraguay | 776 000 | 2 000 | — | — | Sovereign | population; GDP pc |
| BRA | Brazil | 30 028 000 | 1 298 | — | — | Sovereign | population; GDP pc |
| CHN | China | 477 369 000 | 998 | — | — | Sovereign | — |
| JAP | Japan | 59 104 000 | 3 414 | 24 411 000 | 1 579 | Sovereign | population; GDP pc |
| MGL | Mongolia | 682 000 | 700 | — | — | Satellite of SOV | GDP pc |
| TUV | Tannu Tuva | 70 000 | 700 | — | — | Satellite of SOV | population; GDP pc |
| TIB | Tibet | 1 000 000 | 700 | — | — | Sovereign | population; GDP pc |
| SIA | Siam | 10 794 000 | 1 283 | — | — | Sovereign | population |
| AFG | Afghanistan | 6 424 000 | 1 000 | — | — | Sovereign | GDP pc |
| PRS | Persia | 10 660 000 | 1 634 | — | — | Sovereign | — |
| NEP | Nepal | 5 556 000 | 847 | — | — | Sovereign | — |
| BHU | Bhutan | 258 000 | 700 | — | — | Protectorate of GBR | GDP pc |
| RAJ | British Raj | 332 004 000 | 1 119 | — | — | CrownColony of GBR | population; GDP pc |
| IRQ | Iraq | 3 683 000 | 1 516 | — | — | Mandate of GBR | — |
| TRJ | Transjordan | 209 000 | 1 880 | — | — | Mandate of GBR | — |
| HEJ | Hejaz | 729 000 | 1 000 | — | — | Sovereign | GDP pc |
| NEJ | Nejd | 1 701 000 | 900 | — | — | Sovereign | GDP pc |
| YEM | Yemen | 2 949 000 | 900 | — | — | Sovereign | GDP pc |
| OMA | Muscat and Oman | 426 000 | 900 | — | — | Protectorate of GBR | GDP pc |
| KUW | Kuwait | 119 000 | 1 200 | — | — | Protectorate of GBR | GDP pc |
| EGY | Egypt | 13 904 000 | 1 598 | — | — | Sovereign | — |
| ETH | Ethiopia | 13 311 000 | 600 | — | — | Sovereign | population; GDP pc |
| LBR | Liberia | 612 000 | 1 599 | — | — | Sovereign | GDP pc |
| SAF | Union of South Africa | 7 578 000 | 2 171 | 311 000 | 1 200 | Dominion of GBR | GDP pc |
| AUS | Australia | 5 881 000 | 8 851 | 557 000 | 800 | Dominion of GBR | population; GDP pc |
| NZL | New Zealand | 1 366 000 | 8 435 | 60 000 | 800 | Dominion of GBR | population; GDP pc |

## Components per tag

Population on 1925-01-01 and GDP per capita in 2011 international dollars. `MPD` = Maddison
Project Database 2023; `OWID` = Our World in Data population; `census interp A/B` = linear between
the two censuses; `est` = estimate explained above; `proxy` = 800 $ sub-Saharan proxy; a
three-letter code = that country's Maddison value.

#### GBR: metropole 45 128 000 people at 8 199 $; colonies 72 687 000 at 1 079 $

Metropole: United Kingdom (GB + N. Ireland); Isle of Man, Channel Islands. Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| United Kingdom (GB + N. Ireland) | 44 987 000 | 8 199 | MPD | MPD |
| Isle of Man, Channel Islands | 141 000 | 8 199 | census interp 1921/1931 + est | GBR |
| Malta, Gibraltar | 235 000 | 2 000 | OWID MLT + est | est |
| Cyprus | 326 000 | 1 500 | OWID | est |
| Aden Protectorate | 687 000 | 900 | OWID YEM x 0.20 minus Aden town (in RAJ) | est |
| Palestine (mandate) | 824 000 | 1 851 | census interp 1922/1931 | MPD interp PSE |
| Bahrain, Qatar, Trucial States | 198 000 | 1 000 | OWID | est |
| Ceylon | 5 478 000 | 1 892 | MPD | MPD |
| Maldives | 68 000 | 800 | OWID | est |
| Malaya, Sarawak, N. Borneo | 3 906 000 | 1 733 | MPD | MPD |
| Singapore | 480 000 | 3 255 | MPD | MPD |
| Brunei | 29 000 | 1 500 | OWID | est |
| Hong Kong | 616 000 | 2 814 | MPD | MPD interp 1913-1950 |
| Weihaiwei | 157 000 | 998 | census proj 1911/1921 | CHN |
| British Somaliland | 340 000 | 800 | est | proxy |
| Jubaland (Kenya until Jun 1925) | 120 000 | 800 | est | proxy |
| Nigeria | 24 523 000 | 880 | OWID | MPD |
| Gold Coast + Br. Togoland | 3 379 000 | 1 708 | OWID | MPD |
| Nyasaland | 1 975 000 | 519 | OWID | MPD |
| N. Rhodesia | 1 547 000 | 596 | OWID | MPD |
| S. Rhodesia | 1 673 000 | 706 | OWID | MPD |
| Kenya | 3 805 000 | 1 003 | OWID | MPD 1926 |
| Uganda | 3 793 000 | 1 054 | OWID | MPD 1926 |
| Anglo-Egyptian Sudan (N) | 5 498 000 | 800 | OWID | proxy |
| Anglo-Egyptian Sudan (S) | 2 213 000 | 800 | OWID | proxy |
| Tanganyika + Zanzibar | 5 089 000 | 800 | OWID | proxy |
| Sierra Leone | 1 344 000 | 800 | OWID | proxy |
| Gambia | 205 000 | 800 | OWID | proxy |
| Bechuanaland | 249 000 | 800 | OWID | proxy |
| Basutoland | 351 000 | 800 | OWID | proxy |
| Swaziland | 164 000 | 800 | OWID | proxy |
| British Cameroons (south) | 350 000 | 800 | est | proxy |
| Mauritius, Seychelles, St Helena | 448 000 | 1 500 | OWID + est | est |
| Jamaica | 905 000 | 1 078 | MPD | MPD |
| Trinidad and Tobago | 380 000 | 1 100 | MPD | est |
| Other British Caribbean, Guiana, Honduras | 931 000 | 1 100 | OWID + est (Bermuda, small islands) | est |
| Fiji, Solomons, Gilbert & Ellice, Tonga, 1/2 New Hebrides | 401 000 | 800 | OWID | est |

#### IRL: 2 995 000 people, 4 101 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Irish Free State | 2 995 000 | 4 101 | MPD | MPD |

#### FRA: metropole 40 460 000 people at 6 641 $; colonies 65 960 000 at 1 194 $

Metropole: France (metropolitan). Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| France (metropolitan) | 40 460 000 | 6 641 | MPD | MPD |
| Algeria | 6 122 000 | 1 953 | OWID | MPD interp 1913-1950 |
| Tunisia | 2 448 000 | 1 518 | OWID | MPD interp 1913-1950 |
| Syria (mandate) | 2 460 000 | 2 531 | OWID | MPD interp 1913-1950 |
| Lebanon (mandate) | 836 000 | 2 732 | OWID | MPD interp 1913-1950 |
| Indochina: Vietnam | 18 028 000 | 1 122 | OWID | MPD interp 1913-1950 |
| Indochina: Laos | 960 000 | 1 122 | OWID | VNM |
| Indochina: Cambodia | 2 837 000 | 1 122 | OWID | VNM |
| French Morocco (incl. Tangier) | 5 275 000 | 1 428 | OWID MAR minus Spanish zone and Ifni | MPD interp 1913-1950 |
| Sanjak of Alexandretta | 220 000 | 2 531 | est | SYR |
| Senegal | 1 666 000 | 800 | OWID | proxy |
| French Sudan | 3 225 000 | 800 | OWID | proxy |
| Mauritania | 440 000 | 800 | OWID | proxy |
| Guinea | 1 980 000 | 800 | OWID | proxy |
| Ivory Coast | 1 816 000 | 800 | OWID | proxy |
| Upper Volta | 2 976 000 | 800 | OWID | proxy |
| Niger | 1 750 000 | 800 | OWID | proxy |
| Dahomey | 1 522 000 | 800 | OWID | proxy |
| Togo (mandate) | 939 000 | 800 | OWID | proxy |
| Gabon | 336 000 | 800 | OWID | proxy |
| Middle Congo | 559 000 | 800 | OWID | proxy |
| Ubangi-Shari | 1 014 000 | 800 | OWID | proxy |
| Chad | 1 768 000 | 800 | OWID | proxy |
| Madagascar | 2 642 000 | 800 | OWID | proxy |
| Comoros | 134 000 | 800 | OWID | proxy |
| French Somaliland | 46 000 | 800 | OWID | proxy |
| French Cameroun (mandate) | 2 632 000 | 800 | OWID CMR minus British south | proxy |
| Guangzhouwan | 200 000 | 998 | interp 1911/1931 | CHN |
| French India | 290 000 | 1 113 | est (censuses) | IND |
| Reunion, Guadeloupe, Martinique, Guiana, SPM | 715 000 | 1 300 | est (1921/1926 censuses) | est |
| New Caledonia, Polynesia, Wallis, 1/2 New Hebrides | 124 000 | 1 300 | est + OWID | est |

#### BEL: metropole 7 743 000 people at 7 438 $; colonies 11 530 000 at 800 $

Metropole: Belgium. Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Belgium | 7 743 000 | 7 438 | MPD | MPD |
| Belgian Congo | 8 583 000 | 800 | OWID | proxy |
| Ruanda (mandate) | 1 435 000 | 800 | OWID | proxy |
| Urundi (mandate) | 1 512 000 | 800 | OWID | proxy |

#### NLD: metropole 7 315 000 people at 8 019 $; colonies 58 003 000 at 1 497 $

Metropole: Netherlands. Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Netherlands | 7 315 000 | 8 019 | MPD | MPD |
| Dutch East Indies | 57 810 000 | 1 497 | MPD | MPD |
| Suriname, Curacao | 193 000 | 1 500 | OWID + est | est |

#### LUX: 282 000 people, 8 000 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Luxembourg | 282 000 | 8 000 | OWID | est |

#### CHE: 3 903 000 people, 8 385 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Switzerland | 3 903 000 | 8 385 | MPD | MPD |

#### ESP: metropole 22 180 000 people at 3 720 $; colonies 923 000 at 1 310 $

Metropole: Spain (incl. Canaries, Ceuta, Melilla). Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Spain (incl. Canaries, Ceuta, Melilla) | 22 180 000 | 3 720 | MPD | MPD |
| Spanish Morocco | 750 000 | 1 428 | est | MAR |
| Ifni, Spanish Sahara | 50 000 | 800 | est | proxy |
| Spanish Guinea | 123 000 | 800 | OWID | proxy |

#### POR: metropole 6 339 000 people at 2 305 $; colonies 8 704 000 at 843 $

Metropole: Portugal (incl. Azores, Madeira). Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Portugal (incl. Azores, Madeira) | 6 339 000 | 2 305 | MPD | MPD |
| Angola | 3 002 000 | 800 | OWID | proxy |
| Mozambique | 3 972 000 | 800 | OWID | proxy |
| Portuguese Guinea | 366 000 | 800 | OWID | proxy |
| Cape Verde | 166 000 | 800 | OWID | proxy |
| Sao Tome | 59 000 | 800 | OWID | proxy |
| Portuguese Timor | 399 000 | 800 | OWID | proxy |
| Portuguese India | 580 000 | 1 113 | est (1921 census) | IND |
| Macau | 160 000 | 2 000 | est | est |

#### ITA: metropole 39 638 000 people at 4 580 $; colonies 2 651 000 at 1 026 $

Metropole: Italy (modern borders); Julian March, Fiume, Zara (now SVN/HRV). Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Italy (modern borders) | 38 988 000 | 4 580 | MPD | MPD |
| Julian March, Fiume, Zara (now SVN/HRV) | 650 000 | 4 580 | est | ITA |
| Italian Somaliland | 1 200 000 | 800 | OWID SOM minus British parts | proxy |
| Libya | 714 000 | 1 200 | OWID | est |
| Eritrea | 617 000 | 800 | OWID | proxy |
| Dodecanese | 120 000 | 3 411 | est | GRC |

#### SMR: 13 000 people, 4 580 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| San Marino | 13 000 | 4 580 | OWID | ITA |

#### GER: 62 238 000 people, 5 630 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| German Reich (excl. Saar) | 62 238 000 | 5 630 | census 16 Jun 1925, back 0.6%/yr | MPD |

#### SAA: 740 000 people, 5 630 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Saar Basin | 740 000 | 5 630 | census interp (1922 est, 1927 census) | DEU |

#### DNZ: 375 000 people, 5 630 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Free City of Danzig | 375 000 | 5 630 | census interp 1923/1929 | DEU |

#### AUT: 6 572 000 people, 5 367 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Austria | 6 572 000 | 5 367 | MPD | MPD |

#### HUN: 8 266 000 people, 3 633 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Hungary | 8 266 000 | 3 633 | MPD | MPD |

#### CZE: 14 052 000 people, 4 154 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Czechoslovakia | 14 052 000 | 4 154 | census interp 1921/1930 | MPD CSK |

#### POL: 28 752 000 people, 3 202 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Poland (1925 borders) | 28 752 000 | 3 202 | census interp 1921/1931 | MPD interp 1920-1929 |

#### LTU: 2 197 000 people, 2 200 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Lithuania (incl. Memel) | 2 197 000 | 2 200 | census 1923 + Memel 1925 | est |

#### LAT: 1 845 000 people, 3 400 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Latvia | 1 845 000 | 3 400 | census 10 Feb 1925 | est |

#### EST: 1 111 000 people, 3 200 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Estonia | 1 111 000 | 3 200 | census interp 1922/1934 | est |

#### FIN: 3 288 000 people, 3 711 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Finland | 3 288 000 | 3 711 | MPD | MPD |

#### SWE: 6 033 000 people, 5 488 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Sweden | 6 033 000 | 5 488 | MPD | MPD |

#### NOR: 2 738 000 people, 4 733 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Norway | 2 738 000 | 4 733 | MPD | MPD |

#### DEN: metropole 3 429 000 people at 6 956 $; colonies 14 000 at 800 $

Metropole: Denmark; Faroe Islands. Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Denmark | 3 407 000 | 6 978 | MPD | MPD |
| Faroe Islands | 22 000 | 3 500 | est | est |
| Greenland | 14 000 | 800 | est | est |

#### ISL: 102 000 people, 3 037 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Iceland | 102 000 | 3 037 | OWID | MPD |

#### SOV: 140 889 000 people, 1 776 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| USSR (1926 borders) | 140 889 000 | 1 776 | census 17 Dec 1926, back 2.2%/yr | MPD SUN |

#### ROM: 16 710 000 people, 1 395 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Greater Romania | 16 710 000 | 1 395 | census 29 Dec 1930, back 1.3%/yr | MPD |

#### YUG: 12 736 000 people, 1 758 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Kingdom SCS | 12 736 000 | 1 758 | census interp 1921/1931 | MPD |

#### BUL: 5 268 000 people, 1 949 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Bulgaria | 5 268 000 | 1 949 | census interp 1920/1926 | MPD |

#### ALB: 954 000 people, 1 428 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Albania | 954 000 | 1 428 | MPD | MPD interp 1913-1929 |

#### GRE: 5 859 000 people, 3 411 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Greece (excl. Dodecanese) | 5 859 000 | 3 411 | MPD minus Dodecanese | MPD |

#### TUR: 13 794 000 people, 1 199 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Turkey (excl. Hatay) | 13 794 000 | 1 199 | MPD minus Hatay | MPD |

#### LIE: 12 000 people, 5 367 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Liechtenstein | 12 000 | 5 367 | OWID | AUT |

#### MCO: 25 000 people, 6 641 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Monaco | 25 000 | 6 641 | OWID | FRA |

#### AND: 5 000 people, 3 720 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Andorra | 5 000 | 3 720 | OWID | ESP |

#### USA: metropole 115 421 000 people at 11 150 $; colonies 13 179 000 at 2 051 $

Metropole: United States (48 states, AK, HI). Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| United States (48 states, AK, HI) | 115 421 000 | 11 150 | MPD | MPD |
| Philippines | 11 688 000 | 2 109 | MPD | MPD |
| Puerto Rico | 1 419 000 | 1 600 | MPD | est |
| Guam, Am. Samoa, Virgin Is., Canal Zone | 72 000 | 1 600 | est | est |

#### CAN: 9 472 000 people, 6 918 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Canada (excl. Newfoundland) | 9 472 000 | 6 918 | MPD | MPD |

#### NFL: 270 000 people, 3 500 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Newfoundland and Labrador | 270 000 | 3 500 | census interp 1921/1935 | est |

#### MEX: 15 732 000 people, 2 640 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Mexico | 15 732 000 | 2 640 | MPD | MPD |

#### GUA: 1 674 000 people, 2 327 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Guatemala | 1 674 000 | 2 327 | MPD | MPD |

#### HON: 835 000 people, 2 042 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Honduras | 835 000 | 2 042 | MPD | MPD |

#### SAL: 1 285 000 people, 1 476 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| El Salvador | 1 285 000 | 1 476 | MPD | MPD |

#### NIC: 660 000 people, 2 316 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Nicaragua | 660 000 | 2 316 | MPD | MPD |

#### CRI: 455 000 people, 2 654 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Costa Rica | 455 000 | 2 654 | MPD | MPD |

#### PAN: 496 000 people, 2 514 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Panama | 496 000 | 2 514 | MPD | MPD |

#### CUB: 3 388 000 people, 2 560 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Cuba | 3 388 000 | 2 560 | MPD | MPD |

#### HAI: 2 254 000 people, 1 100 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Haiti | 2 254 000 | 1 100 | MPD | est |

#### DOM: 1 047 000 people, 1 300 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Dominican Republic | 1 047 000 | 1 300 | OWID | est |

#### COL: 6 972 000 people, 2 000 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Colombia | 6 972 000 | 2 000 | MPD | MPD |

#### VEN: 3 096 000 people, 2 289 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Venezuela | 3 096 000 | 2 289 | MPD | MPD |

#### ECU: 1 858 000 people, 1 765 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Ecuador | 1 858 000 | 1 765 | MPD | MPD |

#### PER: 5 031 000 people, 2 455 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Peru | 5 031 000 | 2 455 | MPD | MPD |

#### BOL: 2 250 000 people, 2 144 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Bolivia | 2 250 000 | 2 144 | MPD | MPD |

#### CHL: 3 944 000 people, 4 903 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Chile | 3 944 000 | 4 903 | MPD | MPD |

#### ARG: 10 206 000 people, 6 247 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Argentina | 10 206 000 | 6 247 | MPD | MPD |

#### URY: 1 516 000 people, 4 208 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Uruguay | 1 516 000 | 4 208 | MPD | MPD |

#### PRY: 776 000 people, 2 000 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Paraguay | 776 000 | 2 000 | MPD | est |

#### BRA: 30 028 000 people, 1 298 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Brazil | 30 028 000 | 1 298 | MPD | MPD |

#### CHN: 477 369 000 people, 998 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| China (MPD minus Tibet and leased territories) | 477 369 000 | 998 | MPD minus TIB/leases | MPD interp 1913-1929 |

#### JAP: metropole 59 104 000 people at 3 414 $; colonies 24 411 000 at 1 579 $

Metropole: Japan proper. Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Japan proper | 59 104 000 | 3 414 | MPD | MPD |
| Korea | 19 306 000 | 1 452 | census 1 Oct 1925, back 1.5%/yr | MPD KOR+PRK |
| Taiwan | 4 004 000 | 1 959 | MPD | MPD |
| Karafuto | 195 000 | 2 500 | census 1925 (est. Jan) | est |
| Kwantung Leased Territory | 850 000 | 2 500 | est | est |
| South Seas Mandate | 56 000 | 800 | est | est |

#### MGL: 682 000 people, 700 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Mongolia | 682 000 | 700 | census interp 1918/1935 | est |

#### TUV: 70 000 people, 700 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Tannu Tuva | 70 000 | 700 | est | est |

#### TIB: 1 000 000 people, 700 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Tibet (Lhasa-ruled) | 1 000 000 | 700 | est | est |

#### SIA: 10 794 000 people, 1 283 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Siam | 10 794 000 | 1 283 | MPD | MPD interp 1913-1929 |

#### AFG: 6 424 000 people, 1 000 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Afghanistan | 6 424 000 | 1 000 | MPD interp 1913-1950 | est |

#### PRS: 10 660 000 people, 1 634 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Persia | 10 660 000 | 1 634 | OWID | MPD interp 1913-1950 |

#### NEP: 5 556 000 people, 847 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Nepal | 5 556 000 | 847 | census interp 1920/1930 | MPD interp 1913-1950 |

#### BHU: 258 000 people, 700 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Bhutan | 258 000 | 700 | OWID | est |

#### RAJ: 332 004 000 people, 1 119 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| British India (undivided, incl. princely states, Aden) | 318 300 000 | 1 113 | MPD | MPD |
| Burma | 13 704 000 | 1 263 | MPD | MPD interp 1921-1926 |

#### IRQ: 3 683 000 people, 1 516 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Iraq | 3 683 000 | 1 516 | OWID | MPD interp 1913-1950 |

#### TRJ: 209 000 people, 1 880 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Transjordan | 209 000 | 1 880 | OWID | MPD interp 1913-1950 |

#### HEJ: 729 000 people, 1 000 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Hejaz (Jeddah, Medina; Mecca lost Oct 1924) | 729 000 | 1 000 | OWID SAU x 0.30 | est |

#### NEJ: 1 701 000 people, 900 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Nejd and dependencies (incl. Mecca, Asir) | 1 701 000 | 900 | OWID SAU x 0.70 | est |

#### YEM: 2 949 000 people, 900 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Mutawakkilite Yemen | 2 949 000 | 900 | OWID YEM x 0.80 | est |

#### OMA: 426 000 people, 900 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Muscat and Oman | 426 000 | 900 | OWID | est |

#### KUW: 119 000 people, 1 200 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Kuwait | 119 000 | 1 200 | OWID | est |

#### EGY: 13 904 000 people, 1 598 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Egypt | 13 904 000 | 1 598 | census interp 1917/1927 | MPD interp 1913-1950 |

#### ETH: 13 311 000 people, 600 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Ethiopia | 13 311 000 | 600 | est | est |

#### LBR: 612 000 people, 1 599 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Liberia | 612 000 | 1 599 | OWID | MPD |

#### SAF: metropole 7 578 000 people at 2 171 $; colonies 311 000 at 1 200 $

Metropole: Union of South Africa. Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Union of South Africa | 7 578 000 | 2 171 | census interp 1921/1936 | MPD |
| South West Africa (mandate) | 311 000 | 1 200 | OWID | est |

#### AUS: metropole 5 881 000 people at 8 851 $; colonies 557 000 at 800 $

Metropole: Australia. Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Australia | 5 881 000 | 8 851 | MPD | MPD |
| Papua, New Guinea (mandate), Nauru | 557 000 | 800 | OWID | proxy |

#### NZL: metropole 1 366 000 people at 8 435 $; colonies 60 000 at 800 $

Metropole: New Zealand. Colonies: the other rows.

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| New Zealand | 1 366 000 | 8 435 | MPD | MPD |
| Western Samoa, Cook Is., Niue | 60 000 | 800 | OWID + est | proxy |

## Social indicators: literacy, life expectancy, urbanization

`literacy` (share of adults who can read and write), `life_expectancy` (years at birth) and
`urbanization` (share of the population in urban areas) for 1925, with a short `social_source` per
tag. They follow **the same territory rule as population**: a tag's value is the
population-weighted mean over its metropole components in the tables above, and the `colonies`
value the mean over its colonial components (weights = the component populations listed there).
Every tag and every `colonies` entry has all three values; no `None` was needed.

### Social sources

| Series (Our World in Data) | Download | Origins used and their licences |
|---|---|---|
| "Literacy rate", indicator 1271900, updated 2026-05-12 | `https://ourworldindata.org/grapher/cross-country-literacy-rates.csv?v=1&csvType=full&useColumnShortNames=true` | UNESCO (1953) *Progress of literacy in various countries* and UNESCO (1957) *World illiteracy at mid-century*, CC BY 3.0 IGO; van Zanden et al. (2014) *How Was Life? Global Well-being since 1820*, ch. "Education", OECD, CC BY 4.0 |
| "Life expectancy – Riley; Zijdeman et al.; HMD; UN WPP – Long-run data", indicator 1118466, updated 2025-10-22 | `https://ourworldindata.org/grapher/life-expectancy.csv?v=1&csvType=full&useColumnShortNames=true` | Human Mortality Database (2025), CC BY 4.0; Zijdeman, Ribeira da Silva et al. (2015) *Life Expectancy at Birth (Total)*, Clio Infra / IISH, CC0 1.0; UN *World Population Prospects 2024* (1950 values), CC BY 3.0 IGO |
| "Population share in urban areas" (chart "Share of population living in urbanized areas"), indicator 1269916, updated 2026-06-08 | `https://ourworldindata.org/grapher/urbanization-last-500-years.csv?v=1&csvType=full&useColumnShortNames=true` | HYDE 3.5, "HYDE data for the Global Carbon Budget (2025)", PBL / Utrecht University, doi:10.24416/UU01-F45D44, CC BY 4.0 (see the licence note below) |
| "Population" (already listed above) | as above | Weights when one component spans several modern countries |

Chart pages: `https://ourworldindata.org/grapher/cross-country-literacy-rates`,
`https://ourworldindata.org/grapher/life-expectancy`,
`https://ourworldindata.org/grapher/urbanization-last-500-years`. Downloaded 2026-11-15. OWID's own
processing is CC BY 4.0. Attribution: Our World in Data (2026), "Literacy rate", "Life expectancy"
and "Share of population living in urbanized areas", based on the origins in the table.

Licence notes:

- **HYDE 3.5.** OWID's metadata labels this origin CC BY-NC 4.0, but the dataset's own landing
  page (`https://public.yoda.uu.nl/geo/UU01/F45D44.html`) and its DataCite record both give
  **CC BY 4.0**, as for HYDE 3.3 (doi:10.24416/UU01-AEZZIT). The repository of record is followed
  here; the mismatch is listed under the doubts below.
- **Riley (2005)** is the only origin of the life-expectancy series under restrictive terms (JSTOR).
  OWID uses it only for regional and world rows before 1950. No regional or world row was used as
  input here; the world rows appear only as a comparison in the world check.
- Buringh and van Zanden (literacy before 1800) feed only points before 1870, which are ignored.

### Social method

1. **Mapping.** Each component is mapped to the modern-border OWID entities that cover it (the
   "Series used" column). When it spans several, they are weighted by their OWID population for
   1925; entities without a value for that indicator are dropped and the rest reweighted.
2. **Literacy.**
   - Two points between 1870 and 1950 around 1925: linear interpolation.
   - Only a later point (often the single 1950 UNESCO value): back-projected in log-odds,
     `logit(p1925) = logit(p_y) − 0.80 × (y − 1925) / 25`. 0.80 is the median 1925→1950 log-odds
     gain over the 24 countries with both values (range 0.15, Colombia, to 2.98, Finland).
   - Many 1950 values are midpoints of UNESCO bands (3.0 = "under 5 %", 7.5 = "5–10 %", …), so
     back-projected values below about 0.10 are orders of magnitude, not measurements.
   - The historical figures mix age bases (10+, 7+, all ages in the Census of India); they are
     taken as given and treated as adult literacy.
   - Where the 1950 value belongs to other borders or comes after a mass literacy campaign (Poland,
     USSR, Romania, Albania, Mongolia), the rule would overstate 1925, so these are estimates.
3. **Life expectancy.**
   - Two points between 1870 and 1950 around 1925: linear interpolation.
   - Only a later point before 1950: that value minus 0.41 years per year, the median 1925–1935
     gain over the 27 countries with both and life expectancy under 45.
   - Only the 1950 UN value, at most 55: `0.68 × LE1950`, 0.68 being the median ratio
     LE1925 / LE1950 over the 29 countries with both and LE1925 under 45. Floor 22.8, the lowest
     country value for 1925 in the series (Pakistan); the floor binds for 15 components: Aden
     Protectorate, Nyasaland, southern Sudan, Sierra Leone, Gambia (GBR); French Sudan, Ivory Coast,
     Upper Volta (FRA); Angola, Portuguese Timor (POR); Libya, Eritrea (ITA); AFG, YEM, OMA.
   - Only a 1950 value above 55: the ratio rule does not fit rich or small territories, so they take
     the metropole or a neighbour (named in "Series used") or an estimate.
4. **Urbanization.** HYDE gives every country for 1920 and 1930; 1925 is their mean. National urban
   definitions, as harmonised by HYDE. City-states and enclaves HYDE does not list are estimates.
5. **Tag value** = population-weighted mean over the metropole components; **colonial value** the
   same over the colonial components. Shares rounded to 3 decimals, life
   expectancy to 0.1 year.
6. **Labels.** In `social_source` and the table below an indicator is "series" when at least 90 % of
   the population in question (metropole or colonies) takes interpolated series values, otherwise "back-projected" or "estimate",
   whichever covers more people. In the component table `b` marks a back-projected value and `e` an
   estimate.

### World check

Population-weighted over the 83 tags and the 13 colonial aggregates: **literacy 0.389, life
expectancy 36.8 years, urban share 0.215.** For comparison, OWID's world rows (not used as input):
literacy 31.6 % in 1920 and 32.5 % in 1930 (van Zanden et al.), life expectancy 34.1 in 1913 and
46.4 in 1950 (Riley; UN), HYDE urban share 21.3 % in 1925. Literacy is still about six points above the
OWID world line, down from 0.410 when China was 0.289 (back-projected from UNESCO 1950); which
other tags carry the rest of the gap has not been checked.

### China's literacy

`literacy` for CHN is **0.205** for adults (men about 0.355, women about 0.045; the contract holds
one value). It is an estimate, not the log-odds rule, because the rule's only anchor is wrong for
China:

| Source | Figure | Base |
|---|---|---|
| J. L. Buck, *Land Utilization in China* (Nanking, 1937), p. 373, survey of 1929–1933 in 22 provinces | Men 30 % could read a common letter (45 % had attended school); women 1 % (2 %) | Rural, ages 7+ |
| S. D. Gamble, *Ting Hsien: A North China Rural Community* (1954), p. 185 | About 20 % overall; men about one third, women 3 % | Model county in Hebei, 1930s, ages 6+ |
| S. D. Gamble and J. S. Burgess, *Peking: A Social Survey* (1921), p. 507 | About 70 % | Peking 1918–1919, urban |
| E. S. Rawski, *Education and Popular Literacy in Ch'ing China* (1979), pp. 8–23 | Men 30–45 %, women 2–10 % | Late Qing, "functional" literacy |
| Mass Education Movement (James Yen), 1929 | "Over 200 million" illiterates above age 12 | Contemporary claim |
| PRC, commonly cited for 1949 | About 80 % illiterate | Whole population |
| 1982 census, cohort aged 60+ (born by 1922) | Men 39 %, women 5 % literate | After the 1950s adult campaigns |
| UNESCO, *World Illiteracy at Mid-Century* (1957), via OWID | 47.5 % literate around 1950 (midpoint of a 5-point band) | Ages 15+ |

Rural China (about nine tenths of the people) is taken from Buck and Gamble (men 0.32, women
0.03); the towns from the urban surveys (men 0.65, women 0.20). Weighted, men 0.355 and women 0.045,
0.205 for adults. That sits at the low end of Rawski's late-Qing range, as expected, since Buck's
test (read a common letter) is stricter than her functional literacy, and it agrees with the 1949
figure and with the 1982 census cohort that was adult or near it in the 1920s and 1930s. The
UNESCO 1950 midpoint is rejected as an anchor: it is at the level the 1964 census implies after
the literacy campaigns of the 1950s (about 48 % at ages 13+), so back-projecting from it overstates
1925. Buck's page reference and the urban surveys were read as quoted in Gao, *Risen from Chaos*
(PhD thesis, LSE, 2015, ch. 3) and in S. Chakrabarty, "Women and adult literacy in China" (IGNCA);
the 1982 cohort table as reproduced there. The same estimate replaces China's series in the colonial
rows that borrow it (Weihaiwei, Guangzhouwan, Kwantung).

### Social doubts left open

1. **China's literacy** is now an estimate (0.205, see "China's literacy" above), not the
   back-projection from UNESCO 1950 (0.289). Buck's page reference and the urban surveys were read
   in secondary sources, not in the originals.
2. **Recalled census figures.** The estimates for POL, SOV, ROM, LTU, LAT and EST lean on the
   literacy figures of the 1921–1930 censuses as reported in the secondary literature; they were not
   checked against the census tables.
3. **HYDE licence label** (CC BY 4.0 in the repository, CC BY-NC 4.0 in OWID's metadata).
   Settled on 2026-11-17: the project takes the most restrictive text and marks every
   HYDE-derived file CC BY-NC-SA 4.0 (`LAND.md`, "Licença do HYDE").
4. **Southern Africa.** The UN 1950 values are high (Zimbabwe 49.4, Zambia 46.8), so the ratio rule
   gives 31–34 years for the Rhodesias against 23–30 for most of the continent.

### Social indicators per tag

| Tag | Literacy | Life exp. | Urban | Colonies literacy | Colonies life exp. | Colonies urban | Literacy | Life exp. | Urban |
|---|---:|---:|---:|---:|---:|---:|---|---|---|
| GBR | 0.967 | 58.4 | 0.706 | 0.133 | 27.8 | 0.097 | back-projected | series | series |
| IRL | 0.967 | 57.3 | 0.371 | — | — | — | back-projected | series | series |
| FRA | 0.935 | 54.4 | 0.503 | 0.074 | 28.2 | 0.115 | series | series | series |
| BEL | 0.932 | 57.1 | 0.798 | 0.167 | 26.7 | 0.074 | series | series | series |
| NLD | 0.967 | 63.1 | 0.542 | 0.089 | 29.2 | 0.111 | back-projected | series | series |
| LUX | 0.925 | 54.8 | 0.608 | — | — | — | back-projected | series | series |
| CHE | 0.967 | 59.9 | 0.331 | — | — | — | back-projected | series | series |
| ESP | 0.612 | 47.0 | 0.464 | 0.067 | 26.4 | 0.189 | series | series | series |
| POR | 0.373 | 45.2 | 0.225 | 0.032 | 24.8 | 0.062 | series | back-projected | series |
| ITA | 0.753 | 51.3 | 0.429 | 0.044 | 26.3 | 0.095 | series | series | series |
| SMR | 0.759 | 51.3 | 0.150 | — | — | — | back-projected | series | estimate |
| GER | 0.967 | 57.4 | 0.383 | — | — | — | back-projected | series | series |
| SAA | 0.967 | 57.4 | 0.750 | — | — | — | back-projected | series | estimate |
| DNZ | 0.967 | 57.4 | 0.700 | — | — | — | back-projected | series | estimate |
| AUT | 0.967 | 53.0 | 0.547 | — | — | — | back-projected | series | series |
| HUN | 0.883 | 46.1 | 0.338 | — | — | — | series | series | series |
| CZE | 0.946 | 51.3 | 0.293 | — | — | — | back-projected | series | series |
| POL | 0.700 | 47.1 | 0.327 | — | — | — | estimate | back-projected | series |
| LTU | 0.670 | 50.5 | 0.230 | — | — | — | estimate | series | series |
| LAT | 0.860 | 54.1 | 0.375 | — | — | — | estimate | series | series |
| EST | 0.900 | 52.8 | 0.396 | — | — | — | estimate | series | series |
| FIN | 0.770 | 53.4 | 0.291 | — | — | — | series | series | series |
| SWE | 0.967 | 62.5 | 0.596 | — | — | — | back-projected | series | series |
| NOR | 0.967 | 62.5 | 0.458 | — | — | — | back-projected | series | series |
| DEN | 0.967 | 61.9 | 0.521 | 0.967 | 26.9 | 0.312 | back-projected | series | series |
| ISL | 0.967 | 58.2 | 0.585 | — | — | — | back-projected | series | series |
| SOV | 0.470 | 38.3 | 0.231 | — | — | — | estimate | back-projected | series |
| ROM | 0.520 | 38.8 | 0.142 | — | — | — | estimate | back-projected | series |
| YUG | 0.506 | 40.0 | 0.112 | — | — | — | back-projected | estimate | series |
| BUL | 0.591 | 45.1 | 0.159 | — | — | — | series | series | series |
| ALB | 0.150 | 30.3 | 0.114 | — | — | — | estimate | back-projected | series |
| GRE | 0.543 | 47.0 | 0.138 | — | — | — | series | series | series |
| TUR | 0.077 | 30.5 | 0.198 | — | — | — | back-projected | back-projected | series |
| LIE | 0.967 | 55.6 | 0.100 | — | — | — | back-projected | series | estimate |
| MCO | 0.925 | 54.4 | 1.000 | — | — | — | back-projected | series | estimate |
| AND | 0.612 | 47.0 | 0.350 | — | — | — | series | series | series |
| USA | 0.949 | 58.5 | 0.537 | 0.498 | 35.1 | 0.244 | series | series | series |
| CAN | 0.949 | 59.3 | 0.528 | — | — | — | series | series | series |
| NFL | 0.850 | 52.0 | 0.250 | — | — | — | estimate | estimate | estimate |
| MEX | 0.359 | 32.1 | 0.303 | — | — | — | series | series | series |
| GUA | 0.146 | 25.5 | 0.202 | — | — | — | back-projected | series | series |
| HON | 0.255 | 31.5 | 0.135 | — | — | — | back-projected | series | series |
| SAL | 0.212 | 28.5 | 0.272 | — | — | — | back-projected | series | series |
| NIC | 0.212 | 26.0 | 0.281 | — | — | — | back-projected | series | series |
| CRI | 0.607 | 39.5 | 0.269 | — | — | — | back-projected | series | series |
| PAN | 0.483 | 34.0 | 0.285 | — | — | — | back-projected | back-projected | series |
| CUB | 0.666 | 38.8 | 0.451 | — | — | — | series | series | series |
| HAI | 0.060 | 25.3 | 0.097 | — | — | — | back-projected | back-projected | series |
| DOM | 0.249 | 24.0 | 0.194 | — | — | — | back-projected | back-projected | series |
| COL | 0.488 | 33.0 | 0.311 | — | — | — | series | series | series |
| VEN | 0.332 | 31.5 | 0.351 | — | — | — | back-projected | series | series |
| ECU | 0.378 | 32.9 | 0.220 | — | — | — | back-projected | back-projected | series |
| PER | 0.289 | 30.9 | 0.319 | — | — | — | back-projected | back-projected | series |
| BOL | 0.178 | 33.0 | 0.222 | — | — | — | back-projected | series | series |
| CHL | 0.694 | 34.8 | 0.392 | — | — | — | series | series | series |
| ARG | 0.721 | 50.6 | 0.521 | — | — | — | series | series | series |
| URY | 0.679 | 51.0 | 0.621 | — | — | — | back-projected | series | series |
| PRY | 0.483 | 35.5 | 0.276 | — | — | — | back-projected | series | series |
| BRA | 0.373 | 33.0 | 0.288 | — | — | — | series | series | series |
| CHN | 0.205 | 30.0 | 0.068 | — | — | — | estimate | back-projected | series |
| JAP | 0.946 | 44.4 | 0.261 | 0.216 | 32.4 | 0.189 | back-projected | series | series |
| MGL | 0.050 | 26.2 | 0.124 | — | — | — | estimate | back-projected | series |
| TUV | 0.030 | 26.2 | 0.124 | — | — | — | estimate | back-projected | series |
| TIB | 0.030 | 25.0 | 0.040 | — | — | — | estimate | estimate | estimate |
| SIA | 0.332 | 35.4 | 0.131 | — | — | — | back-projected | back-projected | series |
| AFG | 0.014 | 22.8 | 0.046 | — | — | — | back-projected | back-projected | series |
| PRS | 0.060 | 26.0 | 0.217 | — | — | — | back-projected | back-projected | series |
| NEP | 0.014 | 25.5 | 0.020 | — | — | — | back-projected | back-projected | series |
| BHU | 0.014 | 24.0 | 0.016 | — | — | — | back-projected | back-projected | series |
| RAJ | 0.098 | 27.7 | 0.113 | — | — | — | series | series | series |
| IRQ | 0.060 | 25.4 | 0.280 | — | — | — | back-projected | back-projected | series |
| TRJ | 0.087 | 27.8 | 0.290 | — | — | — | back-projected | back-projected | series |
| HEJ | 0.050 | 27.2 | 0.300 | — | — | — | estimate | back-projected | estimate |
| NEJ | 0.014 | 27.2 | 0.174 | — | — | — | back-projected | back-projected | series |
| YEM | 0.014 | 22.8 | 0.045 | — | — | — | back-projected | back-projected | series |
| OMA | 0.014 | 22.8 | 0.069 | — | — | — | back-projected | back-projected | series |
| KUW | 0.087 | 35.6 | 0.600 | — | — | — | back-projected | series | estimate |
| EGY | 0.130 | 32.2 | 0.264 | — | — | — | series | back-projected | series |
| ETH | 0.014 | 24.1 | 0.037 | — | — | — | back-projected | back-projected | series |
| LBR | 0.035 | 25.8 | 0.092 | — | — | — | back-projected | back-projected | series |
| SAF | 0.249 | 29.3 | 0.227 | 0.115 | 28.5 | 0.107 | back-projected | back-projected | series |
| AUS | 0.967 | 63.2 | 0.528 | 0.037 | 22.9 | 0.008 | back-projected | series | series |
| NZL | 0.967 | 64.0 | 0.609 | 0.759 | 33.5 | 0.088 | back-projected | estimate | series |

### Social indicators per component

| Tag | Component | Population | Literacy | Life exp. | Urban | Series used |
|---|---|---:|---:|---:|---:|---|
| GBR | United Kingdom (GB + N. Ireland) | 44 987 000 | 0.967 b | 58.4 | 0.706 | United Kingdom |
| GBR | Isle of Man, Channel Islands | 141 000 | 0.967 b | 58.4 | 0.706 | United Kingdom |
| GBR | Malta, Gibraltar | 235 000 | 0.378 b | 43.0 e | 0.740 | lit Malta; LE est; urban Malta |
| GBR | Cyprus | 326 000 | 0.428 b | 48.5 | 0.130 | Cyprus |
| GBR | Aden Protectorate | 687 000 | 0.014 b | 22.8 b | 0.045 | Yemen |
| GBR | Palestine (mandate) | 824 000 | 0.250 e | 31.1 b | 0.350 e | lit est; LE Palestine; urban est |
| GBR | Bahrain, Qatar, Trucial States | 198 000 | 0.057 b | 27.8 b | 0.207 | lit Bahrain + Qatar; LE Bahrain + Qatar + United Arab Emirates; urban Bahrain + Qatar + United Arab Emirates |
| GBR | Ceylon | 5 478 000 | 0.428 | 36.1 | 0.126 | Sri Lanka |
| GBR | Maldives | 68 000 | 0.542 b | 24.8 b | 0.070 e | lit Maldives; LE Maldives; urban est |
| GBR | Malaya, Sarawak, N. Borneo | 3 906 000 | 0.212 b | 33.0 b | 0.162 | Malaysia |
| GBR | Singapore | 480 000 | 0.289 b | 36.8 b | 0.633 | Singapore |
| GBR | Brunei | 29 000 | 0.146 b | 34.4 b | 0.213 | Brunei |
| GBR | Hong Kong | 616 000 | 0.378 b | 35.0 e | 0.850 e | lit Hong Kong; LE est; urban est |
| GBR | Weihaiwei | 157 000 | 0.205 e | 30.0 b | 0.068 | lit est (China); LE China; urban China |
| GBR | British Somaliland | 340 000 | 0.014 b | 28.2 b | 0.081 | lit Somaliland; LE Somalia; urban Somalia |
| GBR | Jubaland (Kenya until Jun 1925) | 120 000 | 0.014 b | 28.2 b | 0.081 | lit Somaliland; LE Somalia; urban Somalia |
| GBR | Nigeria | 24 523 000 | 0.060 b | 24.2 b | 0.079 | Nigeria |
| GBR | Gold Coast + Br. Togoland | 3 379 000 | 0.115 b | 30.1 | 0.123 | Ghana |
| GBR | Nyasaland | 1 975 000 | 0.035 b | 22.8 b | 0.027 | Malawi |
| GBR | N. Rhodesia | 1 547 000 | 0.115 b | 31.8 b | 0.093 | Zambia |
| GBR | S. Rhodesia | 1 673 000 | 0.115 b | 33.6 b | 0.085 | Zimbabwe |
| GBR | Kenya | 3 805 000 | 0.115 b | 23.1 b | 0.045 | Kenya |
| GBR | Uganda | 3 793 000 | 0.146 b | 23.1 b | 0.022 | Uganda |
| GBR | Anglo-Egyptian Sudan (N) | 5 498 000 | 0.035 b | 31.1 b | 0.060 | Sudan |
| GBR | Anglo-Egyptian Sudan (S) | 2 213 000 | 0.035 b | 22.8 b | 0.060 | lit Sudan; LE South Sudan; urban Sudan |
| GBR | Tanganyika + Zanzibar | 5 089 000 | 0.035 b | 27.9 b | 0.028 | Tanzania |
| GBR | Sierra Leone | 1 344 000 | 0.035 b | 22.8 b | 0.096 | Sierra Leone |
| GBR | Gambia | 205 000 | 0.035 b | 22.8 b | 0.049 | Gambia |
| GBR | Bechuanaland | 249 000 | 0.115 b | 31.4 b | 0.022 | Botswana |
| GBR | Basutoland | 351 000 | 0.332 b | 30.3 b | 0.011 | Lesotho |
| GBR | Swaziland | 164 000 | 0.087 b | 29.5 b | 0.014 | Eswatini |
| GBR | British Cameroons (south) | 350 000 | 0.035 b | 26.3 b | 0.077 | Cameroon |
| GBR | Mauritius, Seychelles, St Helena | 448 000 | 0.324 b | 31.5 | 0.169 | lit Mauritius + Seychelles + Saint Helena; LE Mauritius; urban Mauritius |
| GBR | Jamaica | 905 000 | 0.542 b | 38.9 | 0.180 | Jamaica |
| GBR | Trinidad and Tobago | 380 000 | 0.607 b | 41.6 | 0.168 | Trinidad and Tobago |
| GBR | Other British Caribbean, Guiana, Honduras | 931 000 | 0.656 b | 35.9 b | 0.210 | lit Guyana + Barbados + Belize + Bahamas + Grenada + Antigua and Barbuda + Bermuda; LE Guyana + Barbados + Belize + Grenada + Saint Lucia + Saint Vincent and the Grenadines + Dominica + Saint Kitts and Nevis; urban Guyana + Barbados + Belize + Bahamas + Grenada + Saint Lucia + Saint Vincent and the Grenadines + Antigua and Barbuda + Dominica + Saint Kitts and Nevis |
| GBR | Fiji, Solomons, Gilbert & Ellice, Tonga, 1/2 New Hebrides | 401 000 | 0.371 b | 32.0 b | 0.102 | lit Fiji + Solomon Islands + Kiribati + Tonga + Vanuatu; LE Fiji + Solomon Islands + Kiribati + Tuvalu + Tonga + Vanuatu; urban Fiji + Solomon Islands + Tonga + Vanuatu |
| IRL | Irish Free State | 2 995 000 | 0.967 b | 57.3 | 0.371 | Ireland |
| FRA | France (metropolitan) | 40 460 000 | 0.935 | 54.4 | 0.503 | France |
| FRA | Algeria | 6 122 000 | 0.087 b | 29.3 | 0.175 | Algeria |
| FRA | Tunisia | 2 448 000 | 0.087 b | 29.3 | 0.252 | Tunisia |
| FRA | Syria (mandate) | 2 460 000 | 0.146 b | 28.9 b | 0.216 | Syria |
| FRA | Lebanon (mandate) | 836 000 | 0.289 b | 38.0 e | 0.236 | lit Lebanon; LE est; urban Lebanon |
| FRA | Indochina: Vietnam | 18 028 000 | 0.087 b | 32.7 b | 0.102 | Vietnam |
| FRA | Indochina: Laos | 960 000 | 0.087 b | 26.5 b | 0.051 | Laos |
| FRA | Indochina: Cambodia | 2 837 000 | 0.087 b | 26.8 b | 0.080 | Cambodia |
| FRA | French Morocco (incl. Tangier) | 5 275 000 | 0.060 b | 26.4 b | 0.208 | Morocco |
| FRA | Sanjak of Alexandretta | 220 000 | 0.146 b | 28.9 b | 0.216 | Syria |
| FRA | Senegal | 1 666 000 | 0.014 b | 23.1 b | 0.134 | Senegal |
| FRA | French Sudan | 3 225 000 | 0.014 b | 22.8 b | 0.061 | lit Senegal; LE Mali; urban Mali |
| FRA | Mauritania | 440 000 | 0.014 b | 26.1 b | 0.024 | lit Senegal; LE Mauritania; urban Mauritania |
| FRA | Guinea | 1 980 000 | 0.014 b | 24.2 b | 0.045 | lit Senegal; LE Guinea; urban Guinea |
| FRA | Ivory Coast | 1 816 000 | 0.014 b | 22.8 b | 0.060 | lit Senegal; LE Cote d'Ivoire; urban Cote d'Ivoire |
| FRA | Upper Volta | 2 976 000 | 0.014 b | 22.8 b | 0.029 | lit Senegal; LE Burkina Faso; urban Burkina Faso |
| FRA | Niger | 1 750 000 | 0.014 b | 26.5 | 0.037 | lit Senegal; LE Niger; urban Niger |
| FRA | Dahomey | 1 522 000 | 0.035 b | 24.1 b | 0.032 | lit Togo; LE Benin; urban Benin |
| FRA | Togo (mandate) | 939 000 | 0.035 b | 28.5 b | 0.033 | Togo |
| FRA | Gabon | 336 000 | 0.014 b | 26.7 b | 0.090 | lit Central African Republic; LE Gabon; urban Gabon |
| FRA | Middle Congo | 559 000 | 0.014 b | 26.9 b | 0.175 | lit Central African Republic; LE Congo; urban Congo |
| FRA | Ubangi-Shari | 1 014 000 | 0.014 b | 24.1 b | 0.115 | Central African Republic |
| FRA | Chad | 1 768 000 | 0.014 b | 24.0 b | 0.035 | lit Central African Republic; LE Chad; urban Chad |
| FRA | Madagascar | 2 642 000 | 0.178 b | 27.0 b | 0.062 | Madagascar |
| FRA | Comoros | 134 000 | 0.115 b | 25.4 b | 0.052 | Comoros |
| FRA | French Somaliland | 46 000 | 0.014 b | 27.7 b | 0.317 | Djibouti |
| FRA | French Cameroun (mandate) | 2 632 000 | 0.035 b | 26.3 b | 0.077 | Cameroon |
| FRA | Guangzhouwan | 200 000 | 0.205 e | 30.0 b | 0.068 | lit est (China); LE China; urban China |
| FRA | French India | 290 000 | 0.086 | 27.6 | 0.116 | India |
| FRA | Reunion, Guadeloupe, Martinique, Guiana, SPM | 715 000 | 0.428 b | 35.3 b | 0.390 | lit Reunion + Guadeloupe + Martinique + French Guiana + Saint Pierre and Miquelon; LE Reunion + Guadeloupe + Martinique + French Guiana; urban Reunion + Guadeloupe + Martinique + French Guiana + Saint Pierre and Miquelon |
| FRA | New Caledonia, Polynesia, Wallis, 1/2 New Hebrides | 124 000 | 0.195 b | 30.3 b | 0.112 | lit New Caledonia + French Polynesia + Vanuatu; LE New Caledonia + French Polynesia + Wallis and Futuna + Vanuatu; urban New Caledonia + Vanuatu |
| BEL | Belgium | 7 743 000 | 0.932 | 57.1 | 0.798 | Belgium |
| BEL | Belgian Congo | 8 583 000 | 0.212 b | 26.0 b | 0.094 | Democratic Republic of Congo |
| BEL | Ruanda (mandate) | 1 435 000 | 0.035 b | 29.4 b | 0.014 | Rwanda |
| BEL | Urundi (mandate) | 1 512 000 | 0.035 b | 27.8 b | 0.015 | lit Rwanda; LE Burundi; urban Burundi |
| NLD | Netherlands | 7 315 000 | 0.967 b | 63.1 | 0.542 | Netherlands |
| NLD | Dutch East Indies | 57 810 000 | 0.087 b | 29.2 b | 0.110 | Indonesia |
| NLD | Suriname, Curacao | 193 000 | 0.542 b | 34.5 b | 0.421 | lit Suriname + Netherlands Antilles; LE Suriname; urban Suriname + Netherlands Antilles |
| LUX | Luxembourg | 282 000 | 0.925 b | 54.8 | 0.608 | Luxembourg |
| CHE | Switzerland | 3 903 000 | 0.967 b | 59.9 | 0.331 | Switzerland |
| ESP | Spain (incl. Canaries, Ceuta, Melilla) | 22 180 000 | 0.612 | 47.0 | 0.464 | Spain |
| ESP | Spanish Morocco | 750 000 | 0.060 b | 26.4 b | 0.208 | Morocco |
| ESP | Ifni, Spanish Sahara | 50 000 | 0.060 b | 23.8 b | 0.179 | lit Morocco; LE Western Sahara; urban Western Sahara |
| ESP | Spanish Guinea | 123 000 | 0.115 b | 27.3 b | 0.076 | Equatorial Guinea |
| POR | Portugal (incl. Azores, Madeira) | 6 339 000 | 0.373 | 45.2 b | 0.225 | Portugal |
| POR | Angola | 3 002 000 | 0.014 b | 22.8 b | 0.055 | Angola |
| POR | Mozambique | 3 972 000 | 0.014 b | 25.4 b | 0.019 | Mozambique |
| POR | Portuguese Guinea | 366 000 | 0.014 b | 24.3 b | 0.077 | Guinea-Bissau |
| POR | Cape Verde | 166 000 | 0.115 b | 30.6 b | 0.093 | Cape Verde |
| POR | Sao Tome | 59 000 | 0.087 b | 26.6 b | 0.108 | Sao Tome and Principe |
| POR | Portuguese Timor | 399 000 | 0.014 b | 22.8 b | 0.079 | East Timor |
| POR | Portuguese India | 580 000 | 0.150 e | 27.6 | 0.116 | lit est; LE India; urban India |
| POR | Macau | 160 000 | 0.378 b | 35.0 e | 0.950 e | lit Macao; LE est; urban est |
| ITA | Italy (modern borders) | 38 988 000 | 0.753 | 51.3 | 0.434 | Italy |
| ITA | Julian March, Fiume, Zara (now SVN/HRV) | 650 000 | 0.753 | 51.3 | 0.125 | lit Italy; LE Italy; urban Slovenia + Croatia |
| ITA | Italian Somaliland | 1 200 000 | 0.014 b | 28.2 b | 0.081 | lit Somaliland; LE Somalia; urban Somalia |
| ITA | Libya | 714 000 | 0.035 b | 22.8 b | 0.143 | Libya |
| ITA | Eritrea | 617 000 | 0.014 b | 22.8 b | 0.057 | Eritrea |
| ITA | Dodecanese | 120 000 | 0.543 | 47.0 | 0.138 | Greece |
| SMR | San Marino | 13 000 | 0.759 b | 51.3 | 0.150 e | lit San Marino; LE Italy; urban est |
| GER | German Reich (excl. Saar) | 62 238 000 | 0.967 b | 57.4 | 0.383 | Germany |
| SAA | Saar Basin | 740 000 | 0.967 b | 57.4 | 0.750 e | lit Germany; LE Germany; urban est |
| DNZ | Free City of Danzig | 375 000 | 0.967 b | 57.4 | 0.700 e | lit Germany; LE Germany; urban est |
| AUT | Austria | 6 572 000 | 0.967 b | 53.0 | 0.547 | Austria |
| HUN | Hungary | 8 266 000 | 0.883 | 46.1 | 0.338 | Hungary |
| CZE | Czechoslovakia | 14 052 000 | 0.946 b | 51.3 | 0.293 | lit Czechoslovakia; LE Czechia + Slovakia; urban Czechia + Slovakia |
| POL | Poland (1925 borders) | 28 752 000 | 0.700 e | 47.1 b | 0.327 | lit est; LE Poland + Lithuania + Belarus + Ukraine; urban Poland |
| LTU | Lithuania (incl. Memel) | 2 197 000 | 0.670 e | 50.5 | 0.230 | lit est; LE Lithuania; urban Lithuania |
| LAT | Latvia | 1 845 000 | 0.860 e | 54.1 | 0.375 | lit est; LE Latvia; urban Latvia |
| EST | Estonia | 1 111 000 | 0.900 e | 52.8 | 0.396 | lit est; LE Estonia; urban Estonia |
| FIN | Finland | 3 288 000 | 0.770 | 53.4 | 0.291 | Finland |
| SWE | Sweden | 6 033 000 | 0.967 b | 62.5 | 0.596 | Sweden |
| NOR | Norway | 2 738 000 | 0.967 b | 62.5 | 0.458 | Norway |
| DEN | Denmark | 3 407 000 | 0.967 b | 61.9 | 0.523 | Denmark |
| DEN | Faroe Islands | 22 000 | 0.967 b | 61.9 | 0.200 e | lit Faroe Islands; LE Denmark; urban est |
| DEN | Greenland | 14 000 | 0.967 b | 26.9 b | 0.312 | Greenland |
| ISL | Iceland | 102 000 | 0.967 b | 58.2 | 0.585 | Iceland |
| SOV | USSR (1926 borders) | 140 889 000 | 0.470 e | 38.3 b | 0.231 | lit est; LE Russia + Ukraine + Belarus + Kazakhstan + Uzbekistan + Azerbaijan + Armenia + Kyrgyzstan + Tajikistan + Turkmenistan; urban Russia + Ukraine + Belarus + Kazakhstan + Uzbekistan + Georgia + Azerbaijan + Armenia + Kyrgyzstan + Tajikistan + Turkmenistan |
| ROM | Greater Romania | 16 710 000 | 0.520 e | 38.8 b | 0.142 | lit est; LE Romania + Moldova; urban Romania + Moldova |
| YUG | Kingdom SCS | 12 736 000 | 0.506 b | 40.0 e | 0.112 | lit Yugoslavia; LE est; urban Croatia + Slovenia + Bosnia and Herzegovina + North Macedonia |
| BUL | Bulgaria | 5 268 000 | 0.591 | 45.1 | 0.159 | Bulgaria |
| ALB | Albania | 954 000 | 0.150 e | 30.3 b | 0.114 | lit est; LE Albania; urban Albania |
| GRE | Greece (excl. Dodecanese) | 5 859 000 | 0.543 | 47.0 | 0.138 | Greece |
| TUR | Turkey (excl. Hatay) | 13 794 000 | 0.077 b | 30.5 b | 0.198 | Turkey |
| LIE | Liechtenstein | 12 000 | 0.967 b | 55.6 | 0.100 e | lit Liechtenstein; LE Switzerland + Austria; urban est |
| MCO | Monaco | 25 000 | 0.925 b | 54.4 | 1.000 e | lit Monaco; LE France; urban est |
| AND | Andorra | 5 000 | 0.612 | 47.0 | 0.350 | lit Spain; LE Spain; urban Andorra |
| USA | United States (48 states, AK, HI) | 115 421 000 | 0.949 | 58.5 | 0.537 | United States |
| USA | Philippines | 11 688 000 | 0.491 | 34.7 b | 0.233 | Philippines |
| USA | Puerto Rico | 1 419 000 | 0.542 b | 38.0 e | 0.324 | lit Puerto Rico; LE est; urban Puerto Rico |
| USA | Guam, Am. Samoa, Virgin Is., Canal Zone | 72 000 | 0.798 b | 40.0 e | 0.425 | lit Guam + American Samoa + United States Virgin Islands; LE est; urban American Samoa + United States Virgin Islands |
| CAN | Canada (excl. Newfoundland) | 9 472 000 | 0.949 | 59.3 | 0.528 | Canada |
| NFL | Newfoundland and Labrador | 270 000 | 0.850 e | 52.0 e | 0.250 e | est |
| MEX | Mexico | 15 732 000 | 0.359 | 32.1 | 0.303 | Mexico |
| GUA | Guatemala | 1 674 000 | 0.146 b | 25.5 | 0.202 | Guatemala |
| HON | Honduras | 835 000 | 0.255 b | 31.5 | 0.135 | Honduras |
| SAL | El Salvador | 1 285 000 | 0.212 b | 28.5 | 0.272 | El Salvador |
| NIC | Nicaragua | 660 000 | 0.212 b | 26.0 | 0.281 | Nicaragua |
| CRI | Costa Rica | 455 000 | 0.607 b | 39.5 | 0.269 | Costa Rica |
| PAN | Panama | 496 000 | 0.483 b | 34.0 b | 0.285 | Panama |
| CUB | Cuba | 3 388 000 | 0.666 | 38.8 | 0.451 | Cuba |
| HAI | Haiti | 2 254 000 | 0.060 b | 25.3 b | 0.097 | Haiti |
| DOM | Dominican Republic | 1 047 000 | 0.249 b | 24.0 b | 0.194 | Dominican Republic |
| COL | Colombia | 6 972 000 | 0.488 | 33.0 | 0.311 | Colombia |
| VEN | Venezuela | 3 096 000 | 0.332 b | 31.5 | 0.351 | Venezuela |
| ECU | Ecuador | 1 858 000 | 0.378 b | 32.9 b | 0.220 | Ecuador |
| PER | Peru | 5 031 000 | 0.289 b | 30.9 b | 0.319 | Peru |
| BOL | Bolivia | 2 250 000 | 0.178 b | 33.0 | 0.222 | Bolivia |
| CHL | Chile | 3 944 000 | 0.694 | 34.8 | 0.392 | Chile |
| ARG | Argentina | 10 206 000 | 0.721 | 50.6 | 0.521 | Argentina |
| URY | Uruguay | 1 516 000 | 0.679 b | 51.0 | 0.621 | Uruguay |
| PRY | Paraguay | 776 000 | 0.483 b | 35.5 | 0.276 | Paraguay |
| BRA | Brazil | 30 028 000 | 0.373 | 33.0 | 0.288 | Brazil |
| CHN | China (MPD minus Tibet and leased territories) | 477 369 000 | 0.205 e | 30.0 b | 0.068 | lit est (Buck); LE China; urban China |
| JAP | Japan proper | 59 104 000 | 0.946 b | 44.4 | 0.261 | Japan |
| JAP | Korea | 19 306 000 | 0.212 b | 31.2 | 0.180 | lit South Korea; LE South Korea + North Korea; urban South Korea + North Korea |
| JAP | Taiwan | 4 004 000 | 0.200 e | 38.0 e | 0.170 | lit est; LE est; urban Taiwan |
| JAP | Karafuto | 195 000 | 0.946 b | 44.4 | 0.300 e | lit Japan; LE Japan; urban est |
| JAP | Kwantung Leased Territory | 850 000 | 0.205 e | 30.0 b | 0.450 e | lit est (China); LE China; urban est |
| JAP | South Seas Mandate | 56 000 | 0.300 e | 35.0 b | 0.100 e | lit est; LE Micronesia (country) + Palau + Marshall Islands; urban est |
| MGL | Mongolia | 682 000 | 0.050 e | 26.2 b | 0.124 | lit est; LE Mongolia; urban Mongolia |
| TUV | Tannu Tuva | 70 000 | 0.030 e | 26.2 b | 0.124 | lit est; LE Mongolia; urban Mongolia |
| TIB | Tibet (Lhasa-ruled) | 1 000 000 | 0.030 e | 25.0 e | 0.040 e | est |
| SIA | Siam | 10 794 000 | 0.332 b | 35.4 b | 0.131 | Thailand |
| AFG | Afghanistan | 6 424 000 | 0.014 b | 22.8 b | 0.046 | Afghanistan |
| PRS | Persia | 10 660 000 | 0.060 b | 26.0 b | 0.217 | Iran |
| NEP | Nepal | 5 556 000 | 0.014 b | 25.5 b | 0.020 | Nepal |
| BHU | Bhutan | 258 000 | 0.014 b | 24.0 b | 0.016 | Bhutan |
| RAJ | British India (undivided, incl. princely states, Aden) | 318 300 000 | 0.086 | 27.6 | 0.112 | lit India; LE India; urban India + Pakistan + Bangladesh |
| RAJ | Burma | 13 704 000 | 0.375 | 30.4 b | 0.129 | Myanmar |
| IRQ | Iraq | 3 683 000 | 0.060 b | 25.4 b | 0.280 | Iraq |
| TRJ | Transjordan | 209 000 | 0.087 b | 27.8 b | 0.290 | Jordan |
| HEJ | Hejaz (Jeddah, Medina; Mecca lost Oct 1924) | 729 000 | 0.050 e | 27.2 b | 0.300 e | lit est; LE Saudi Arabia; urban est |
| NEJ | Nejd and dependencies (incl. Mecca, Asir) | 1 701 000 | 0.014 b | 27.2 b | 0.174 | Saudi Arabia |
| YEM | Mutawakkilite Yemen | 2 949 000 | 0.014 b | 22.8 b | 0.045 | Yemen |
| OMA | Muscat and Oman | 426 000 | 0.014 b | 22.8 b | 0.069 | Oman |
| KUW | Kuwait | 119 000 | 0.087 b | 35.6 | 0.600 e | lit Kuwait; LE Kuwait; urban est |
| EGY | Egypt | 13 904 000 | 0.130 | 32.2 b | 0.264 | Egypt |
| ETH | Ethiopia | 13 311 000 | 0.014 b | 24.1 b | 0.037 | Ethiopia |
| LBR | Liberia | 612 000 | 0.035 b | 25.8 b | 0.092 | Liberia |
| SAF | Union of South Africa | 7 578 000 | 0.249 b | 29.3 b | 0.227 | South Africa |
| SAF | South West Africa (mandate) | 311 000 | 0.115 b | 28.5 b | 0.107 | lit Botswana; LE Namibia; urban Namibia |
| AUS | Australia | 5 881 000 | 0.967 b | 63.2 | 0.528 | Australia |
| AUS | Papua, New Guinea (mandate), Nauru | 557 000 | 0.037 b | 22.9 b | 0.008 | lit Papua New Guinea + Nauru; LE Papua New Guinea; urban Papua New Guinea |
| NZL | New Zealand | 1 366 000 | 0.967 b | 64.0 e | 0.609 | lit New Zealand; LE est; urban New Zealand |
| NZL | Western Samoa, Cook Is., Niue | 60 000 | 0.759 b | 33.5 b | 0.088 | lit Samoa + Cook Islands + Niue; LE Samoa + Cook Islands + Niue; urban Samoa |

### Social estimates

Every value marked `e` above, with its reasoning.

- **GBR, Malta, Gibraltar**, life expectancy 43.0: below Cyprus (48.5): Malta had one of the highest infant mortality rates in Europe in the 1920s.
- **GBR, Palestine (mandate)**, literacy 0.250: Arab majority as Syria and Transjordan (~0.12) plus a Jewish sixth at ~0.85.
- **GBR, Palestine (mandate)**, urban share 0.350: between Syria and Lebanon; Jerusalem, Jaffa, Haifa, Tel Aviv.
- **GBR, Maldives**, urban share 0.070: Male (~5 000) out of ~70 000.
- **GBR, Hong Kong**, life expectancy 35.0: Chinese majority with plague and TB; a few years above China (29).
- **GBR, Hong Kong**, urban share 0.850: a port city with the New Territories; most people in Victoria and Kowloon.
- **FRA, Lebanon (mandate)**, life expectancy 38.0: no series before 1950; above Syria (29) for Beirut and Mount Lebanon, below Cyprus (48.5).
- **POR, Portuguese India**, literacy 0.150: Goa: above British India (0.09) for its Catholic schools, far below Portugal.
- **POR, Macau**, life expectancy 35.0: as Hong Kong.
- **POR, Macau**, urban share 0.950: a city territory.
- **SMR, San Marino**, urban share 0.150: the town of San Marino and Borgo Maggiore; the rest is rural.
- **SAA, Saar Basin**, urban share 0.750: dense mining and steel district; well above the Reich average.
- **DNZ, Free City of Danzig**, urban share 0.700: Danzig and Zoppot hold about two thirds of the people.
- **POL, Poland (1925 borders)**, literacy 0.700: the 1950 value (0.925) is for post-1945 borders; the 1921 and 1931 censuses put literacy at ages 10+ near 0.67 and 0.77.
- **LTU, Lithuania (incl. Memel)**, literacy 0.670: no series; the 1923 census put literacy near two thirds, below Latvia and Estonia.
- **LAT, Latvia**, literacy 0.860: no series; the 1925 census put literacy in the mid-80s.
- **EST, Estonia**, literacy 0.900: no series; Estonia had the highest literacy of the former Russian Empire (~0.9 in the 1922 census).
- **DEN, Faroe Islands**, urban share 0.200: fishing villages; Torshavn ~3 000.
- **SOV, USSR (1926 borders)**, literacy 0.470: the 1926 census counted 0.51 literate at ages 9+; adults were less literate than the schooled young; the 1950 USSR value reflects the literacy campaigns.
- **ROM, Greater Romania**, literacy 0.520: the 1930 census counted ~0.57 literate at ages 7+; 1925 adults a little lower.
- **YUG, Kingdom SCS**, life expectancy 40.0: the successor states have only 1950 values, and the ratio rule skips the richer ones (above 55), which would bias the mean down; between Romania (38.8) and Bulgaria (45.1).
- **ALB, Albania**, literacy 0.150: the 1950 value (0.725) reflects the post-1945 campaign; inter-war estimates put illiteracy above 80 %.
- **LIE, Liechtenstein**, urban share 0.100: villages; Vaduz ~1 500.
- **MCO, Monaco**, urban share 1.000: a city-state.
- **USA, Puerto Rico**, life expectancy 38.0: between Cuba (38.8) and Jamaica (38.9) in 1925; the 1950 UN value (60.4) follows the 1940s health campaigns.
- **USA, Guam, Am. Samoa, Virgin Is., Canal Zone**, life expectancy 40.0: as Puerto Rico and the Canal Zone garrison mix.
- **NFL, Newfoundland and Labrador**, literacy 0.850: below Canada (0.95): outport schooling was thin.
- **NFL, Newfoundland and Labrador**, life expectancy 52.0: below Canada (59.3) and the Maritimes: TB and outport poverty.
- **NFL, Newfoundland and Labrador**, urban share 0.250: St John's (~40 000) and a few towns out of ~270 000.
- **JAP, Taiwan**, literacy 0.200: no series; as Korea under Japanese rule (0.21).
- **JAP, Taiwan**, life expectancy 38.0: no series before 1950; between Korea (31) and Japan (44), with the Japanese public-health campaigns.
- **JAP, Karafuto**, urban share 0.300: settler colony; Toyohara and the ports.
- **JAP, Kwantung Leased Territory**, urban share 0.450: Dairen and Port Arthur hold about half the people.
- **JAP, South Seas Mandate**, literacy 0.300: mission and Japanese primary schools; well below Guam (0.75).
- **JAP, South Seas Mandate**, urban share 0.100: Koror, Garapan and a few ports.
- **MGL, Mongolia**, literacy 0.050: the 1950 value (0.575) reflects the 1940s campaign; in the 1920s reading was largely confined to lamas.
- **TUV, Tannu Tuva**, literacy 0.030: below Mongolia; Tuvan had no script until 1930.
- **TIB, Tibet (Lhasa-ruled)**, literacy 0.030: as Nepal and Bhutan, a little above for monastic reading.
- **TIB, Tibet (Lhasa-ruled)**, life expectancy 25.0: as Nepal and Bhutan in the 1950 x 0.68 rule (~25).
- **TIB, Tibet (Lhasa-ruled)**, urban share 0.040: Lhasa (~25 000), Shigatse, Gyantse out of ~1 M.
- **HEJ, Hejaz (Jeddah, Medina; Mecca lost Oct 1924)**, literacy 0.050: Jeddah and Medina towns above Arabia's 0.014.
- **HEJ, Hejaz (Jeddah, Medina; Mecca lost Oct 1924)**, urban share 0.300: Jeddah, Medina, Yanbu are a large part of the Hejaz left to King Ali.
- **KUW, Kuwait**, urban share 0.600: Kuwait town held most of the settled people.
- **NZL, New Zealand**, life expectancy 64.0: no series before 1948; New Zealand had the longest lives of the period, a little above Australia (63.2).
- **CHN, China**, literacy 0.205: Buck's 1929–1933 survey and the urban surveys; see "China's literacy".
- **GBR, Weihaiwei; FRA, Guangzhouwan; JAP, Kwantung Leased Territory**, literacy 0.205: as China.
