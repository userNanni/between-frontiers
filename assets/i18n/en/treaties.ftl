### SPDX-License-Identifier: CC-BY-SA-4.0
### Treaties as sets of clauses (phase 4). Keys: clause-<key> (name), clause-<key>-effect (what it
### does, for the giver), front-<key> (why the clause has no effect and what wakes it),
### term-<key> (terms of the account), bundle-<key> (subject grades).

## Clauses: name and effect
clause-cede_states = Cession of states
clause-cede_states-effect = The states pass to the receiver, with their people, plants and works; the debt stays with the giver.
clause-release_country = New country
clause-release_country-effect = The states become a new country, sovereign or a subject, born without debt unless a debt clause says so.
clause-union = Union
clause-union-effect = The giver ceases to exist and joins the receiver, with its treasury and debt.
clause-debt = Debt between governments
clause-debt-effect = The giver comes to owe the receiver, with interest and an annuity; the service leaves the treasury every week.
clause-assume_debt = Assumption of debt
clause-assume_debt-effect = The receiver takes over part of the giver's market debt.
clause-revise_debt = Revision of debt
clause-revise_debt-effect = The creditor changes the annuity, rate or term, suspends the service or forgives part of the debt.
clause-payment = Payment
clause-payment-effect = One payment from the giver's treasury to the receiver's.
clause-techniques = Techniques
clause-techniques-effect = Techniques, people and designs pass from the giver to the receiver.
clause-recognition = Recognition
clause-recognition-effect = The giver recognises the receiver as a state (and, with borders, its present borders); without it there is no alliance, tariff or government loan between the two.
clause-non_aggression = Non-aggression
clause-non_aggression-effect = The giver opens no crisis and no war against the receiver.
clause-defence = Defence
clause-defence-effect = The giver goes to war if the receiver is attacked.
clause-consultation = Consultation
clause-consultation-effect = The giver consults the receiver before acting in the region.
clause-border_guarantee = Border guarantee
clause-border_guarantee-effect = The giver defends the receiver's border against anyone.
clause-demilitarized = Demilitarised zone
clause-demilitarized-effect = The giver keeps no troops nor fortifications in the states.
clause-arms_limit = Arms limit
clause-arms_limit-effect = The giver keeps its army and fleet under the ceilings.
clause-base = Base
clause-base-effect = The receiver keeps a base and a garrison on the giver's territory.
clause-passage = Passage
clause-passage-effect = The giver lets the receiver's forces through.
clause-occupation = Occupation
clause-occupation-effect = The receiver occupies states of the giver as a guarantee, until the term or the condition.
clause-tariff_cap = Tariff ceiling
clause-tariff_cap-effect = The giver's tariff on the receiver's goods stays under the ceiling.
clause-tariff_preference = Tariff preference
clause-tariff_preference-effect = The giver charges the receiver a lower tariff than the others.
clause-customs_union = Customs union
clause-customs_union-effect = The giver applies the receiver's tariff and joins its customs territory.
clause-currency_peg = Pegged currency
clause-currency_peg-effect = The giver's currency follows the receiver's, with reserves deposited.
clause-monetary_union = Monetary union
clause-monetary_union-effect = The parties use a common currency.
clause-capital_protection = Protection of capital
clause-capital_protection-effect = The giver does not take the receiver's capital without paying.
clause-financial_control = Financial control
clause-financial_control-effect = Officials of the receiver collect and spend part of the giver's revenue.
clause-tribute = Tribute
clause-tribute-effect = The giver pays the receiver a sum every year.
clause-foreign_policy = Foreign policy
clause-foreign_policy-effect = The giver signs no treaty without the receiver's consent.
clause-intervention = Right of intervention
clause-intervention-effect = The receiver may impose a law or a government on the giver.
clause-minorities = Protection of minorities
clause-minorities-effect = The giver applies no law aimed at the protected community.
clause-troops_quota = Contingent
clause-troops_quota-effect = The giver puts troops at the receiver's disposal.
clause-offence = Offensive alliance
clause-offence-effect = The giver joins the war the receiver starts, against the parties named.
clause-neutrality = Neutrality
clause-neutrality-effect = The giver does not help whoever is at war with the receiver.

