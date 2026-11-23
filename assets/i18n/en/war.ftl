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
war-advance-tip = Mean, in km, of what the front gained or lost over its segments this week; in brackets, the segment that moved the most. Each segment advances up to the depth of the next state, which then changes hands.
war-enemy-losses = Enemy losses since the start
war-our-losses = Our land losses since the start
war-advance-value = { $mean } on average (up to { $most } km on one segment)
war-overrun = People under the enemy
war-overrun-value = { $share }, of { $threshold } that bring the government down
war-overrun-capital-taken = The capital is taken: the threshold falls from { $plain } to { $capital }.
war-overrun-capital-held = The capital is free; taken, the threshold falls from { $plain } to { $capital }.
war-overrun-fall = Past the threshold, the government falls at once and the new government withdraws from the war.
war-overrun-colony = A colony still free holds the government: while there is one, occupation does not bring it down.
war-overrun-ally = An ally still fights: while it does, occupation does not bring the government down.
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
war-into-theirs = We advanced { $share } into their state; at 100% it passes to us.
war-into-ours = They advanced { $share } into our state; at 100% it passes to them.
war-segment-density = Front of { $width } km · { $ours } men per km us, { $theirs } them · full line: { $full } per km
war-segment-density-tip = The denser the defender, the slower the attacker advances: against a full line, at the base speed; against a line five times as dense, 25 times slower; against a thin line, up to 1.75 times faster. The width is the border between the two states.
war-segment-dug = Dug in { $share }: still for { $weeks ->
    [one] { $weeks } week
   *[other] { $weeks } weeks
}; full at { $full } weeks, when the defender's terrain counts double.
war-peace = Propose peace to { $country }…
war-peace-note = Peace is an agreement at the negotiation table, in the Agendas; signed between the leaders, the war ends and the occupation between them ends.
war-support-thresholds = On the bar, the light tick is peace and the red one, collapse. An AI government below { $peace } asks for peace; below { $collapse }, it counts down to its fall.
war-support-you = { $country } (you)
war-support-tends-down = Support tends to fall to { $target }, from:
war-support-tends-up = Support tends to rise to { $target }, from:
war-support-tends-still = Support tends to stay at { $target }, from:
war-support-why-start = cause of the war
war-support-why-casualties = the dead
war-support-why-duration = a long war
war-support-why-deprivation = hardship at home
war-support-why-territory-lost = land lost
war-support-why-territory-won = land taken
war-support-why-result-down = a front falling back
war-support-why-result-up = a front moving forward
war-support-pace = It moves slowly: { $count ->
    [one] { $weeks } week
   *[other] { $weeks } weeks
} to go half the way
war-support-censored = censorship slows the change
war-support-censored-fall = censorship slows the fall
war-support-vote-wide = with a wide vote, support reacts faster
war-support-vote-narrow = with a narrow vote, opinion weighs in slowly
war-support-you-thresholds = For you: below { $peace } the AI would ask for peace, but you decide; below { $collapse }, the government falls in { $count ->
    [one] { $weeks } week
   *[other] { $weeks } weeks
} and the new one asks for peace.
war-devastation-ours = Our side
war-devastation-theirs = Their side
war-devastation-total = { $count ->
    [one] { $civilians } civilian killed
   *[other] { $civilians } civilians killed
} in { $n ->
    [one] { $regions } region
   *[other] { $regions } regions
}
war-devastation-more = and { $count ->
    [one] { $n } more region with a smaller loss
   *[other] { $n } more regions with smaller losses
}
war-support-term = { $term } { $value }
war-support-group = { $group } { $value } (weight { $clout })
war-support-speed-tip = The support moves toward its target little by little. A free vote makes opinion weigh in faster; the censorship of the laws in force slows it. Changing the press or the voting law changes this speed.
war-support-countdown = Below the collapse: the government falls in { $count ->
    [one] { $weeks } week
   *[other] { $weeks } weeks
}; until then, peace may still be made on its own terms.
war-support-fallen-regime = The government fell and another form of government came ({ $form }); the new government asks for peace. The security reputation was halved.
war-support-fallen-cabinet = The cabinet fell; the new government asks for peace.
war-occupied-line = { $state } — held by { $by } for { $count ->
    [one] { $weeks } week
   *[other] { $weeks } weeks
}; resistance { $resistance }, garrison { $garrison } of what is needed; takes { $extraction } of the output ({ $taken } a week; rising to { $cap })
war-occupied-tip = Resistance rises with what the occupier takes and with a short garrison; the garrison is the share kept of the troops the territory asks for. The occupier takes a share of the output that grows with the weeks up to the cap, cut when the garrison runs short.
war-devastation = Devastation
war-wreck-ours = { $state } (ours): { $share } of the buildings destroyed, { $count ->
    [one] { $civilians } civilian killed
   *[other] { $civilians } civilians killed
}
war-wreck-theirs = { $state } (theirs): { $share } of the buildings destroyed, { $count ->
    [one] { $civilians } civilian killed
   *[other] { $civilians } civilians killed
}

