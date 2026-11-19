## Agendas window (J) and the side panel on the map.

bar-journal = Agendas
journal-title = Agendas
journal-not-ready = The agendas fill in once the campaign has run its first day.
journal-empty = Nothing under way: no queue, research, fiscal alert or movement to follow.
journal-pin = Pin to the panel
journal-unpin = Unpin
journal-pinned = pinned
journal-unknown = not measured by the simulation yet
journal-measured = now { $value }
journal-limit-at-least = needs ≥ { $value }
journal-limit-at-most = needs ≤ { $value }
journal-weeks = { $weeks } wk
journal-enter = WHAT OPENS IT
journal-milestones = MILESTONES
journal-forces = FORCES
journal-exit = WHAT CLOSES IT
journal-completes = WHEN IT COMPLETES
journal-open-budget = Open the budget
journal-open-money = Open money and debt
journal-open-laws = Open the laws
journal-open-research = Open research
journal-group-politics = Politics
journal-group-fiscal = Treasury and money
journal-group-works = Works
journal-group-research = Research

journal-works-title = Construction queue
journal-works-cause = Every new level, public or private, waits in one national queue for construction points from the building trade. Works take points in queue order, each at most a twenty-sixth of its cost a week.
journal-works-enter = Works in the queue
journal-works-exit = Queue emptied
journal-works-progress = { $public } public · { $private } private
journal-works-completes = Each finished level starts running and the next works move up.
journal-work-line = { $building } · { $state }
journal-force-points = Construction points delivered per week
journal-bottleneck-builders = Builders met only { $met } of the demand for construction
journal-bottleneck-price = Construction costs { $price } of its base price
journal-bottleneck-input = { $good } short: { $met } of the demand met

journal-large-title = { $building } in { $state }
journal-large-cause = The largest public work in the queue: { $points } construction points.
journal-large-enter = Place in the queue
journal-large-points = Construction points delivered
journal-large-exit = Points still missing
journal-force-ahead = Works ahead of it in the queue
journal-force-min-weeks = Fastest a work can be finished
journal-weeks-left = about { $weeks } to finish at last week's pace
journal-weeks-beyond = more than { $weeks } at last week's pace
journal-waiting = waiting: earlier works take every point
journal-large-completes = The { $building } opens and the state pays no more for it.

journal-research-title-focus = Researching { $tech }
journal-research-title-auto = On its way: { $tech }
journal-research-cause-focus = The technology the government chose to research.
journal-research-cause-auto = No focus chosen: the country spreads its research by itself. These are the prototypes closest to industry.
journal-research-enter-focus = A focus is chosen
journal-research-enter-auto = No focus chosen
journal-research-milestone = { $tech } → { $stage }
journal-research-exit = Reaches the { $stage } stage
journal-research-progress = toward { $stage }
journal-research-completes = The technology reaches the { $stage } stage; at the industrial stage its methods start to raise productivity.

journal-cond-cash = Cash in the treasury
journal-cond-kept = Share of the planned purchases kept
journal-cond-room = Deficit room the debt leaves
journal-cond-defaults = Defaults so far
journal-cond-year = Year
journal-cond-inflation = Inflation
journal-cond-measure = { $measure }

journal-force-balance = Last week's balance, a year at that pace

journal-unfunded-title = Treasury in the red
journal-unfunded-cause = Nobody bought the bonds the treasury sold: the cash went below zero. Purchases fall with the hole and the tax rule steps up each year while it lasts.
journal-unfunded-completes = The treasury borrows again and the cuts stop.
journal-force-kept = Purchases kept by the budget rule
journal-force-tax-rule = Fiscal correction of the tax rule

journal-purchases-title = Purchases cut
journal-purchases-cause = The budget rule cuts the state's purchases when the treasury could not borrow or when interest takes more of the revenue than the rule tolerates.
journal-purchases-progress = { $kept } of the planned purchases kept
journal-purchases-completes = The state buys its full plan again.
journal-force-interest = Share of revenue taken by interest

journal-debt-title = Debt closing the credit
journal-debt-cause = The debt rule lets the state borrow for purchases while debt and rates stay low, narrowing the room as they rise. This entry opens once the debt has taken half the room a country without debt would have.
journal-debt-closed = No room left: the state borrows nothing for purchases
journal-debt-progress = { $room } of the room left
journal-debt-completes = Purchases rest on revenue alone; the deficit stops growing.
journal-force-debt-gdp = Debt over GDP
journal-force-bond-rate = Rate on new bonds

journal-shutout-title = Shut out of foreign lending
journal-shutout-cause = After a default foreign lenders stay away for years and the risk premium stays high ({ $defaults } so far).
journal-shutout-back = Foreign lenders come back in { $year }
journal-shutout-completes = Foreign savers buy the bonds again.
journal-force-default-history = Default history the premium reads

