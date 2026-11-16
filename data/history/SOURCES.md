# Historical reference series: sources and licences

The files in this folder are the reference the historical scorecard compares a 100-year run
with (`tools/history/scorecard.py`, `docs/history.pt-BR.md`). They are trimmed copies of public
datasets, by **modern-border country** (ISO 3166 alpha-3) and year, 1900–2025. Nothing here was
typed by hand: every number comes out of `tools/history/fetch.py`, which downloads the sources to
`data/raw/history/` (gitignored) and writes these files. To rebuild them:

    python3 tools/history/fetch.py            # download and rebuild
    python3 tools/history/fetch.py --offline  # rebuild from the files already downloaded

How modern countries add up to the 1925 tags is `tools/history/mapping.csv`.

## Licence rule

The game is not sold, so every public dataset may be used, non-commercial licences included
(decision of the project owner, November 2026). Each file keeps the licence of its source, stated
below; a file under a non-commercial or share-alike licence other than CC BY-SA 4.0 is **not**
relicensed with the rest of the repository's data. Sources whose terms forbid passing the data on
stay out of the repository (see "Left out").

## Files

| File | Content | Source | Licence | Years |
|---|---|---|---|---|
| `maddison.csv` | GDP per head (2011 international dollars) and population | Maddison Project Database 2023, `mpd2023_web.xlsx`, doi:10.34894/INZBF2 | CC BY 4.0 | 1900–2022, many countries only from 1950; `SUN`, `CSK`, `YUG`, `SDN_FORMER` are former states |
| `population.csv` | Population | Our World in Data, "Population" (`population`); after 2023 the UN WPP 2024 medium variant (`population-with-un-projections`) | OWID: CC BY 4.0. Origins: HYDE 3.3 (CC BY 4.0) before 1800, Gapminder v7 (CC BY 4.0) 1800–1949, UN World Population Prospects 2024 (CC BY 3.0 IGO) from 1950 | 1900–2025 |
| `urban.csv` | Share of the population in urban areas | OWID, "Share of population living in urbanized areas" (`long-term-urban-population-region`, formerly `urbanization-last-500-years`), from HYDE 3.5, doi:10.24416/UU01-F45D44 | OWID processing CC BY 4.0. HYDE 3.5: the DataCite record of the DOI gives CC BY 4.0, OWID's metadata CC BY-NC 4.0; either is acceptable under the rule above | 1900–1950 every 10 years, then yearly |
| `literacy.csv` | Adult literacy rate | OWID, "Literacy rate" (`cross-country-literacy-rates`) | OWID processing CC BY 4.0. Origins: UNESCO (1953, 1957) CC BY 3.0 IGO; van Zanden et al. (2014), *How Was Life?*, OECD, CC BY 4.0; UNESCO Institute for Statistics, CC BY-SA 4.0; World Bank WDI, CC BY 4.0 | sparse before 1950 |
| `vital.csv` | Births and deaths per 1,000 people a year; life expectancy at birth (years) | OWID `crude-birth-rate` and `crude-death-rate` (UN World Population Prospects 2024, estimates); OWID `life-expectancy` (long run: Riley 2005, Zijdeman et al. 2015 / Clio Infra, Human Mortality Database, UN WPP 2024) | OWID processing CC BY 4.0. UN WPP: CC BY 3.0 IGO. HMD: CC BY 4.0. Clio Infra (Zijdeman et al.): its own dataset terms, not checked; Riley (2005): published estimates used as fact | births and deaths 1950–2023; life expectancy sparse before 1950, yearly after |
| `production.csv` | Production of coal, crude oil and natural gas (TWh of primary energy), electricity (TWh) and cereals (tonnes) | OWID `fossil-fuels` (fuel = coal, oil, gas; production), `electricity-mix` (total generation), `cereal-production` | OWID processing CC BY 4.0. Fossil fuels: Energy Institute, *Statistical Review of World Energy* (© Energy Institute, its own terms of use, not a Creative Commons licence; not read, the site blocks automated access); The Shift Data Portal, after Etemad and Luciani (CC BY-SA 3.0); US EIA (public domain). Electricity: Ember (CC BY 4.0), Energy Institute (as above), Pinto et al. (2023) (CC BY 4.0). Cereals: FAOSTAT (CC BY 4.0) | fossil fuels from 1900; electricity from 1900 for a few countries, 1985 for most; cereals from 1961 |
| `wdi.csv` | Employment in agriculture, industry and services (% of employment); consumer price inflation (%); government revenue, expense and debt (% of GDP) | World Bank, World Development Indicators, via its API: `SL.AGR.EMPL.ZS`, `SL.IND.EMPL.ZS`, `SL.SRV.EMPL.ZS` (ILO modelled estimates), `FP.CPI.TOTL.ZG` (IMF), `GC.REV.XGRT.GD.ZS`, `GC.XPN.TOTL.GD.ZS`, `GC.DOD.TOTL.GD.ZS` (IMF Government Finance Statistics) | CC BY 4.0 (the indicators' own metadata, `License_Type`) | employment 1991–; inflation 1960–; public finance 1972–, patchy |
| `fdi.csv` | Foreign direct investment: net inflows and net outflows (current US$), net inflows (% of GDP) | World Bank, World Development Indicators, via its API: `BX.KLT.DINV.CD.WD`, `BM.KLT.DINV.CD.WD`, `BX.KLT.DINV.WD.GD.ZS`; the indicators' own source is the IMF Balance of Payments database, supplemented by UNCTAD | CC BY 4.0 (the indicators' own metadata, `License_Type`) | 1970– (China from 1979 for inflows and 1982 for outflows; Russia and Ukraine from 1992, Czechia from 1993) |
| `informal.csv` | Informal employment, % of total employment, both sexes, all activities; latest year per country (147 countries, 2009–2025; no China, no United States) | ILOSTAT, indicator `SDG_0831_SEX_ECO_RT_A` (SDG 8.3.1, ICLS13 definition), `https://rplumber.ilo.org/data/indicator?id=SDG_0831_SEX_ECO_RT_A&format=.csv` | CC BY 4.0 (ILO, "Rights and permissions": datasets and their metadata since 3 May 2023) | latest available |