composition-title = Composition of the forces
composition-basis = Once a year the infantry changes equipment toward the target share on wheels and on tracks: at most 2% of the land men a year, paid from what the military budget leaves. The army does not grow: the men are the same, with the training they had.
composition-target-none = No target: the country does not make the infantry's equipment yet.
composition-wheels-now = On wheels today
composition-tracks-now = On tracks today
composition-target = Composition target
composition-rule-line = Historical rule: { $wheels } on wheels and { $tracks } on tracks, by what the country can already make.
composition-fixed-note = The government fixes both shares; the conversion keeps its pace, only the destination changes.
composition-step-tip = One percentage point per click; the two shares add up to at most 100%.
composition-converting = Converting to { $unit }
composition-converting-done = { $done } of { $of } thousand men
composition-stall-maker = Stalled for { $count ->
    [one] { $weeks } week
   *[other] { $weeks } weeks
}: the country cannot make this type's equipment yet.
composition-stall-room = Stalled for { $count ->
    [one] { $weeks } week
   *[other] { $weeks } weeks
}: the military ceiling of state purchases all goes to supplying the forces and the orders; nothing is left to convert.
composition-stall-infantry = Stalled for { $count ->
    [one] { $weeks } week
   *[other] { $weeks } weeks
}: no infantry is left at home to convert.
composition-stall-prices = Stalled for { $count ->
    [one] { $weeks } week
   *[other] { $weeks } weeks
}: what is left of the military budget does not pay for the vehicles at today's market prices.
composition-no-wheels = No type on wheels the country can equip: the target's part on wheels stays unconverted.
composition-no-tracks = No type on tracks the country can equip: the target's part on tracks stays unconverted.
composition-wheels = On wheels
composition-tracks = On tracks

nuclear-balance = Balance of deterrence
nuclear-warheads = { $n } warheads
nuclear-warheads-opaque = { $n } warheads, as you read an arsenal never tested
nuclear-umbrella = Under the umbrella of { $who }: their arsenals count on this side
nuclear-second-strike = After suffering the first strike, it still delivers { $delivered } warheads; the other holds { $needed } as unacceptable.
nuclear-secure = A secure second strike: the other cannot escape the answer.
nuclear-not-secure = No secure second strike: a first strike by the other leaves it without an answer that counts.
nuclear-first-strike = It gains by striking first: only it holds a secure second strike.
nuclear-ceiling = How far it can climb against the other: { $rung }.
nuclear-mutual = Mutual deterrence: direct war stays open, but carries { $risk }% a week of escalation risk; coercion has no credit.
nuclear-not-mutual = Deterrence is not mutual: each side's ceiling is above.
nuclear-opacity-note = An arsenal never tested counts { $share }% for the others; yours counts whole for you. Warheads fit on launchers (submarines, missiles, bombers); without a launcher, they do not count.
nuclear-rung-tension = tension
nuclear-rung-rivalry = rivalry
nuclear-rung-arms_race = arms race
nuclear-rung-crisis = crisis, no war
nuclear-rung-war = conventional war
nuclear-rung-coercion = nuclear coercion
nuclear-rung-use = use of the weapon

