## Name and numbers

game-title = Between Frontiers
game-subtitle = Political and economic simulator, 1925–2025
game-version = Version { $version } · foundation phase

number-thousands = { $value } thousand
number-millions = { $value } million
number-billions = { $value } billion
money-intl-dollars = ${ $value }
date-long = { $month } { $day }, { $year }
month-1 = January
month-2 = February
month-3 = March
month-4 = April
month-5 = May
month-6 = June
month-7 = July
month-8 = August
month-9 = September
month-10 = October
month-11 = November
month-12 = December

## Main menu

menu-continue = Continue
menu-continue-detail = { $save } · { $date }
menu-continue-empty = No saved campaign
menu-play = Play
menu-settings = Settings
menu-quit = Quit
menu-back = Back

## Settings

settings-title = Settings
settings-language = Language
settings-language-hint = Changes every text at once, country names included.
settings-ui-scale = Interface size
settings-ui-scale-value = { $percent }%
settings-map-labels = Country names on the map
settings-autosave = Autosave
settings-autosave-off = Off
settings-autosave-monthly = Monthly
settings-autosave-yearly = Yearly
settings-saves-folder = Campaign folder
settings-done = Done

## Saved campaigns

saves-title = Campaigns
saves-subtitle = Continue a campaign or start a new one on January 1, 1925.
saves-empty = No saved campaign yet.
saves-empty-hint = Create the first one on the right.
saves-country-pending = Country not chosen yet
saves-last-played = Played { $when }
saves-load = Load
saves-delete = Delete
saves-delete-confirm = Delete “{ $name }”? This cannot be undone.
saves-delete-yes = Delete campaign
saves-delete-no = Keep
saves-new-title = New campaign
saves-new-name = Campaign name
saves-new-name-default = Campaign { $n }
saves-new-start = Start
saves-new-start-value = January 1, 1925
saves-new-seed = World seed
saves-new-seed-hint = The same seed with the same decisions replays the same history.
saves-new-create = Create and choose a country
saves-new-name-taken = A campaign with this name already exists.
saves-new-name-empty = Give the campaign a name.
saves-error-io = Could not write the campaign: { $error }

## Country choice

pick-title = Choose your country
pick-hint = Click a country on the map. Drag to pan, use the mouse wheel to zoom.
pick-back = Back to campaigns
pick-start = Start campaign
pick-none = No country selected

## Country panel

country-status-sovereign = Sovereign country
country-status-subject = { $kind } of { $article ->
    [the] the { $overlord }
   *[other] { $overlord }
}
subject-kind-Dominion = Dominion
subject-kind-Mandate = Mandate
subject-kind-Protectorate = Protectorate
subject-kind-Satellite = Satellite
subject-kind-CrownColony = Crown colony
stat-capital = Capital
stat-government = Government
stat-population = Population
stat-gdp = GDP
stat-gdp-per-capita = GDP per capita
stat-gdp-unit = 2011 international dollars
stat-gdp-unit-live = Dollars at the simulation's base prices
stat-area = Area
stat-area-value = { $value } km²
stat-states = States
stat-subjects = Subjects
stat-rank-population = #{ $rank } in the world by population
stat-rank-gdp = #{ $rank } economy in the world
stat-difficulty = Difficulty
difficulty-great = Great power
difficulty-major = Regional power
difficulty-minor = Minor country
difficulty-subject = Subject

## Map

map-sea = Sea
map-unowned = Unowned
map-tooltip-province = { $province }, { $country }

## Campaign

game-paused = Paused
game-speed = Speed { $speed }
game-menu = Menu
game-menu-save = Save
game-menu-saved = Campaign saved
game-menu-resume = Resume
game-menu-exit = Exit to main menu

## Top bar

