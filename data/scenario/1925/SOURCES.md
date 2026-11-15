# 1925 scenario: country data sources

Data in `countries.ron`, `assets/i18n/*/countries.ftl` and `assets/i18n/*/governments.ftl` is
licensed CC BY-SA 4.0. This file records where every number comes from and how it was built.

## Rule: what a tag counts

`population` and `gdp_per_capita` cover **all territory the tag rules directly on 1925-01-01**,
colonies included, because colonies under direct rule are provinces of the owner in the game
(`Status` in `crates/bf-sim/src/scenario.rs`). Concretely:

- Colonies, protectorates and League mandates **without a tag of their own** count under the power
  that administers them (Nigeria, Malaya, Palestine and Bahrain under GBR; Indochina, Morocco and
  Syria under FRA; the Dutch East Indies under NLD; Korea, Taiwan and Kwantung under JAP; the
  Philippines under USA; Papua and New Guinea under AUS; South West Africa under SAF; Western Samoa
  under NZL).
- Subjects with a tag (dominions, RAJ, mandates and protectorates in the tag list, MGL, TUV) count
  only their own territory; they are **not** added to the overlord.
- Consequence: the GDP per capita of colonial powers is the average over the whole empire, far below
  the metropole (GBR 3 806 overall against 8 199 for the United Kingdom alone; NLD 2 227 against
  8 019). The metropole value is the first row of each tag in the component tables below.

Where the province map ends up assigning a territory differently, the component tables say exactly
which pieces were counted where, so the numbers can be moved.

## Sources

| Source | Licence | Used for |
|---|---|---|
| Maddison Project Database 2023 (Bolt & van Zanden), `mpd2023_web.xlsx`, Dataverse doi:10.34894/INZBF2 | CC BY 4.0 | GDP per capita (2011 international dollars) and population, mid-year |
| Our World in Data, "Population" (HYDE 3.3, Gapminder v7, UN WPP 2024), `population.csv` | CC BY 4.0 | Population of modern-border components without a Maddison series (Gapminder v7 for 1800–1949) |
| Official censuses of the period (listed per row) | Official statistics; only the published totals are used, as facts | Population of 1925 territories that differ from modern borders |
| Wikidata | CC0 | Capital coordinates (city centre, rounded to 4 decimals) |

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
   series jumps from 6.6 M (1913) to 10.2 M (1920) and back to 6.6 M (1930); Ethiopia is an estimate
   (see below) because the OWID series is flat at 17.48 M from 1920 to 1940, a placeholder rather
   than an estimate.
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
- **CHN**: Maddison China (modern PRC without Taiwan) minus Tibet (1.0 M), the Kwantung Leased
  Territory (0.85 M, JAP), Weihaiwei (0.15 M, GBR) and Guangzhouwan (0.21 M, FRA), all estimates.
  GDP per capita interpolated between 1913 and 1929.
- **TIB**: 1.0 M for the territory ruled from Lhasa (Ü-Tsang, western Kham, Ngari); estimates of the
  period range from about 1 M for this area to 3–4 M for all ethnic Tibet. GDP per capita 700 $,
  as Mongolia and Bhutan, a little below Nepal (847).
- **MGL**: linear between the counts of 1918 (647 500) and 1935 (738 200). GDP per capita 700 $
  (Maddison has only 1950: 677). **TUV**: about 60 000 Tuvans and 10 000 Russian settlers; GDP as
  Mongolia.
- **ETH**: 14.0 M. The UN puts Ethiopia (without Eritrea) at about 18 M in 1950; 1 %/year back to
  1925 gives 14 M. The OWID series is flat at 17.5 M (see Method 3); League of Nations era guesses
  were 6–12 M. GDP per capita 600 $ (Maddison 1950: 622).
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
  New Caledonia, Curaçao, Macau, Bermuda, the Channel Islands, the Faroes, Greenland, Guam and
  others) carry rounded figures from the censuses of the 1920s; they add up to about 1.4 M worldwide.
- **Splits of modern countries** (all estimates): Somalia (British Somaliland 340 000, Jubaland
  120 000, which passes from Kenya to Italy on 29 June 1925, rest Italian); Cameroon (southern
  British Cameroons 350 000, rest French); Morocco (Spanish zone 750 000, Ifni 20 000, Tangier
  stays in the French zone until its statute takes effect on 1 June 1925); New Hebrides half French
  and half British.