war-expedition = Expedition to { $country }
war-expedition-by-land = The colony is reached by land: the army gets there without landing and without a cap.
war-expedition-landed = Men landed
war-expedition-goal = Expedition goal
war-expedition-ordered = The goal is the government's. The men land, or come back, through the ports, at each week's pace.
war-expedition-landed-value = { $landed } of at most { $cap } (half the land men)
war-expedition-rule-line = Historical rule: { $goal } men, enough to cover 20 per thousand of the people you hold in the unit (people hostile to the occupation). On the bar, the tick is the goal.
war-expedition-rule-line-fixed = Historical rule: enough men to cover 20 per thousand of the people you hold in the unit (people hostile to the occupation).
war-expedition-men = Men to keep there
war-expedition-step-tip = Ten thousand men a click, up to half the land men.
war-expedition-tied = Tied in the unit
war-expedition-tied-value = { $men } ({ $share }% of the land army)
war-expedition-armed = The enemy's armed irregulars
war-expedition-note = More men cover the unit better (the guerrilla loses more for each of your casualties), but they tie the army and weigh more on the war's weariness, in proportion to the army committed.
composition-apply = Apply
army-heading = Army generation
army-basis = A second-generation army fights on a linear front, by fire and attrition, commanded from the centre; a third-generation one acts on the commander's intent, concentrates on one point, breaks through and exploits. The generation comes from the readiness, summed term by term and read at the start of each month.
army-second = 2nd generation · readiness { $total } of 1,000; moves to the 3rd at { $up } ({ $short } to go)
army-third = 3rd generation · readiness { $total } of 1,000; falls back to the 2nd below { $down }
army-war-tip = The generation our army fights with. In the 3rd, the front puts six tenths of its free men on the weakest segment, sends the mobile troops there and breaks through at three times the power. The way there is in the Government's Forces tab.
army-points = { $points } of { $most }
army-points-taken = −{ $points } of { $most }
army-term-officer_education = Officers trained to act on intent
army-term-training = Training
army-term-radio = Radio
army-term-motorization = Motorization
army-term-experience = Experience of war
army-term-central_control = Political control of the officers
army-do-officer_education = A class of officers a year leaves trained in what the school teaches, as far as the training goes.
army-do-training = Keep the army trained; new recruits dilute the training.
army-do-radio = The military budget's surplus replaces equipment with the new generation, which brings radio.
army-do-motorization = Order mechanized and armoured formations.
army-do-experience = It comes only from war; half of it fades every five years.
army-do-central_control = Changing the form of government puts the officer corps under politics for five years.
army-hold-school = The school does not teach the doctrine yet: armoured warfare is at { $stage }. Researching it to the industrial stage makes the new classes leave trained in it.
army-hold-tradition = No tradition on record: it counts half the people's literacy. Teaching people to read raises this term.
army-hold-training = Training incomplete: new recruits dilute it.
army-hold-equipment = No formation in equipment newer than 1925's; the military budget's surplus modernizes it little by little.
army-hold-motorized = No formation on wheels or tracks; order mechanized or armoured ones in New order.
army-hold-experience = No experience of war: this term only comes from war.
army-hold-regime = The form of government changed: the officer corps answers to politics for { $count ->
    [one] { $years } more year
   *[other] { $years } more years
}.

war-expedition-cut = Landing stopped: the route { $from }–{ $to } is cut at { $zone }, a zone the enemy controls.

