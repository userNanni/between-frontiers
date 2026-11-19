# Crisis and declaration (phase 5, slice B), in the Agendas.

crisis-section = Crises and wars
crisis-list = { $by } demands of { $target } · { $stage }
crisis-title = { $by } demands of { $target }
crisis-stage-answer = ours to answer
crisis-stage-waiting = waiting for the answer
crisis-stage-decide = refused: ours to declare or back down
crisis-stage-refused = we refused: the other side decides
crisis-stage-accepted = accepted: becomes a treaty next week
crisis-deadline = { $count ->
    [one] { $left } week left; no answer counts as a refusal
   *[other] { $left } weeks left; no answer counts as a refusal
}
crisis-takes-union = The whole country: { $country } ceases to exist
crisis-takes-states = States: { $list }
crisis-takes-money = Money: { $money }
crisis-takes-nothing = No clause yet
crisis-no-cause = No recognised cause: the demand is recorded as made without a casus belli.
crisis-weighing = How the target weighs it
crisis-force = Land force of the side that demands over the target's
crisis-force-value = { $ratio } ({ $by } against { $target } men)
crisis-share = Share of the country asked
crisis-needed = Force that would make it yield
crisis-needed-never = never: it is existential
crisis-yields = Expected answer: yields
crisis-refuses = Expected answer: refuses
crisis-third-parties = Who would join, and by which clause
crisis-accept = Accept…
crisis-refuse = Refuse…
crisis-accept-confirm = Accepting carries the clauses out as a treaty next week. A restore point is saved first.
crisis-refuse-confirm = Refusing hands the decision back to the side that demanded: declare war or back down. A restore point is saved first.
crisis-declare = Declare war…
crisis-back-down = Back down…
crisis-declare-confirm = The war cannot be undone. A restore point is saved first.
crisis-back-down-confirm = Backing down closes the crisis with nothing. A restore point is saved first.
crisis-confirm = Confirm
crisis-cancel = Back
crisis-war-line = At war: { $a } against { $b }, since { $year }
crisis-blocked-war = We are already at war with this country.
crisis-blocked-open = A crisis with this country is already open.
crisis-demand-title = Demand of another country
crisis-demand-from = Demand of { $country }…
crisis-demand-pick-target = Select on the map the country to demand of, or choose it by name.
crisis-demand-of = Of { $country }
crisis-demand-clauses = What to demand
crisis-demand-union = The whole country (annexation)
crisis-demand-states = States to cede
crisis-demand-payment = One payment, % of the target's GDP
crisis-demand-tribute = Tribute a year, % of the target's GDP
crisis-demand-of-gdp = As a percentage of last year's GDP of the country the demand is made of
crisis-demand-deadline = Time to answer: { $weeks } weeks
crisis-demand-review = Review the demand
crisis-demand-confirm = Send the demand to { $country }?
crisis-demand-send = Send the demand
crisis-demand-final = Refused, it is ours to declare war or back down. A restore point is saved before sending.
crisis-share-all = the whole country
crisis-third-parties-off = No agreement in force in this campaign: nobody is called (the agreements are switched off in the rules).
crisis-third-parties-none = No agreement in force calls a third party in this crisis.
crisis-agreement = { $name }, { $year }
crisis-duty-defend = { $party } — defence ({ $agreement }) — with { $partner }
crisis-duty-attack = { $party } — offence ({ $agreement }) — with { $partner }
crisis-duty-neutral = { $party } — neutrality ({ $agreement }): does not join against { $partner }
crisis-duty-not-attack = Declaring breaks the non-aggression with { $partner } ({ $agreement })
crisis-duty-suspended = suspended: the partner broke the agreement first
crisis-duty-barred = barred: a neutrality forbids it
call-title = { $ally } calls: { $enemy } attacked it
call-by = By the agreement: { $agreement }
call-deadline = { $count ->
    [one] { $left } week left; no answer counts as honouring
   *[other] { $left } weeks left; no answer counts as honouring
}
call-honour-means = Honour
call-honour-what = enter the war: reserve called up and a weekly cost
call-refuse-means = Do not honour
call-refuse-what = the breach goes on the record, and costs { $price } (a year of the forces)
call-honour = Honour…
call-refuse = Do not honour…
call-honour-confirm = Honouring enters the war and cannot be undone. A restore point is saved first.
call-refuse-confirm = Not honouring records the breach of the agreement. A restore point is saved first.
