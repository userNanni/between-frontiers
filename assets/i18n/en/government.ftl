## Government window: budget, money and debt. Amounts in dollars at 1925 base prices.

gov-pct = { $value }%
gov-pct-gdp-year = { $value }% of GDP/year
gov-no-economy = This country has no simulated economy.
gov-pending = Paused: changes take effect when time runs again.
gov-budget-basis = Last week's accounts × 52, in dollars at 1925 base prices. Hover a number to see the sum behind it.
gov-money-basis = Rates and inflation in % a year; indices with 1925 = 100. Hover a number to see the sum behind it.

## Budget
gov-revenue = Revenue per year
gov-income-tax = Income tax
gov-tip-income-tax = The three classes' income tax, the lines above.
gov-indirect-tax = Tax on consumption
gov-cap-class = { $class } { $cap }
gov-tip-indirect =
    The taxation law in force caps the income tax: { $caps }.
    What the caps keep the rule from charging (the mean cut of the three classes) is charged on what people buy, up to { $ceiling }.
    Rate on consumption: { $rate }. Last week: { $week }.
gov-state-firms = Profit of state firms
gov-tip-state-firms =
    The state owns { $ownership } of industry; { $paid } of what that share earns goes to the treasury, by the ownership law in force.
    Last week: { $week }.
gov-revenue-total = Total revenue
gov-class-0 = Lower class
gov-class-1 = Middle class
gov-class-2 = Upper class
gov-class-0-members = peasants, farm labourers, workers, clergy and soldiers
gov-class-1-members = skilled workers, clerks, engineers, professionals and civil servants
gov-class-2-members = managers, capitalists and landowners
gov-tip-class-tax =
    { $people } people: { $members }.
    Income after tax: { $income } a year.
    Rate: { $rate }. Tax = income × rate ÷ (1 − rate) = { $tax }.
gov-tip-revenue =
    Revenue last week, every source: { $week } × 52 = { $year } ({ $gdp } of GDP).
    Quarter's average: { $avg } a year.
gov-spending = Spending per year
gov-spending-total = Total spending
gov-spend-0 = Payroll
gov-spend-1 = Purchases
gov-spend-2 = Schools, hospitals and research
gov-spend-3 = Public works
gov-spend-4 = Interest
gov-spend-research = …of which research
gov-tip-payroll =
    { $workers } civil servants, clergy and soldiers, paid { $pay } of the country's average wage times their profession's weight.
    An obligation: paid even with the treasury in the red.
gov-tip-goods =
    What the state buys, at base prices: services 30%; transport 10%; paper 10%; medicine 10%; small arms 10%; ammunition 10%; warships 10%; artillery 5%; military aircraft 5%.
    This week's rule, per year: average revenue { $revenue } + the deficit the debt rule allows { $room } − payroll, schools and hospitals, works and interest { $committed } = { $planned }.
    At least 60% of the payroll: { $floor }. { $kept } of the plan is kept (cut with negative cash or interest above 10% of revenue) = { $total }.
gov-tip-services = The state buys everything schools, hospitals and laboratories produce; it becomes the pay of teachers, doctors and staff.
gov-tip-research = Laboratories: the treasury pays { $state } a year ({ $share } of the cost, by the ownership law) and the firms pay { $firms } out of their fund; { $gdp } in all. How much to research comes from the value industry adds and from state revenue times the weight of "Research" among public works.
gov-tip-works = Construction points paid to the public works in the queue as they arrive.
gov-tip-interest =
    Interest on debt held privately and abroad ({ $cost } a year on { $debt } of debt).
    The central bank's interest goes back to the treasury and is left out.
gov-tip-yearly = Last week: { $week }; × 52 = { $year } ({ $gdp } of GDP).
gov-tip-spending = { $gdp } of GDP.
gov-balance = Balance
gov-amount-gdp = { $amount } · { $gdp } of GDP
gov-tip-deficit =
    Revenue { $revenue } − spending { $spending } = deficit of { $balance } ({ $gdp } of GDP).
    The treasury sells bonds to cover it; the central bank bought { $bank } of them.
