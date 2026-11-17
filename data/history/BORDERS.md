# Borders 1925-2025: notes on `borders_draft.ron`

Draft for review. 293 lines: 143 Secession, 61 Transfer, 33 Annexation, 33 Rename, 23 Status;
133 tags that do not exist in 1925.

## How it was built and checked

- `events.py` holds the event table (one `E(...)` per output line). `build_borders.py` replays
  ownership of the 870 states in date order and stops on any line that moves a state not held
  by `from` on that date, or annexes a country that holds nothing. The final replay has 0
  errors. Order within a day is by `from`.
- Each line cites English Wikipedia pages. The script resolves each title, following
  redirects, to its Wikidata item through `pageprops`. It then fetches the item's P571, P576,
  P580, P582 and P585, plus the wikitext of every cited page, and looks for the line's date in
  both.
  - Result: 132 lines have the date in Wikidata and Wikipedia, 147 in Wikipedia only, 3 in
    Wikidata only, and 3 match Wikidata only to the year or month.
  - `source` names the Wikidata property only when its date equals the line's date. Many
    country items carry year-only or later inceptions (Ghana P571 1957-01-01, Malaysia 1963,
    Nigeria 1963), so the Wikipedia page is often the confirming source.
  - Rerun: `BF_CACHE=<cache dir> python3 -I build_borders.py`. The cache defaults to `.cache/`
    next to the script; this run cached in the session scratchpad.
- Checked by hand (web fetch or search, not by the automatic match):
  - Iran 1935-03-21: *Names of Iran*, "as from the next 21st March".
  - Tanzania 1964-10-29: *History of Tanzania*, "On 29 October of the same year".
  - Azerbaijan People's Government 1945-11-20 to 1946-12-12: infobox `date_start` and
    `date_end`.
  - Thailand 1949-05-11: UN note and Bangkok Post timeline.
  - Katanga 1963-01-17: *Operation Grandslam*.
  - Heard Island 1950-12-19: exchange of notes, backdated to Dec 1947.
  - Albania 1939-04-12, 1941-06-07 and 1943-09-08: *Italian protectorate of Albania*.
  - Cambodia 1989-04-30: *People's Republic of Kampuchea*.
  - Egypt 1971-09-11: *Egyptian Constitution of 1971*.
  - Bouvet 1928-01-23: infobox.
  - Vilnius 1939-10-10: *Vilnius Region*.
  - Kampuchea 1945-03-13: article lead.
  - Manchukuo 1945-08-18: Wikidata P576.
- Day not confirmed (only month or year found; flag before use):
  - Spratly lines 1933-07-25 and 1946-12-12 (only "1933" and "1946" found).
  - Bulgarian entry into Macedonia and Thrace 1941-04-20 (only "April 1941").
  - Siam rename 1945-09-08 (sources give 7, 8 or 9 September).
  - Jordan rename 1949-04-26 (commonly cited; Wikipedia says only "1949").
  - Kwantung 1945-09-02 (Wikipedia: "September 1945").
  - Laos puppet end 1946-04-24 (date of the French retaking of Vientiane, approx.).
  - Kosovo back to Yugoslavia 1944-11-29 (approx.: end of German control).
  - Bhutan 2011-01-01: the date the 2007 treaty took effect according to *Bhutan-India
    relations*; the signing date is not given there.

## Criteria used

- **Date.** The day sovereignty or administration actually changed hands. A unilateral
  declaration the old sovereign defeated or ignored counts only once it took effect:
  - Bangladesh 1971-12-16, not 03-26.
  - Guinea-Bissau 1974-09-10.
  - Indonesia 1949-12-27, not 1945-08-17.
  - Algeria 1962-07-03, recognition, not proclamation on 07-05.
  - Baltic states 1991-09-06, USSR State Council recognition.

  The other Soviet republics, and Slovenia, Croatia, Macedonia and Bosnia, take their
  declaration dates. These are the dates Wikipedia and Wikidata use; Moscow's control was
  collapsing by then. The alternative is 1991-12-26 for every Soviet republic and 1991-10-08
  for Slovenia and Croatia (end of the Brioni moratorium).
- **WWII annexations.** Recorded only where the annexing state made a formal act of
  incorporation, or set up a client state. Occupation regimes are not recorded:
  - General Government, the Reichskommissariats, occupied Serbia (Nedic) and the Italian
    governorate of Montenegro.
  - Japanese-occupied China, Hong Kong, Malaya, the Dutch East Indies and Guam.
  - Transnistria Governorate, Italian Ionian Islands, Alsace-Moselle and Lower Styria.
  - Luxembourg (Wikipedia: "de facto annexed").

  Collaborationist governments that claimed to be the existing state are not border changes:
  Vichy, the Italian Social Republic, Wang Jingwei's Reorganized National Government of China,
  and the Nedic and Greek collaborationist governments. Bulgaria's Vardar Macedonia and Thrace
  (1941-44) are included as administered-as-annexed and are flagged in `note`.