## Doubts left open

1. Ethiopia's population (14 M here, 17.5 M in OWID).
2. The census totals were transcribed from the published reports as cited above and checked for
   consistency against Maddison, but not yet against the original tables one by one.
3. Whether the game wants colonial GDP averaged into the empire's GDP per capita (the rule applied
   here) or will keep separate per-province values later; the component tables make either easy.
4. The Rif Republic controlled most of the Spanish zone of Morocco in early 1925; here the zone is
   counted in ESP.

## Summary

World total: **1 948 720 000**. OWID's world for 1 January 1925 is ≈ 1 970 M, but its
modern-border series do not always agree with the censuses: for example OWID India + Pakistan +
Bangladesh is 365 M in 1925, while the Census of India line (1921: 305.7 M, 1931: 338.2 M, both
without Burma) and Maddison give 318 M.

| Tag | Name | Population | GDP pc (2011$) | Status | Estimated part |
|---|---|---:|---:|---|---|
| GBR | United Kingdom | 117 808 000 | 3 806 | Sovereign | population; GDP pc |
| IRL | Irish Free State | 2 995 000 | 4 101 | Dominion of GBR | population; GDP pc |
| FRA | France | 106 431 000 | 3 264 | Sovereign | population; GDP pc |
| BEL | Belgium | 19 273 000 | 3 467 | Sovereign | population; GDP pc |
| NLD | Netherlands | 65 319 000 | 2 227 | Sovereign | population; GDP pc |
| LUX | Luxembourg | 282 000 | 8 000 | Sovereign | GDP pc |
| CHE | Switzerland | 3 903 000 | 8 385 | Sovereign | population; GDP pc |
| ESP | Spain | 23 104 000 | 3 624 | Sovereign | population; GDP pc |
| POR | Portugal | 15 042 000 | 1 459 | Sovereign | population; GDP pc |
| ITA | Italy | 42 288 000 | 4 357 | Sovereign | population; GDP pc |
| SMR | San Marino | 12 900 | 4 580 | Sovereign | GDP pc |
| GER | Germany | 62 238 000 | 5 630 | Sovereign | GDP pc |
| SAA | Saar Basin Territory | 740 000 | 5 630 | Sovereign | population; GDP pc |
| DNZ | Free City of Danzig | 375 000 | 5 630 | Sovereign | GDP pc |
| AUT | Austria | 6 572 000 | 5 367 | Sovereign | population; GDP pc |
| HUN | Hungary | 8 266 000 | 3 633 | Sovereign | population; GDP pc |
| CZE | Czechoslovakia | 14 052 000 | 4 154 | Sovereign | — |
| POL | Poland | 28 752 000 | 3 202 | Sovereign | — |
| LTU | Lithuania | 2 197 000 | 2 200 | Sovereign | GDP pc |
| LAT | Latvia | 1 845 000 | 3 400 | Sovereign | GDP pc |
| EST | Estonia | 1 111 000 | 3 200 | Sovereign | GDP pc |
| FIN | Finland | 3 288 000 | 3 711 | Sovereign | population; GDP pc |
| SWE | Sweden | 6 033 000 | 5 488 | Sovereign | population; GDP pc |
| NOR | Norway | 2 738 000 | 4 733 | Sovereign | population; GDP pc |
| DEN | Denmark | 3 443 000 | 6 931 | Sovereign | population; GDP pc |
| ISL | Iceland | 102 000 | 3 037 | Sovereign | GDP pc |
| SOV | Soviet Union | 140 889 000 | 1 776 | Sovereign | — |
| ROM | Romania | 16 710 000 | 1 395 | Sovereign | GDP pc |
| YUG | Kingdom of the Serbs, Croats and Slovenes | 12 736 000 | 1 758 | Sovereign | GDP pc |
| BUL | Bulgaria | 5 268 000 | 1 949 | Sovereign | GDP pc |
| ALB | Albania | 954 000 | 1 428 | Sovereign | population |
| GRE | Greece | 5 859 000 | 3 411 | Sovereign | GDP pc |
| TUR | Turkey | 13 794 000 | 1 199 | Sovereign | GDP pc |
| LIE | Liechtenstein | 12 100 | 5 367 | Sovereign | GDP pc |
| MCO | Monaco | 24 700 | 6 641 | Sovereign | GDP pc |
| AND | Andorra | 5 300 | 3 720 | Sovereign | GDP pc |
| USA | United States | 128 600 000 | 10 217 | Sovereign | population; GDP pc |
| CAN | Canada | 9 472 000 | 6 918 | Dominion of GBR | population; GDP pc |
| NFL | Newfoundland | 270 000 | 3 500 | Dominion of GBR | GDP pc |
| MEX | Mexico | 15 732 000 | 2 640 | Sovereign | population; GDP pc |
| GUA | Guatemala | 1 674 000 | 2 327 | Sovereign | population; GDP pc |
| HON | Honduras | 835 000 | 2 042 | Sovereign | population; GDP pc |
| SAL | El Salvador | 1 285 000 | 1 476 | Sovereign | population; GDP pc |
| NIC | Nicaragua | 660 000 | 2 316 | Sovereign | population; GDP pc |
| CRI | Costa Rica | 455 000 | 2 654 | Sovereign | population; GDP pc |
| PAN | Panama | 496 000 | 2 514 | Sovereign | population; GDP pc |
| CUB | Cuba | 3 388 000 | 2 560 | Sovereign | population; GDP pc |
| HAI | Haiti | 2 254 000 | 1 100 | Sovereign | population; GDP pc |
| DOM | Dominican Republic | 1 047 000 | 1 300 | Sovereign | GDP pc |
| COL | Colombia | 6 972 000 | 2 000 | Sovereign | population; GDP pc |
| VEN | Venezuela | 3 096 000 | 2 289 | Sovereign | population; GDP pc |
| ECU | Ecuador | 1 858 000 | 1 765 | Sovereign | population; GDP pc |
| PER | Peru | 5 031 000 | 2 455 | Sovereign | population; GDP pc |
| BOL | Bolivia | 2 250 000 | 2 144 | Sovereign | population; GDP pc |
| CHL | Chile | 3 944 000 | 4 903 | Sovereign | population; GDP pc |
| ARG | Argentina | 10 206 000 | 6 247 | Sovereign | population; GDP pc |
| URY | Uruguay | 1 516 000 | 4 208 | Sovereign | population; GDP pc |
| PRY | Paraguay | 776 000 | 2 000 | Sovereign | population; GDP pc |
| BRA | Brazil | 30 028 000 | 1 298 | Sovereign | population; GDP pc |
| CHN | China | 477 366 000 | 998 | Sovereign | — |
| JAP | Japan | 83 515 000 | 2 878 | Sovereign | population; GDP pc |
| MGL | Mongolia | 682 000 | 700 | Satellite of SOV | GDP pc |
| TUV | Tannu Tuva | 70 000 | 700 | Satellite of SOV | population; GDP pc |
| TIB | Tibet | 1 000 000 | 700 | Sovereign | population; GDP pc |
| SIA | Siam | 10 794 000 | 1 283 | Sovereign | population |
| AFG | Afghanistan | 6 424 000 | 1 000 | Sovereign | GDP pc |
| PRS | Persia | 10 660 000 | 1 634 | Sovereign | — |
| NEP | Nepal | 5 556 000 | 847 | Sovereign | — |
| BHU | Bhutan | 258 000 | 700 | Protectorate of GBR | GDP pc |
| RAJ | British Raj | 332 004 000 | 1 119 | CrownColony of GBR | population; GDP pc |
| IRQ | Iraq | 3 683 000 | 1 516 | Mandate of GBR | — |
| TRJ | Transjordan | 209 000 | 1 880 | Mandate of GBR | — |
| HEJ | Hejaz | 729 000 | 1 000 | Sovereign | GDP pc |
| NEJ | Nejd | 1 701 000 | 900 | Sovereign | GDP pc |
| YEM | Yemen | 2 949 000 | 900 | Sovereign | GDP pc |
| OMA | Muscat and Oman | 426 000 | 900 | Protectorate of GBR | GDP pc |
| KUW | Kuwait | 119 000 | 1 200 | Protectorate of GBR | GDP pc |
| EGY | Egypt | 13 904 000 | 1 598 | Sovereign | — |
| ETH | Ethiopia | 14 000 000 | 600 | Sovereign | population; GDP pc |
| LBR | Liberia | 612 000 | 1 599 | Sovereign | GDP pc |
| SAF | Union of South Africa | 7 888 000 | 2 133 | Dominion of GBR | GDP pc |
| AUS | Australia | 6 438 000 | 8 155 | Dominion of GBR | population; GDP pc |
| NZL | New Zealand | 1 426 000 | 8 114 | Dominion of GBR | population; GDP pc |