gov-tip-surplus =
    Revenue { $revenue } − spending { $spending } = surplus of { $balance } ({ $gdp } of GDP).
    What passes the reserve becomes new public works.
gov-treasury = Treasury cash
gov-tip-treasury =
    Cash: { $cash }. Reserve: { $weeks } weeks of average revenue ({ $avg } a week) = { $reserve }.
    The bar shows cash against the reserve; what passes it becomes public works.
gov-issued = Bonds sold to the central bank
gov-tip-issued = New money the central bank lent the treasury by buying bonds, last week × 52. Only a subordinate bank does this.

gov-taxes = Income tax
gov-tax-rule = Historical rule
gov-tax-fixed = Set by the government
gov-tip-tax-rule =
    Rule for { $year }: { $base } × fiscal correction { $adjust } = { $rule }.
    The rule rises in a straight line from 1925 to 1975. Ceiling of { $max } per class.
gov-tax-adjust = Fiscal correction of the rule: × { $adjust } (rises when debt costs too much, falls when cash piles up).

gov-works = Public works
gov-cat-0 = Infrastructure ({ $share })
gov-cat-1 = Education ({ $share })
gov-cat-2 = Health ({ $share })
gov-cat-3 = Urban housing ({ $share })
gov-cat-4 = Research ({ $share })
gov-tip-priority =
    Weight { $weight } of { $sum } = { $share } of the money for new works (default: { $default }).
    In the queue: { $works } works of this kind, { $done } of their points delivered.
gov-works-stopped = Every weight at zero: no new public works.
gov-queue = Queue: { $public } public works, { $private } private
gov-tip-work = { $done } of { $points } construction points delivered.
gov-queue-more = and { $count } more public works.

gov-migration = Migration law
gov-migration-open = Open
gov-migration-restricted = Internal passports
gov-migration-open-tip = Anyone may move to the city that pays best.
gov-migration-restricted-tip = Only 1/4 of the flow within a state and 1/10 between states get through.

## Money and debt
gov-prices = Prices and exchange
gov-inflation = Inflation
gov-tip-inflation-subordinate =
    Now { $now } a year; last year { $last }; expected { $expected }.
    With a subordinate bank, expectations follow the past and the world, and the { $target } target only a little.
    Money created beyond the rule: { $monetized } of GDP a year (each point adds 1.5 points of inflation).
gov-tip-inflation-autonomous =
    Now { $now } a year; last year { $last }; expected { $expected }.
    Autonomous bank: expectations stay anchored on the { $target } target.
    Money created beyond the rule: { $monetized } of GDP a year (each point adds 1.5 points of inflation).
gov-price-level = Price level (1925 = 100)
gov-tip-price-level =
    On January 1st: { $jan }.
    Currency reforms (three zeros cut): { $reforms }.
gov-exchange = Exchange rate (1925 = 100)
gov-tip-exchange =
    Local currency per reference unit: { $rate }. Purchasing-power parity: { $parity }.
    Real exchange rate = rate ÷ parity × 100 = { $real } (above 100: a cheap currency).
    The rate moves 1/26 a week toward parity, weakened by the risk premium.
gov-central-bank = Central bank
gov-policy-rate = Policy rate
gov-tip-rate-fixed = Set by the government at { $rate } a year.
gov-tip-rate-taylor =
    Taylor rule: real rate { $real } + inflation { $inflation } = neutral { $neutral };
    + half the inflation above the { $target } target: { $infl_term };
    + half the plants' utilization gap: { $gap_term };
    = { $rate } (floored at zero).
gov-regime = Regime in force
gov-regime-subordinate = Subordinate to the treasury
gov-regime-autonomous = Autonomous
gov-tip-regime-historical = Historical rule: subordinate until 1990, or until a stabilization plan after inflation above 200%.
gov-tip-regime-historical-since = Historical rule: autonomous since { $year }.
gov-tip-regime-autonomous = Chosen by the government: follows its rule and buys no new treasury bonds.
gov-tip-regime-subordinate = Chosen by the government: buys the bonds nobody else does.
gov-credit = Credit to banks
gov-tip-credit =
    Money the central bank created and lent to banks, which put it in the firms' investment pool: { $gdp } of GDP.
    It matters because it pays for plants and private works without drawing on household savings; withdrawn, investment falls.