number-short-thousands = { $value }K
number-short-millions = { $value }M
number-short-billions = { $value }B
bar-gdp = GDP
bar-population = Population
bar-quality = Quality of life
bar-literacy = Literacy
bar-life = Life expectancy
bar-urban = Urbanization
bar-unknown = —
bar-years = { $value } years
bar-percent = { $value }%
bar-of-hundred = { $value } of 100
bar-rank = #{ $rank } in the world
bar-gdp-tip = Value of everything the country produces in a year.
bar-gdp-per-capita = Per person: { $value }
bar-population-tip = People living in the metropole. Colonies count apart.
bar-density = Density: { $value } per km²
bar-quality-tip = Geometric mean of income, health and education, from 0 to 100. One weak dimension pulls the whole index down.
bar-quality-income = Income
bar-quality-health = Health
bar-quality-education = Education
bar-quality-missing = Data is missing to compute this country's index.
bar-literacy-tip = Adults (15 and older) who can read and write.
bar-life-tip = Years a newborn would live at today's mortality.
bar-urban-tip = Share of the population living in urban areas.
bar-country-tip = Your country. Click for its summary.
bar-speed-down = Slower
bar-speed-up = Faster
bar-pause = Pause
bar-play = Resume
bar-date-tip = Speed { $speed } of 5. Click to pause or resume.
bar-menu = Menu
source-economy = Sources: Maddison Project 2023 and Our World in Data; own estimates where data is missing.
source-social = Sources: Our World in Data and HYDE; own estimates where data is missing.

## Corner panel

hud-per-capita = Income per capita
hud-per-capita-tip = GDP divided by population. The bar runs from $300 to $40,000, the century's range.
hud-density = Density
hud-density-value = { $value }/km²
hud-density-tip = People per km² of territory.
hud-area = Area
hud-area-tip = Territory of the metropole. Colonies count apart.
hud-states = States
hud-states-tip = States of the metropole, the unit of the simulation. Colonies count apart.
hud-subjects = Subjects
hud-subjects-tip = Countries under this one's overlordship: dominions, mandates, protectorates and satellites.
hud-subjects-none = No subjects.
hud-gdp-share = Share of world GDP
hud-gdp-share-tip = Part of everything the world produces that comes from this country.
hud-pop-share = Share of world population
hud-pop-share-tip = Part of humanity living in this country.
hud-rank-quality = Rank in quality of life
hud-rank-population = Rank in population
hud-rank-value = #{ $rank }
hud-due-to = This is due to:
hud-world-total = World total: { $value }
hud-life-bar = The bar runs from 20 to 85 years.
hud-medal-tip = #{ $rank } economy in the world
hud-metropole = Metropole
hud-colonies = Colonies
hud-total = Total
hud-includes-colonies = Colonies included.
country-colonies = Colonies
country-colonies-value = { $population } · { $states } states
settings-map-relief = Terrain relief on the map
saves-error-load = Could not open the campaign: { $error }
hud-engine = Simulation day { $tick } · state { $hash }

## Simulation figures

unit-scenario = In 2011 international dollars (Maddison Project), those of the 1925 scenario.
unit-live = In dollars at the simulation's base prices: fixed catalogue prices, no inflation. Not to be added to or compared directly with the scenario's 2011 dollars.
source-live = Source: the simulation, read every month.
source-live-gdp-year = GDP of { $year }, the last full year.
source-live-gdp-calibration = No full year of play yet: the GDP calibrated in 1925 stands.
hud-gdp-split = Metropole and colonies trade in the same market: its GDP is split between them by people's income, and in the first year as in the scenario.
hud-life-scenario = 1925 figure: the simulation does not compute life expectancy yet.
hud-quality-life-scenario = Health uses the 1925 life expectancy, which the simulation does not compute yet.

## Map modes

lens-title = Map mode
lens-political = Political
lens-resources = Resources
lens-climate = Climate
lens-farms = Farming
lens-no-data = Resource and land data are not loaded yet.
legend-deposit = Deposit size
legend-small = Small
legend-large = World class
legend-suitability = Suitability
legend-low = Low
legend-high = High
tip-deposit = { $resource }: deposit of size { $size } of 5
tip-climate = { $climate } climate on { $share } of the land
tip-farm = { $farm }: suitability { $value }
lens-infrastructure = Infra
legend-local = Demand met in the state
legend-unmet = Short
legend-met = Met
tip-local = { $good }: { $share } of demand met