## Components per tag

Population on 1925-01-01 and GDP per capita in 2011 international dollars. `MPD` = Maddison
Project Database 2023; `OWID` = Our World in Data population; `census interp A/B` = linear between
the two censuses; `est` = estimate explained above; `proxy` = 800 $ sub-Saharan proxy; a
three-letter code = that country's Maddison value.

#### GBR: 117 808 000 people, 3 806 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| United Kingdom (GB + N. Ireland) | 44 987 000 | 8 199 | MPD | MPD |
| Isle of Man, Channel Islands | 140 000 | 8 199 | est | GBR |
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
| Weihaiwei | 150 000 | 998 | est | CHN |
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

#### FRA: 106 431 000 people, 3 264 $ per person

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
| Guangzhouwan | 210 000 | 998 | est | CHN |
| French India | 290 000 | 1 113 | est (censuses) | IND |
| Reunion, Guadeloupe, Martinique, Guiana, SPM | 715 000 | 1 300 | est (1921/1926 censuses) | est |
| New Caledonia, Polynesia, Wallis, 1/2 New Hebrides | 124 000 | 1 300 | est + OWID | est |

#### BEL: 19 273 000 people, 3 467 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Belgium | 7 743 000 | 7 438 | MPD | MPD |
| Belgian Congo | 8 583 000 | 800 | OWID | proxy |
| Ruanda (mandate) | 1 435 000 | 800 | OWID | proxy |
| Urundi (mandate) | 1 512 000 | 800 | OWID | proxy |