gov-monetized = Money beyond the rule
gov-tip-monetized =
    Money the central bank created beyond its rule: treasury bonds it bought because nobody else would, or a rate set below the rule. Last year's average, in % of GDP.
    It matters because each point adds 1.5 points of inflation; bonds bought within the rule do not count here.
gov-bank-controls = Monetary policy
gov-bank-historical = Historical
gov-bank-autonomous = Autonomous
gov-bank-subordinate = Subordinate
gov-rate-rule = Taylor rule
gov-rate-fixed = Fixed rate
gov-tip-rate-step = Half a point per click. Changing the rate fixes it; go back to the rule with the selector above.

gov-debt = Public debt by holder
gov-debt-home = At home
gov-debt-bank = Central bank
gov-debt-abroad = Abroad, local currency
gov-debt-fx = Abroad, in dollars
gov-tip-debt-home = The country's savers and banks: { $amount } ({ $gdp } of GDP, { $share } of the debt).
gov-tip-debt-bank = Bonds at the central bank: { $amount } ({ $gdp } of GDP, { $share } of the debt). Its interest goes back to the treasury.
gov-tip-debt-abroad = Foreigners, in the country's currency: { $amount } ({ $gdp } of GDP, { $share } of the debt). Inflation eats it.
gov-tip-debt-fx =
    In foreign currency: { $fx } in reference units × real exchange rate { $real } ÷ 100 = { $amount } ({ $gdp } of GDP, { $share } of the debt).
    It does not shrink with the country's inflation and grows when the currency loses value.
gov-debt-total = Total
gov-debt-total-value = { $amount } · { $gdp } of GDP
gov-tip-debt-total = The four holders against last year's GDP ({ $gdp_value }) = { $gdp }.
gov-interest = Interest
gov-interest-paid = Interest paid per year
gov-tip-interest-paid =
    Last week: { $week } × 52.
    { $revenue } of revenue, { $gdp } of GDP.
gov-bond-rate = New bond, local currency
gov-tip-bond-rate = Half the central bank's rate + half the world rate corrected for the country's inflation, + the risk premium.
gov-fx-rate = New bond, in dollars
gov-tip-fx-rate = World rate + the risk premium.
gov-cost-local = Average cost of the stock, local
gov-cost-fx = Average cost of the stock, dollars
gov-tip-cost = The cost of old debt follows the market rate over about five years.
gov-defaults = Defaults
gov-defaults-count = Defaults so far
gov-tip-defaults = History the risk premium reads: { $history } (loses 10% a year; each default adds 1).
gov-foreign-credit = Foreign credit
gov-excluded-until = Closed until { $year }
gov-tip-excluded = After a default, the country sells no bonds abroad for three years.
gov-credit-open = Open
gov-tip-credit-open = Foreigners buy the country's bonds.
gov-foreign-assets = Bonds of other countries
gov-tip-foreign-assets = What the country's savers lent to other states.
gov-waiting = The accounts show once the first day has passed.
gov-no-player = No country to govern in this campaign.

gov-campaign = Adult literacy campaign
gov-campaign-offer = { $illiterate } adults aged 15 to 49 cannot read. A five-year campaign enrols { $enrolled } a year, and { $readers } of them keep reading for good. It costs { $cost } a year.
gov-campaign-running = Under way, { $years } years left. At the last school year, { $taught } adults learned to read. It costs { $cost } a year.
gov-campaign-note = Half drop out, half of those who stay pass, and some forget without practice (Abadzi 2003). It takes effect at the next school year, in January.
gov-campaign-start = Start a campaign…
gov-campaign-stop = Stop the campaign…
gov-campaign-confirm-start = Start the campaign? It costs { $cost } a year for five years.
gov-campaign-confirm-stop = Stop the campaign? Those who learned keep reading.
gov-campaign-yes = Confirm
gov-campaign-no = Cancel