## Research
bar-research = Research
research-title = Research
research-close = Close
research-summary = { $points } research points a month · focus: { $focus }
research-focus-auto = chosen automatically
research-focus-clear = Let the country choose
research-set-focus = Research
research-focused = In focus
research-locked = Needs other technologies
research-locked-short = Locked
research-requires = Requires: { $list }
research-unlocks = Unlocks: { $list }
research-history = In history: prototype in { $demonstrated }, industry in { $industrialized }
research-first-industrial = First made at scale in this game in { $year }
research-state-buyer = The state can be the first buyer
research-level = Level { $n } · ≈ { $cost } points
research-level-short = Level { $n }
research-cost = Level { $n } · { $cost } points to reach science
research-zoom-in = Zoom in
research-zoom-out = Zoom out
research-fit = Fit
research-fit-hint = Show the whole tree
research-all-fields = All fields

## Government window
bar-government = Government
gov-title = Government
gov-tab-budget = Budget
gov-tab-money = Money and debt
gov-tab-laws = Laws
gov-close = Close
gov-coming = Not built yet.

## Laws (government tab)
laws-missing = The law data was not found.
laws-not-held = Does not apply: no colonies
laws-not-held-hint = The country has no colonies; these laws only hold for those that do.
laws-in-force = In force
laws-targeted = Targeted
laws-no-effects = Changes no effect.
laws-scope-metropole = Holds in the metropole
laws-scope-targeted = Holds in the communities the law targets
laws-scope-colonies = Holds in the colonies
laws-source = Legal source of the laws in force (1925)
laws-propose = Propose law
laws-propose-pending = Already proposed; the simulation decides on the next day.
laws-propose-in-force = This law is already in force.
laws-propose-not-held = The country has no colonies, and this law only holds in them.
laws-changes = What changes
laws-changes-from = Against the law in force, { $law }:
laws-changes-none = No effect changes.
laws-stances = Interest groups
laws-stances-hint = Law in force: each term is how much one of the law's effects weighs for the group, in points (the weight the group gives the effect times how far the law moves it from neutral). The sum is what the group gains from it; there is no ideology.
laws-stances-hint-change = Each term is how much one effect of the change weighs for the group, in points (the weight the group gives the effect times how much it moves, with every law in force counted). The sum is the group's stance on the change; there is no ideology. In brackets, the group's clout.
laws-stance-for = for
laws-stance-against = against
laws-stance-indifferent = indifferent
laws-terms-none = No effect this group weighs.
laws-value-cap = { $value } % ceiling
laws-value-no-cap = no ceiling
laws-value-share = { $value } %
laws-value-multiplier = ×{ $value }
laws-pending = Proposed
laws-row-pending = { $law } · proposal pending
laws-legitimacy = Legitimacy
laws-movements-none = No active movement.
laws-movement-support = { $value } % support
laws-balance = Clout of the groups: { $for } % for, { $against } % against.
laws-group-clout = { $group } ({ $clout } %)
laws-history = Changes in the campaign
laws-history-line = { $date }: { $from } → { $to } ({ $by })
laws-by-command = by the government
laws-by-groups = by the interest groups
laws-by-movement = by the { $movement } movement
laws-balance-none = No group with clout backs this change. Once laws go through a passage, a law without support will not pass; for now a proposal is enacted on the next day.
laws-balance-minority = Most of the clout is against. Once laws go through a passage, the law will depend on changing that sum; for now a proposal is enacted on the next day.
laws-balance-majority = Most of the clout backs this change.
laws-effects-inert = The effects of laws do not act on the economy yet: the change moves the law in force and the groups' stances, not the country's figures.
laws-migration-in-budget = The internal migration the simulation uses today is set by the migration law, in the Budget tab.