war-fleet = Fleet
war-fleet-blockading = Stronger on the surface than { $country }: it holds the way out of its sea and cuts the trade that passes there.
war-fleet-home = Weaker on the surface than { $country }: it stays at its own way out to sea, a fleet in being, without seeking battle.
war-fleet-home-alone = The enemy has no fleet: yours stays at its own way out to sea.
war-fleet-ships = Ships
war-fleet-zone = Zone
war-fleet-zone-value = { $zone } · mission: control
war-fleet-week = At the end of the week
war-fleet-holds-short = the zone is yours
war-fleet-contested-short = contested: the enemy has more power there
war-fleet-blockade-head = Under our blockade: { $countries } · { $count ->
    [one] { $pairs } pair cut
   *[other] { $pairs } pairs cut
}
war-fleet-pair = { $a } and { $b }
war-fleet-pair-value = { $value } a year
war-fleet-pairs-more = and { $count ->
    [one] { $n } more pair
   *[other] { $n } more pairs
}
war-fleet-blockade-note = The value is what the pair traded in the last year, both ways, before the cut. Pairs by land, and routes that do not pass the zones you control, stay open.
war-blockaded = Blockade against us
war-blockaded-basis = The enemy fleet cuts { $count ->
    [one] { $pairs } pair
   *[other] { $pairs } pairs
} of our sea trade, worth { $total } a year. By land, trade goes on.
war-fleet-command-later = The command of the fleet (the mission by zone) comes with the next slice; until then the fleet follows the rule above.
programme-title = Nuclear programme
programme-basis = A share of GDP a year, outside the military ceiling. Until the bomb, the money buys the laboratory and moves fission, enrichment and the bomb ahead; after it, three quarters go to the warhead order, which grows as far as that money buys; each warhead takes a weapon core and an assembly, and they join one by one.
programme-start = Start the programme (0.1% of GDP a year)
programme-share = Share of GDP a year
programme-step-tip = A tenth of a point of GDP a year
programme-cost = Cost a year at this share
programme-spent = Delivered so far
programme-stage-unknown = not yet studied ({ $progress })
programme-stage-science = understood; prototype at { $progress }
programme-stage-prototype = prototype; industry at { $progress }
programme-stage-industrial = in production
programme-stage-diffused = diffused
programme-months = At last month's pace, the step under way ends in about { $months } months.
programme-months-none = No credit last month yet, or the whole chain is in production.
programme-warheads = Warheads
programme-warheads-value = { $n } ready, { $ordered } on order
programme-fissile = Weapon cores on the market
programme-fissile-value = { $price } a unit; the market delivered { $delivered } of what was asked
programme-end = End the programme
programme-note = The cores come from the country's core plants and from the market. The plant makes cores by high enrichment or by a production reactor (which needs the reactor technology, a programme's only); both run on uranium ore, from the country's mines, the market's stock or abroad.

programme-test = Test the weapon
programme-test-note = The test makes the arsenal known to all: until it, the others count half your warheads. The whole world hears it, and it cannot be undone.
programme-test-asked = Test set: it happens next week.
programme-tested = Weapon tested: the arsenal is known to all.

# The use of the nuclear weapon (step 4; N5, N6, N7)
strike-title = Nuclear use
strike-counterforce = Counterforce
strike-countervalue = Countervalue
strike-warheads = Warheads
strike-warheads-tip = How many warheads to launch, up to what the launchers carry
strike-ready = Warheads ready on launchers
strike-dead = Estimated dead, if all arrive
strike-men = Men of the enemy's land forces put out of the war
strike-stain = The security reputation loses the equivalent of { $years } years of our forces' cost.
strike-answer = If the enemy has warheads ready, it answers the next week on the same rung and with the same number; without land forces, with everything on the cities.
strike-taboo-since = Taboo: { $taboo }; { $years ->
    [one] { $years } year since the last use in the world
   *[other] { $years } years since the last use in the world
}. Each use cuts the taboo by half, for all.
strike-taboo-never = Taboo: { $taboo }; the weapon has never been used in the world. Each use cuts the taboo by half, for all.
strike-launch = Launch…
strike-confirm = Confirm the launch
strike-confirm-line = { $n ->
    [one] { $warheads } warhead
   *[other] { $warheads } warheads
} on { $country }, about { $dead } dead. It cannot be undone. A restore point is saved first.

# The fallout (N8)
war-fallout = Nuclear fallout
war-fallout-basis = The productivity each state lost to the fallout. The loss falls with caesium-137: half in 30 years.
war-fallout-ours = { $state }: we lost { $share } of the productivity; it halves in { $half } years, under 1% in { $year }
war-fallout-theirs = { $state }: they lost { $share } of the productivity; it halves in { $half } years, under 1% in { $year }