#### NLD: 65 319 000 people, 2 227 $ per person

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

#### ESP: 23 104 000 people, 3 624 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Spain (incl. Canaries, Ceuta, Melilla) | 22 180 000 | 3 720 | MPD | MPD |
| Spanish Morocco | 750 000 | 1 428 | est | MAR |
| Ifni, Spanish Sahara | 50 000 | 800 | est | proxy |
| Spanish Guinea | 123 000 | 800 | OWID | proxy |

#### POR: 15 042 000 people, 1 459 $ per person

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

#### ITA: 42 288 000 people, 4 357 $ per person

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

#### DEN: 3 443 000 people, 6 931 $ per person

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

#### USA: 128 600 000 people, 10 217 $ per person

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

#### CHN: 477 366 000 people, 998 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| China (MPD minus Tibet and leased territories) | 477 366 000 | 998 | MPD minus TIB/leases | MPD interp 1913-1929 |

#### JAP: 83 515 000 people, 2 878 $ per person

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

#### ETH: 14 000 000 people, 600 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Ethiopia | 14 000 000 | 600 | est | est |

#### LBR: 612 000 people, 1 599 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Liberia | 612 000 | 1 599 | OWID | MPD |

#### SAF: 7 888 000 people, 2 133 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Union of South Africa | 7 578 000 | 2 171 | census interp 1921/1936 | MPD |
| South West Africa (mandate) | 311 000 | 1 200 | OWID | est |

#### AUS: 6 438 000 people, 8 155 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| Australia | 5 881 000 | 8 851 | MPD | MPD |
| Papua, New Guinea (mandate), Nauru | 557 000 | 800 | OWID | proxy |

#### NZL: 1 426 000 people, 8 114 $ per person

| Component | Population | GDP pc (2011$) | Population source | GDP pc source |
|---|---:|---:|---|---|
| New Zealand | 1 366 000 | 8 435 | MPD | MPD |
| Western Samoa, Cook Is., Niue | 60 000 | 800 | OWID + est | proxy |