- **Status.**
  - A dominion becomes sovereign when the Statute of Westminster is in force for it: CAN, SAF
    and IRL 1931-12-11, AUS 1942-10-09 (retroactive to 1939-09-03), NZL 1947-11-25.
  - A mandate or protectorate becomes sovereign when the treaty ends it: IRQ 1932, TRJ 1946,
    KUW 1961, OMA 1970, BHU 2011.
  - Satellite to sovereign: MGL 1992-02-12 (new constitution; Soviet troops gone 1992).
  - Sovereign to occupied and back: GER 1945-06-05 / 1955-05-05, JAP 1945-09-02 / 1952-04-28,
    AUT 1955-07-27 (it was restored by a Secession in 1945).
  - Also: HEJ under NEJ (1926), NFL to commission rule (1934), ALB under ITA and then GER
    (1939, 1943, 1944), BHU passing from GBR to India's guidance (1949).
  - For Status, `to` is the new overlord; `to == from` means the country becomes sovereign.
- **Rename.** Only changes of the common English name, with the new name in `note`. Changes of
  form of government with the same short name are left out (Republic of India 1950, Republic
  of South Africa 1961, Islamic Republic of Pakistan 1956, Czechia 2016, Turkiye 2022). Also
  included: CHN 1949 (PRC, a change of government), YEM 1962 and 1990, KHM 1970/1976/1989.
- **De facto states.** Included when they held a whole state for more than about a year, marked
  "de facto" in `note`: Katanga, Biafra, Rhodesia, Somaliland 1991, Ichkeria, Transnistria,
  Abkhazia, Taiwan, Kosovo, Crimea 2014 and the oblasts claimed in 2022. Excluded as short or
  below resolution:
  - Carpatho-Ukraine (one day), the Republic of Kuwait (Aug 1990) and the Democratic Republic
    of Yemen (1994).
  - Republika Srpska, Serbian Krajina, Nagorno-Karabakh, South Ossetia and Northern Cyprus.
  - The DPR and LPR in 2014, the SADR, and the bantustans.
- **Occupation zones.** No `Occupation` lines. Zones are recorded in `occupier` on the line that
  creates the post-war state or status:
  - AUT 1945 and GER 1945: "USA,GBR,FRA,SOV".
  - KOR 1948: USA. PRK 1948: SOV. DDR 1949: SOV. SAA 1947: FRA.
  - JAP 1945: USA. Manchuria and Kwantung 1945: SOV. Azerbaijan 1945: SOV.

  Reason: in Germany and Korea the zones became the post-war states, so the creation lines
  already carry them. Austria's and Japan's zones changed no sovereignty. The four German zones
  follow state lines only roughly. If the game needs zones as units, add `Occupation` lines
  later from this list.
- **Majority rule.** A state goes with its largest share, a plurality where no country passes
  0.5:
  - JAP-NANYO goes to FSM (0.35).
  - GBR-LEEWARD goes to DMA (0.41).
  - GBR-WINDWARD goes to LCA (0.46).
  - BEL-RUANDA-URUNDI is 0.50/0.50 in the csv; it goes to BDI because Burundi is larger
    (27,834 vs 26,338 km2), so Rwanda gets no state.

## Tag decisions

- Continuity:
  - SOV = USSR, then Russia (Rename 1991-12-26).
  - GER = Reich, then FRG; DDR seceded 1949 and was annexed back 1990.
  - RAJ = British India, then India (Status and Rename 1947-08-15).
  - YEM = Kingdom of Yemen, then the YAR, then the Republic of Yemen.
  - EGY keeps its tag through the UAR (1958-61, and the name until 1971), instead of a UAR
    tag. Egypt is UAR's legal continuator, and a UAR tag would leave Egypt without EGY after
    1971. Switch to UAR if the game prefers a separate entity.
  - TZA = Tanganyika, continued by Tanzania. MYS = Federation of Malaya, continued by
    Malaysia. VNM = DRV, continued by the SRV. CHN = ROC on the mainland, then the PRC; TWN =
    ROC on Taiwan from 1949-12-07.
- Restored tags: LTU, LAT, EST (1991), AUT (1945), ETH (1941), SAA (Saar Protectorate 1947),
  SYR (1961), KUW (1991), ALB (Status). SML is used for both the 1960 State of Somaliland and
  the 1991 Somaliland, which claims its succession.
- Yugoslavia: SFRY dissolved (Badinter); FRY is an heir with tag SCG from 1992. Serbia keeps
  SCG after 2006 as the legal continuator (alternative: retag to SRB).
