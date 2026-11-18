# Armed forces (phase 5, slice A).

gov-tab-forces = Forces
forces-none = No armed forces loaded.
forces-basis = Strength, generation, equipment and training of each type, read once a month. Equipment is made good little by little with what the state buys; with room left in the military budget, part of the force moves to the next generation each week and the rest of the budget buys equipment for the war reserve, in store, which formations short of equipment draw on first (reservists called up, a newer generation). Training rises week by week.
forces-summary = Summary
forces-men = Men in service
forces-reserve = Trained reserve
forces-share = Peace size: share of the men of service age (set in January)
forces-goods = Goods for the forces
forces-goods-value = { $week } a week ({ $gdp } of GDP a year)
forces-peace = In peace: the reserve stays home.
forces-mobilizing = Mobilizing: { $reported } of { $total } reservists reported.
forces-units = Types
forces-col-unit = Type
forces-depot = War reserve in store
forces-col-generation = Generation
forces-col-domain = Domain
forces-col-size = Elements
forces-col-men = Men
forces-col-equipment = Equipment
forces-col-training = Training
forces-called-up = { $unit } (reservists)
forces-decision-hint = Mobilizing is a decision of the journal, in Agendas, with a confirmation step. Disbanding takes whole elements of a type: pick how many and confirm.
forces-disband = Disband…
forces-disband-title = Disband: { $unit }
forces-disband-count = Elements
forces-disband-count-tip = Elements to disband: 1,000 men on land, one ship, one aircraft, one weapon
forces-disband-reserve = Men who join the reserve
forces-disband-home = Men who go home
forces-disband-depot = Back to the depot
forces-disband-good = { $good } { $value }
forces-disband-goods-more = { $goods } and { $count ->
    [one] { $n } more good
   *[other] { $n } more goods
}
forces-disband-no-goods = No equipment: nothing goes back to the depot.
forces-disband-note-peace = The peace army comes back to its size in peace in January: disbanding now eases the budget until then. To be smaller for good, change the military service law.
forces-disband-note-raised = The men ordered join the reserve, with the training they had; their equipment stays in the depot for the next order.
forces-disband-note-service = They leave service and do not come back: what can be used of them goes to the depot. A new order takes months.
forces-disband-waiting = Disband sent; the table updates next week.
forces-disband-yes = Disband
forces-disband-no = Keep
forces-domain-land = Land
forces-domain-sea = Sea
forces-domain-air = Air
forces-domain-strategic = Strategic

journal-group-forces = Armed forces
journal-forces-title = Armed forces in peace
journal-forces-mobilizing-title = Mobilization under way
journal-forces-cause = The army in peace follows the men of service age and the conscription law. Mobilizing calls up the reserve, which reports over { $weeks } weeks and leaves work.
journal-force-in-service = Men in service
journal-force-reserve = Reserve still at home
journal-force-reported = Reservists reported
journal-forces-progress = { $reported } of { $total } reported
journal-open-forces = Open the forces
journal-decision = Decision
journal-mobilize = Mobilize the reserve…
journal-mobilize-confirm = Confirm the mobilization
journal-mobilize-explain = The reserve leaves work and reports little by little; output feels the loss. A restore point is saved first.
journal-demobilize = Send the reserve home…
journal-demobilize-confirm = Confirm the demobilization
journal-demobilize-explain = The reservists go back home and to work.
journal-decision-cancel = Cancel

unit-irregular = Militia and guerrillas
unit-garrison = Garrison
unit-infantry = Infantry
unit-light_infantry = Light infantry
unit-cavalry = Cavalry
unit-artillery = Artillery
unit-engineers = Engineers
unit-anti_air = Anti-aircraft
unit-armor = Armour
unit-mechanized = Motorized and mechanized
unit-airborne = Airborne
unit-battleship = Battleships
unit-carrier = Aircraft carriers
unit-cruiser = Cruisers
unit-destroyer = Destroyers
unit-submarine = Submarines
unit-escort = Escorts
unit-coastal = Coastal defence
unit-mine_warfare = Mine warfare
unit-transport = Transports
unit-fighter = Fighters
unit-attack = Attack and tactical bombers
unit-strategic_bomber = Strategic bombers
unit-patrol = Reconnaissance and patrol
unit-air_transport = Air transport
unit-naval_aviation = Naval aviation
unit-helicopter = Helicopters
unit-drone = Drones
unit-ballistic_missile = Ballistic missiles
unit-missile_submarine = Missile submarines
unit-nuclear_warhead = Nuclear warheads
unit-satellite = Satellites
unit-missile_defense = Missile defence

