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
stat-area = Area
stat-area-value = { $value } km²
stat-provinces = Provinces
stat-subjects = Subjects
stat-rank-population = #{ $rank } in the world by population
stat-rank-gdp = #{ $rank } economy in the world
stat-source = Source: { $source }
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