## Fronts: why the clause has no effect and what wakes it
front-borders = No effect until the border operations enter the game (borders, part B).
front-ledger = No effect until the ledger of debts between governments enters the game (treaties, step 2).
front-recognition = No effect until recognition between states enters the game (treaties, step 2).
front-trade = No effect until the tariff enters world trade (phase 4).
front-money = No effect until the monetary regime drives the exchange rate (phase 4).
front-war = No effect until forces, crises and war enter the game (phase 5).
front-communities = No effect until communities enter the population.
front-foreign_capital = No effect until foreign capital enters the game (phase 4).
front-research = No effect until the transfer of techniques enters research.
front-politics = No effect until politics enters the game.

## Terms of each side's account
term-money = Money (present value)
term-trade = Trade
term-territory = Territory
term-security = Security
term-sovereignty = Sovereignty
term-politics = Politics at home
term-trust = Trust in the other party

## Subject grades
bundle-crown_colony = Crown colony
bundle-protectorate = Protectorate
bundle-mandate = Mandate
bundle-dominion = Dominion
bundle-satellite = Satellite

## The negotiation table (Agendas): propose, answer, counter
table-title = Propose an agreement…
table-propose-to = Propose an agreement to { $country }…
table-counter-title = Counterproposal
table-off = The negotiation table is off in this game.
table-pick-country = Pick on the map the country to propose to, or choose it by name.
table-parties = { $me } and { $other }
table-gives = { $country } gives
table-gives-nothing = Nothing yet.
table-add = Add a clause…
table-once = Once, on signature
table-in-force = While the agreement is in force
table-remove = Remove
table-param-amount = Amount
table-param-yearly = Per year
table-param-principal = Principal
table-param-rate = Interest
table-param-years = Term of the debt
table-param-deferrable = Deferrable share
table-param-share = Share
table-param-margin = Margin
table-per-year = { $money } a year
table-annuity = Annuity: { $money }
table-recognition-border = With today's borders
table-out-of-limits = { $param } outside the catalogue's limit.
table-unrecognised = The parties do not recognise each other: this clause cannot be signed without mutual recognition.
table-clause-value = Worth { $me } to you, { $other } to the other side.
table-clause-mine = Worth { $value } to you.
table-states = { $count ->
    [one] { $n } state
   *[other] { $n } states
}
table-states-pick = Pick states…
table-term = Term of the agreement
table-term-none = No term
table-term-tip = How long the agreement is in force
table-years = { $n } years
table-notice = Notice to denounce
table-notice-tip = Years of notice to leave the agreement without a breach
table-notice-some = With notice, either side may leave at the end of the notice, without a breach.
table-notice-none = Without notice, leaving the agreement before its term is a breach.
table-account = Account of { $country }:
table-account-empty = Nothing to weigh yet.
table-likely-accept = Likely answer: { $country } accepts.
table-likely-counter = Likely answer: { $country } counters, with { $count ->
    [one] { $n } clause
   *[other] { $n } clauses
}.
table-likely-refuse = Likely answer: { $country } refuses; { $term } weighed most.
table-term-unknown = the whole
table-blocked = An identical proposal was refused on { $refused }; it may come back to the table on { $back }.
table-deadline = { $country } has { $weeks } weeks to answer; its government answers the next Monday.
table-refusal-empty = Add a clause to propose.
table-refusal-recognition = Remove the marked clauses: without mutual recognition, only recognition and one-off exchanges may be signed.
table-refusal-limits = Adjust the marked parameters: they are outside the catalogue's limits.
table-refusal-cooldown = Change the proposal: the identical one is blocked.
table-weighing = Weighing…
table-propose = Propose
table-send-counter = Send the counterproposal
table-counter-drop = Drop the counterproposal
table-received-title = Proposal from { $country }
table-round = Round { $round } of 3.
table-answer-by = Answer by { $date }; without an answer the proposal lapses as refused.
table-accept = Accept
table-refuse = Refuse
table-counter = Counter…
table-accept-confirm = Accepting signs the agreement: the one-off clauses act on signature, the others hold while it is in force.
table-refuse-confirm = Refusing closes the proposal; an identical one may come back only in 6 months.
table-accept-yes = Sign the agreement
table-refuse-yes = Refuse the proposal
table-answer-back = Back
table-last-round = Third round: no more counterproposals.
table-received = { $country } proposes an agreement ({ $count ->
    [one] { $weeks } week
   *[other] { $weeks } weeks
})
table-section = Negotiation table
table-outgoing = Proposal to { $country } on { $date }: waiting for the answer.
table-closed-signed = Agreement with { $country } signed on { $date }.
table-closed-refused-them = { $country } refused on { $date }; { $term } weighed most.
table-closed-refused-me = You refused the proposal from { $country } on { $date }.
table-closed-countered = Counterproposal in the round with { $country } on { $date }.
table-closed-lapsed = The proposal with { $country } lapsed on { $date }.
table-closed-failed = The proposal with { $country } could not be carried out ({ $date }).
table-revise-of = Debt: { $treaty }
table-annuity-label = Annuity
table-param-grace_years = Grace
table-renegotiate-title = Renegotiate the debt
table-peace-title = Propose peace