# Orders of forces, in the forces tab.
orders-title = Orders
orders-rule = Orders are paid in queue order, after keeping the existing forces supplied; when the military budget is not enough, they wait.
orders-room = Left in the military budget this week, once the forces are supplied: { $room }
orders-none = No orders.
orders-generation = { $unit }, generation { $generation }
orders-colonial-unit = { $unit } (colonial)
orders-delivered = { $done } of { $ordered } delivered
orders-raising = { $share } of the men have reported; their training comes after
orders-building = { $share } of the equipment built; they join whole, at the end
orders-weeks = delivered { $weeks } weeks after the order, expected { $month } { $year }
orders-waiting = { $count ->
    [one] Waiting for room in the military budget for { $weeks } week
   *[other] Waiting for room in the military budget for { $weeks } weeks
}
orders-cancel = Cancel
orders-cancel-explain = What was built goes to the depot; the men already called up stay.
orders-cancel-yes = Cancel the order
orders-keep = Keep it
orders-new = New order
orders-nothing = Nothing to order yet.
orders-pick = Pick a type to see what it costs.
orders-count = Elements
orders-count-tip = Elements: 1,000 men on land, one ship, one aircraft, one weapon
orders-amount-men = { $men } men
orders-amount-ships = { $count ->
    [one] { $n } ship
   *[other] { $n } ships
}
orders-amount-aircraft = { $count ->
    [one] { $n } aircraft
   *[other] { $n } aircraft
}
orders-amount-weapons = { $count ->
    [one] { $n } weapon
   *[other] { $n } weapons
}
orders-colonial = Raised in the colonies
orders-cost-build = To build, at this week's prices
orders-cost-yearly = Every year once it serves
orders-cost-yearly-share = { $money } ({ $share } of the military budget)
orders-lead = Time to service
orders-lead-land = { $weeks } weeks for the men to report, then { $training } of training
orders-lead-build = { $weeks } weeks on the slipway or the line
orders-confirm = A large order: { $unit }, { $amount }. It costs { $build } to build and { $yearly } every year after.
orders-confirm-yes = Place the order
orders-confirm-no = Back
orders-place = Order

orders-yearly-parts = A year: { $pay } of pay and { $upkeep } of upkeep and wear; it stays as long as the force does.

## The war on land (slice C): the war window
cmd-window-war = War
    .short = War
war-title = War
war-off = The war on land is off in this game.
war-none = The country is not at war.
war-sides = { $ours } against { $theirs }
war-since-declared = Declared on { $date }, { $count ->
    [one] { $weeks } week
   *[other] { $weeks } weeks
} ago.
war-since-declared-on = Declared on us on { $date }, { $count ->
    [one] { $weeks } week
   *[other] { $weeks } weeks
} ago.
war-at-stake = At stake
war-support = Support for the war
war-support-soon = Each side's support decides peace and surrender; its value, what moves it and the revolution come with the next part of the simulation.
war-occupation = Occupation
war-taken = Held by us:
war-lost = Ours, held by them:
war-states-none = none
war-occupation-soon = What occupied land pays, the devastation and the garrison come with the next part of the simulation.
war-fronts = Fronts
war-fronts-basis = One front per enemy country, read from the last week of combat: segments, forces, losses and advance.
war-front = Front with { $country }
war-posture = Posture:
war-hold = Hold
war-attack = Attack
war-hold-note = Holding: the troops defend and fight only if the enemy attacks.
war-attack-note = Attacking: the troops advance on every segment, with losses on both sides.
war-front-none = No land border with this enemy this week.
war-both = { $ours } us · { $theirs } them
war-men = Men at the front
war-power = Firepower
war-week-losses = Losses this week
war-advance = Advance this week
war-advance-tip = Sum, in km, of what the front gained or lost on each segment this week; not a distance on the map. Each segment advances up to the depth of the next state, which then changes hands.
war-enemy-losses = Enemy losses since the start
war-our-losses = Our land losses since the start
war-km-gained = gained { $km } km
war-km-lost = lost { $km } km
war-km-none = still
war-segments-show = Show { $count ->
    [one] { $n } segment
   *[other] { $n } segments
}
war-segments-hide = Hide { $count ->
    [one] { $n } segment
   *[other] { $n } segments
}
war-segment = { $ours } – { $theirs }
war-quiet = no fighting
war-segment-line = Power { $ours } us, { $theirs } them · losses { $lost } us, { $killed } them · { $advance }
war-terms-ours = Our terms:
war-terms-theirs = Their terms:
war-term = { $term }: { $value }
war-term-number = Number (thousands of men)
war-term-equipment = Equipment
war-term-training = Training
war-term-supply = Supply
war-term-terrain = Terrain
war-term-air = Air
war-into-theirs = We advanced { $share } into their state; at 100 % it passes to us.
war-into-ours = They advanced { $share } into our state; at 100 % it passes to them.
war-peace = Propose peace to { $country }…
war-peace-note = Peace is an agreement at the negotiation table, in the Agendas; signed between the leaders, the war ends and the occupation between them ends.