journal-inflation-title = Inflation above the target
journal-inflation-cause-subordinate = Prices rise faster than the central bank's target. The bank answers to the treasury and buys the bonds nobody else does.
journal-inflation-cause-autonomous = Prices rise faster than the central bank's target. The autonomous bank raises its rate to lean against it.
journal-inflation-completes = Inflation back at the target.
journal-force-policy-rate = Central bank rate
journal-force-monetized = Money created beyond the rule
journal-force-expected = Expected inflation

journal-movement-title = Movement: { $movement }
journal-movement-cause = { $desc }
journal-movement-progress = { $support } support
journal-movement-exit = Cause gone: support halves every { $years } years
journal-goal-law = Enact { $law }
journal-goal-independence = Independence
journal-goal-revision = Win back lost territory and standing
journal-force-pressure = Pressure of the cause
journal-force-growth = Growth a year at full pressure
journal-force-weakest = Weakest condition: { $measure }

journal-pending-title = Law requested: { $law }
journal-pending-cause = The government asked for this law; the politics system enacts it.
journal-pending-enter = Asked for
journal-pending-exit = Enacted
journal-pending-completes = { $law } in force.

journal-changes-title = Law changes
journal-changes-cause = Laws changed in this campaign. Legitimacy: { $legitimacy }.
journal-changes-enter = Changes
journal-change-line = { $from } → { $to }
journal-force-legitimacy = Legitimacy

outliner-title = Overview
outliner-journal = Agendas →
outliner-works = Works
outliner-works-count = { $public } public · { $private } private
outliner-works-none = The queue is empty.
outliner-weeks = ~{ $weeks } wk
outliner-weeks-beyond = > { $weeks } wk
outliner-waiting = waiting
outliner-work-tip-public = Public work, { $done } built. Click for the budget and the whole queue.
outliner-work-tip-private = Private work, { $done } built. Click for the budget and the whole queue.
outliner-research = Research
outliner-research-focus = focus
outliner-research-auto = no focus
outliner-research-none = Nothing on the way.
outliner-pinned = Pinned agendas
outliner-pinned-none = Pin agendas from the Agendas window (J).
outliner-pinned-closed = An agenda you pinned has closed.

journal-difference = { $value } apart
journal-agendas = Agendas
journal-agendas-none = No fiscal alert, movement or law under way now.
journal-followed = Followed
journal-followed-hint = Works and research: progress to follow, without a condition that closes them.
journal-work-line-many = { $building } ×{ $count } · { $state }
journal-work-line-states = { $building } in { $states } states
journal-noworks-title = No money for new public works
journal-noworks-cause = Each month the state starts public works with the cash above its reserve of { $weeks } weeks of revenue, less what the works in the queue still cost, plus what the debt rule lets it borrow for works. That sum is at or below zero: the queue gets no new public works.
journal-cond-works-budget = Money for new public works
journal-force-over-reserve = Cash above the reserve of { $reserve }
journal-force-committed = Still owed on public works in the queue
journal-force-borrowing = Borrowing the debt rule allows for works
journal-noworks-completes = Public works start again, by the weights of the budget.
journal-bottleneck-cash = No money for new public works: cash above the reserve does not cover the queue
outliner-research-focus-tip = The government chose this technology in the Research screen (R).
outliner-research-auto-tip = No focus: the country spreads its research effort by itself. Choose a focus in the Research screen (R).

journal-revolt-title = A country born in revolt: { $country }
journal-revolt-cause = { $owner } refused independence and { $country } proclaimed itself without an agreement. A colonial war has begun: the metropole wants the union back and fights with its garrison and the men it lands; the new country fights with irregulars, armed little by little.
journal-revolt-irregulars = Irregular combatants
journal-revolt-armed = Of them, armed
journal-revolt-landed = Men landed by the metropole
journal-revolt-switch = To play the new country or the metropole, switch countries in the campaign's “…” menu, on the Campaigns screen.

journal-programme-title = Nuclear programme
journal-programme-cause = { $share } of GDP a year for the Project: the money moves fission, enrichment and the bomb ahead, and then keeps the warhead order going, which depends on the fissile material on the market. The controls are in the Forces tab.
journal-programme-progress = { $done } of { $total } steps in production
journal-programme-warheads = Warheads ready
journal-programme-months = Months of the step under way, at last month's pace
journal-programme-end = End the programme…
journal-programme-end-confirm = Confirm the end of the programme
journal-programme-end-explain = The money stops at once; the research already done stays, the warheads built stay, and the plants keep running for the market. A restore point is saved first.