- Clashes and suffixes:
  - CZR: the Czech Republic, because ISO CZE is the 1925 tag of Czechoslovakia.
  - SLK: Slovak Republic 1939-45.
  - Wartime clients whose ISO code belongs to the later state: MMRX (State of Burma), PHLX
    (Second Philippine Republic), VNMX (Empire of Vietnam), KHMX (Kingdom of Kampuchea 1945),
    LAOX (Kingdom of Luang Prabang 1945).
  - ISO TUV (Tuvalu) equals the 1925 tag of Tuva, but Tuvalu is below resolution, so it is not
    used.
- Customary codes:
  - ISO 3166-3: YMD, DDR, RHO.
  - Given in the brief: MCK, NDH, RVN.
  - Chosen here: HTY (Hatay), AZG (Azerbaijan People's Government), MLF (Mali Federation), ZAN
    (Zanzibar), KAT, BIA, ICH (Ichkeria), PMR, ABH.
  - XKX: Kosovo (EU/IMF user-assigned code; Wikidata lists it under XKS).

## Partial shares that are approximate (estimated, not from `modern_iso3_by_area`)

| Line | State | Share | What |
|---|---|---|---|
| 1932 MCK | CHN-NM | 0.40 | Xing'an provinces + Jehol (1933) |
| 1939 POL->GER | POL-WARSZAWA 0.35, POL-KRAKOW 0.2, POL-KIELCE 0.2, POL-BIALYSTOK 0.15 | | Zichenau, Chrzanow/Zywiec, Zaglebie, Suwalki |
| 1939 POL->GER | POL-LODZ moved whole | | most of the voivodeship (Lodz) went to Wartheland; to check |
| 1940 ROM->HUN | ROU-TRANSILVANIA | 0.40 | rest of Northern Transylvania |
| 1941 NDH | HRV-DALMACIJA 0.35, SRB-VOJVODINA 0.15 | | Italian Dalmatia; Syrmia |
| 1941 YUG->BUL | KOS | 0.10 | Gnjilane area |
| 1941 GRE->BUL | GRC-MAKEDONIA | 0.20 | Eastern Macedonia |
| 1943 MMRX | MMR-SHAN | 0.25 | Kengtung and Mongpan to Thailand |
| 1947 ITA->GBR | LBY-TRIPOLITANIA | 0.37 | Tripolitania proper; the state is moved to FRA because Fezzan (~551,000 km2 of 870,100) is the majority |
| 1949 DDR | DEU-BE | 0.45 | East Berlin |
| 1954 VNM | VNM-ANNAM | 0.27 | Annam north of the 17th parallel |

The other partial shares come from `modern_iso3_by_area`: Bengal 0.20, Kashmir 0.41, Assam 0.05,
Bukovina 0.48, Neumark (DEU-BB) 0.32, Kurils (JPN-HOKKAIDO) 0.12, Gangwon 0.37 and Venezia
Giulia to Croatia 0.36.

## Below resolution (no line; recorded here only)

- Europe:
  - Svalbard, Norwegian from 1925-08-14 (about 0.35 of NOR-NORD-NORGE; the state was already
    NOR).
  - Vatican City 1929-02-11.
  - Eastern Greenland case 1933.
  - Sudetenland 1938-10-01 (part of CZE-CECHY/MORAVA); First Vienna Award 1938-11-02 (about
    0.21 of SVK, the south of Ruthenia); Zaolzie 1938 to Poland.
  - Memel 1939-03-22 (about 0.05 of LTU).
  - Finnish Karelia and Petsamo 1940/1944. The map puts these areas in SOV states and
    FIN-VIIPURI is the modern Finnish part only, so the map does not show them. Same for the
    Finnish Democratic Republic (1939).
  - Eupen-Malmedy 1940.
  - Southern Dobruja 1940-09-07 to Bulgaria (0.34 of ROU-DOBROGEA, permanent).
  - Italian Ljubljana Province and German Upper Carniola/Lower Styria 1941; Hungarian Backa,
    Baranja, Prekmurje and Medjimurje 1941; the Italian Governorate of Dalmatia.
  - Konigsberg 1945 (0.35 of GER-OSTPREUSSEN to SOV).
  - Tende and Brigue 1947.
  - Trieste: the Free Territory 1947-54, then Zone A to Italy.
  - Dutch annexations of German border villages 1949-63.
  - Porkkala 1944-56.
  - Northern Cyprus 1974/1983 (about 0.36).
  - Monaco, Andorra, San Marino, Liechtenstein: no change.
  - The Kingdom of Iceland is already sovereign in the 1925 data; the 1944 republic needs no
    line.
- Middle East and Asia:
  - Jubaland, Kenya to Italian Somaliland on 1925-06-29: already inside the SOM state, so the
    1925 data is slightly off.
  - Aqaba-Ma'an 1925 (0.09 of SAU-HEJAZ to Transjordan).
  - Mosul treaty 1926: IRQ already holds the state.
  - Najran and Jizan 1934.
  - West Bank to Jordan 1950 (0.20 of GBR-PALESTINE); Gaza under Egypt 1948-67; Israeli
    occupation 1967; Sinai 1967-82; Golan 1967; State of Palestine 1988.
  - Mahabad 1946.
  - Gwadar 1958 to Pakistan.
  - Chandernagore 1950/52; Dadra and Nagar Haveli 1954; Aksai Chin.
  - Sikkim: RAJ holds IND-SIKKIM in 1925, so its protectorate status 1947-75 and the 1975
    merger are not recorded.
  - Franco-Thai war 1941: about 0.3 of KHM and 0.1 of LAO, returned 1946.
  - Mengjiang 1936/39 (no majority in CHN-NM, SX or ZHILI); East Hebei; East Turkestan
    Republics 1933 and 1944-49.
  - Singapore 1963/1965 (0.15 of MYS-STRAITS).
  - Cocos 1955.
  - Amami 1953 and Bonin 1968 returned to Japan.
  - Andaman under nominal Azad Hind rule 1943-45.
- Africa:
  - Sarra Triangle 1934.
  - Tangier 1956; Tarfaya 1958; Ifni 1969.
  - Rwanda 1962 (0.50 of BEL-RUANDA-URUNDI; see the majority rule).
  - Northern Cameroons to Nigeria 1961.
  - Mauritania's Tiris al-Gharbiyya 1976-79; the SADR.
  - Walvis Bay 1994.
  - The bantustans 1976-94.
  - Eritrea's full annexation by Ethiopia 1962 (an internal change after the 1952
    federation).
