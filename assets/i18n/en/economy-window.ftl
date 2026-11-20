## Economy window: what the country has built
cmd-window-economy = Economy: buildings and works
    .short = Economy
economy-title = Economy
economy-tab-buildings = Buildings
economy-waiting = The figures arrive at the end of the first week.
economy-basis = What your country has built, by kind of building. Output and profit are per year, at this month's prices; use is the share of capacity that runs. Click a building to see its states and the method each one runs.
economy-group-all = All
economy-group-chip = { $group } · { $levels }
economy-col-building = Building
economy-col-levels = Levels
economy-col-workers = Workers
economy-col-output = Output
economy-col-use = Use
economy-col-profit = Profit
economy-col-states = States
economy-levels-building = { $levels } +{ $building }
economy-none = No buildings in this sector.
economy-goto-state = Open the state panel
economy-tab-methods = Methods
methods-basis = How each building produces. Today the historical rule holds: owners pick the method that earns most on its capital and switch once a year if another earns 10% more, the country's investment fund paying the equipment. Figures per level and per year, at this month's prices.
methods-none = The country has no building with more than one method.
methods-kind = { $building } · { $levels }
methods-best = Owners would pick today, where the state has transport and power: { $method }
methods-best-none = No method runs today: an input is missing from the market.
methods-in-use = in { $levels } levels, { $states ->
        [one] 1 state
       *[other] { $states } states
    }
methods-owners-pick = owners' pick today
methods-needs = needs { $tech }
methods-not-yet = not available yet
methods-quantity = { $n } { $what }
methods-inputs = Uses: { $v }
methods-outputs = Makes: { $v }
methods-jobs = Employs: { $v }
methods-profit = Owners' profit per level: { $v } a year
economy-state-firms = State firms: { $levels } levels (their profit and loss go to the treasury)
economy-state-firm-name = { $building } · state
methods-who = Who picks the method:
methods-rule = Historical rule (the owners)
methods-fixed = Fixed by the government
methods-fixed-later = The government cannot fix the method yet; the historical rule holds for now.