Downloaded 2026-11-16.

## Attribution

- Bolt, J. and van Zanden, J. L. (2024), "Maddison style estimates of the evolution of the world
  economy: A new 2023 update", *Journal of Economic Surveys*, 1–41. Maddison Project Database,
  version 2023, CC BY 4.0.
- Our World in Data (2026): "Population"; "Share of population living in urbanized areas";
  "Literacy rate"; "Fossil fuel production"; "Electricity generation"; "Cereal production" —
  each based on the origins named in the table, CC BY 4.0 for OWID's processing.
- Klein Goldewijk, K. et al., HYDE 3.5 (2025), PBL Netherlands Environmental Assessment Agency /
  Utrecht University, doi:10.24416/UU01-F45D44.
- United Nations, Department of Economic and Social Affairs, Population Division (2024), *World
  Population Prospects 2024*, CC BY 3.0 IGO.
- Energy Institute (2026), *Statistical Review of World Energy*; The Shift Project, The Shift
  Data Portal; U.S. Energy Information Administration, International Energy Statistics; Ember,
  Yearly Electricity Data; FAO, FAOSTAT, Crops and livestock products.
- International Labour Organization (2026), ILOSTAT, SDG indicator 8.3.1 "Proportion of informal
  employment in total employment", CC BY 4.0.
- Riley, J. C. (2005), "Estimates of Regional and Global Life Expectancy, 1800–2001",
  *Population and Development Review* 31(3); Zijdeman, R. and Ribeira da Silva, F. (2015), "Life
  Expectancy at birth (Total)", IISH Dataverse (Clio Infra); Human Mortality Database (2025),
  University of California, Berkeley and Max Planck Institute for Demographic Research.
- World Bank (2026), World Development Indicators, CC BY 4.0, with data from the International
  Labour Organization (ILOSTAT modelled estimates), the International Monetary Fund (Balance of
  Payments, Government Finance Statistics, International Financial Statistics) and UNCTAD.

## Left out

| Series | Why |
|---|---|
| Crude steel by country (OWID `minerals`, USGS and British Geological Survey, 1970–) | The BGS terms (`https://www.bgs.ac.uk/mineralsuk/statistics/world-mineral-statistics/bgs-mineral-statistics-terms-and-conditions-ipr/`) require permission to pass the data to third parties, which a public repository does. `fetch.py` keeps it in `data/raw/history/production_local.csv` and the scorecard uses it on the machine that downloaded it; it is never committed. worldsteel is copyright. |
| Cement by country | No free series by country: the USGS world table by country covers only recent years; the cement CO₂ of the Global Carbon Budget is a proxy, not used. |
| Motor vehicles | OICA, "All Rights Reserved"; no free alternative found. |
| Cereals before 1961, sector employment before 1991, public finance before 1972 | No series by country under usable terms was found (OWID's long-run sector employment, after Herrendorf et al., carries no licence; Clio Infra inflation carries none). The Jordà–Schularick–Taylor Macrohistory Database (CC BY-NC-SA 4.0, 18 advanced economies, 1870–2020) and the IMF "Public Finances in Modern History" (IMF terms) would fill public finance and inflation for rich countries; not added yet. |