## The country's agreements (Agendas): in force, breaches, reputation, denouncing
agreements-open = The country's agreements ({ $n } in force)
agreements-title = The country's agreements
agreements-basis = The agreements in force the country is party to, the breaches that touch it and each country's reputation. An agreement is denounced from here.
agreements-in-force = In force ({ $n })
agreements-none = No agreement in force.
agreements-in-play = Agreement of { $year }
agreements-parties = { $a } with { $b }, signed in { $year }
agreements-term = term of { $years } years, until { $end }
agreements-term-none = no term
agreements-notice = { $years } years' notice to denounce
agreements-notice-none = no notice: denouncing is a breach
agreements-clause = { $clause }: { $givers } gives to { $receivers }
agreements-clause-until = { $line } (until { $until })
agreements-denounce = Denounce…
agreements-denounce-off = With the negotiation table off, a denunciation is not carried out in this game.
agreements-denounce-sent = Denunciation sent; the table carries it out on Monday.
agreements-denounce-notice = With { $years } years' notice, the agreement ends in { $year }, without a breach: until then its clauses hold.
agreements-denounce-breach = This agreement has no notice: denouncing it now breaks every clause the country gives ({ $clauses }). Each party harmed gains a cause against the country, and the reputation suffers.
agreements-denounce-nothing-given = the country gives no clause
agreements-denounce-breach-last = Last step: the agreement ends now, as a breach. A restore point is written first.
agreements-denounce-yes = Denounce with notice
agreements-denounce-breach-next = I want to break the agreement…
agreements-denounce-breach-yes = Break the agreement
agreements-keep = Keep the agreement
agreements-breaches = Breaches
agreements-breaches-none = No breach touching the country.
agreements-breach = { $date }: { $by } broke { $clause } ({ $treaty }), against { $harmed }; { $cause }; value: { $value }.
agreements-cause-held = The country holds a cause against { $country } (value: { $value }).
agreements-cause-faced = { $country } holds a cause against the country (value: { $value }).
agreements-reputation = Reputation
agreements-reputation-off = The reputation is off in this game: breaches stay on the record but do not weigh on promises.
agreements-reputation-basis = What each country's broken promises weigh, in money and as a share of its GDP: the general one, and the security one (defence, alliance, neutrality, non-aggression). A promise is worth less in proportion; it loses a tenth a year.
agreements-col-country = Country
agreements-col-general = General
agreements-col-security = Security
agreements-reputation-value = { $money } ({ $share } of GDP)
agreements-us = { $country } (us)
agreements-ended = Ended
agreements-ended-line = { $name }: { $parties }, from { $signed } to { $ended }.
breach-call-unanswered = call to war unanswered
breach-service-unpaid = debt service unpaid
breach-test-failed = clause failed its monthly test
breach-repudiated = debt repudiated
breach-attacked = attack on a party promised non-aggression
breach-denounced = denounced without notice
agreements-parties-all = Among { $a }, signed in { $year }
agreements-clause-mutual = { $clause }: each party to the others
agreements-value-unmeasured = not measured yet