- Americas:
  - Leticia 1932-33; Chaco War 1932-38 (the map already uses modern borders); Ecuador-Peru
    1942.
  - Panama Canal Zone 1979/1999.
  - Grenada 1974, Saint Vincent 1979, Antigua 1981, Saint Kitts 1983 (minority parts of the
    Windward and Leeward states).
  - Curacao, Aruba and Sint Maarten as constituent countries (no change of sovereignty).
- Pacific and Indian Ocean:
  - Marshall Islands 1986, Palau 1994, Northern Marianas (minority parts of JAP-NANYO).
  - Tuvalu 1978.
  - Cook Islands 1965 and Niue 1974: free association with NZL, not recorded as sovereign.
  - Chagos: the UK-Mauritius treaty signed 2025-05-22 was not in force by 2025-12-31.

## Biggest doubts

1. **2022 oblasts.** UKR-09, 14, 23 and 65 move whole to SOV because Russia held most of each
   at the end of 2025. Year-end estimates:
   - DeepState: Donetsk 78%, Zaporizhzhia 75%, Kherson 72%, Luhansk about 99%.
   - ISW (Oct 2025): Zaporizhzhia 65%.

   Crimea 2014 is also de facto. The csv's `modern_iso3_by_area` already gives Crimea to RUS,
   but the line on 1991-08-24 gives it to Ukraine because it was part of the Ukrainian SSR.
2. **1939-45 Eastern Europe.** Shares in Poland, Transylvania and Dalmatia are estimates (table
   above). Bulgaria's Vardar Macedonia and Thrace are included although the annexation was
   never formal in international law. POL-LODZ moved whole on a majority estimate.
3. **De facto states.** Including Ichkeria (1991-2000), Transnistria, Abkhazia, Somaliland and
   Katanga is a judgement call. Remove them if the game wants only recognised sovereignty.
4. **Plurality cases.** FSM, Dominica, Saint Lucia and Burundi get whole states they share with
   other countries.
5. **Libya 1947-51.** LBY-TRIPOLITANIA is mostly Fezzan, so it goes to France; Tripolitania
   proper was British.
6. **Status dates.**
   - Oman 1970 (de facto British protection ended; the earlier option was the 1951 treaty).
   - Bhutan 2011-01-01 (in-force date from Wikipedia; signed 2007).
   - Mongolia 1992.
   - Australia 1942 vs 1939.
7. **Hejaz 1926.** Modelled as Status (personal union) and then Annexation plus Rename in 1932.
   The conquest in Dec 1925 could instead be an Annexation at 1926-01-08.
8. **Spratlys (PGA).** GBR to FRA 1933, to JAP 1939, to CHN 1946; after 1949 Itu Aba is held by
   the ROC (TWN). Days not confirmed.
9. **Puppet ends.** VNMX returns to FRA on 1945-08-25, though the DRV held the north until
   1946. MMRX returns on 1945-03-27; LAOX's end date is approximate.
