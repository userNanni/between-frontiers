# The gold standard: exchange, reserves and capital (Money and debt tab)
points-reason-exchange = Before changing the exchange regime or capital controls
gold-points = { $value } { $n ->
    [one] percentage point
   *[other] percentage points
}
gold-title = Exchange and reserves
gold-basis = The central bank holds the parity while its reserves cover the money it issued. Changing the regime is a decision of the Agendas.
gold-regime = Exchange regime
gold-regime-float = Floating exchange rate
gold-regime-gold = Gold standard
gold-regime-control = Parity behind exchange control
gold-regime-float-tip = The exchange rate drifts to purchasing-power parity; the rate follows the Taylor rule.
gold-regime-gold-tip = The exchange rate holds the parity; the reserves' cover sets the rate: below the legal cover, the rate rises above the Taylor rule and the bank creates no money.
gold-regime-control-tip = The parity holds, behind exchange control: no cover rule and no exit by it; what the country cannot pay stays blocked abroad in its own currency.
gold-by-rule = by the historical rule
gold-by-government = set by the government
gold-pending = Asked of the government: { $choice }. Takes effect next week.
gold-parity = Parity
gold-parity-value = { $parity } (today's rate { $today })
gold-parity-tip = Exchange rate index, 1925 = 100. The parity is the rate the central bank holds.
gold-cover-line = Cover { $cover } · legal { $legal } · floor { $floor }
gold-cover-tip = Cover: the central bank's gold and foreign exchange, at the real exchange rate, over the monetary base (the bank's notes and deposits: what the game issued, bonds and credit, plus the rest of the 1925 base). It runs from −1000% to +1000%. The white mark is the legal cover; the red one, the floor.
gold-below-legal = Below the legal cover for { $weeks } { $weeks ->
    [one] week
   *[other] weeks
}:
gold-below-floor-rule = Below the floor for { $weeks } { $weeks ->
    [one] week
   *[other] weeks
}: by the historical rule, it leaves gold in { $left } { $left ->
    [one] week
   *[other] weeks
}.
gold-below-floor-defend = Below the floor for { $weeks } { $weeks ->
    [one] week
   *[other] weeks
}: the government holds the parity with the rule's rate and leaves gold only when the reserves (gold and foreign exchange) run out.
gold-below-floor-devalue = Below the floor for { $weeks } { $weeks ->
    [one] week
   *[other] weeks
}: in { $left } { $left ->
    [one] week
   *[other] weeks
} the government moves the parity by { $share } and stays on gold; the count starts again after.
gold-posture = Below the floor
gold-posture-rule = Historical rule: leave gold
gold-posture-fixed = Set by the government
gold-posture-defend = Defend
gold-posture-devalue = Devalue
gold-posture-rule-means = After some weeks below the floor the country leaves gold and the rate floats; the risk premium rises for some years.
gold-posture-defend-means = Holds the parity with the rule's rate until the reserves (gold and foreign exchange) run out: deflation and unemployment instead of a devaluation.
gold-posture-devalue-means = Moves the parity by { $share } and stays on gold: holders of our currency lose, and the cover rises again.
gold-premium = Risk premium
gold-premium-tip = Points an exit from gold left on the bonds' rate; they fall to zero within some years.
gold-reserves = Central bank reserves
gold-reserves-tip = Gold and foreign exchange, in dollars at base prices, as in the balance of payments.
gold-reserves-gold = Gold
gold-reserves-fx = In the currency of { $country }
gold-reserves-fx-tip = Foreign exchange of { $country }. It loses value when { $country }'s currency falls, by a devaluation or floating: if its parity rises { $share }, these reserves lose { $loss }.
gold-owed = Our currency in other reserves
gold-owed-tip = What other central banks hold in our currency: it comes off our net reserves and loses value with it.
gold-capital = Private capital
gold-capital-tip = Capital follows the expected return: the rate without the premium, less inflation and the exchange risk. Short-term money leaves faster.
gold-controls = Capital controls
gold-controls-on = on
gold-controls-off = off
gold-controls-tip = With capital controls the country neither lends abroad nor takes new capital, and positions freeze; interest is still paid.
gold-controls-ask-on = Turn capital controls on…
gold-controls-ask-off = Turn capital controls off…
gold-controls-on-means = Turning them on freezes every position, ours abroad and those from abroad here, and stops new capital both ways; the risk premium rises { $points }. A restore point is saved first, if the settings ask for it.
gold-controls-off-means = Turning them off frees the positions: capital follows the rates again, and short-term money may leave fast if the return here is low. A restore point is saved first, if the settings ask for it.
gold-controls-turn-on = Turn controls on
gold-controls-turn-off = Turn controls off
gold-choice-prewar = adhere to gold at the prewar parity
gold-choice-market = adhere to gold at today's rate
gold-choice-float = leave gold and float
gold-choice-control = hold the parity behind exchange control
gold-reserves-rest = In { $n } other currencies
gold-capital-abroad-long = Our savers abroad, long term
gold-capital-abroad-short = Our savers abroad, short term
gold-capital-here-long = From abroad, here, long term
gold-capital-here-short = From abroad, here, short term
gold-below-legal-rate = the rate { $points } above the Taylor rule;
gold-below-legal-rate-fixed = the government fixed the rate, without the rule's addition;
gold-below-legal-stop = the bank creates no money and, in { $left } { $left ->
    [one] week
   *[other] weeks
}, starts withdrawing money.
gold-below-legal-withdrawing = the bank is withdrawing money: { $share } of GDP a year.
gold-reserves-net = Net reserves
gold-reserves-net-tip = Reserves less our currency in other reserves: what is left if others exchanged the currency of ours they hold. The cover counts the gross reserves, without this.

# The gold standard decision (Agendas)
gold-decision-open-join = Adhere to the gold standard…
gold-decision-open-leave = Leave the gold standard…
gold-decision-title = Gold standard
gold-decision-now = Today: { $regime }, { $by }. Each choice shows what it would mean this week; it takes effect the week after, with a restore point first.
gold-decision-parity = Parity { $parity } (1925 = 100): the currency sits { $gap } above its market value
gold-decision-cover = Cover it would give { $cover } · legal { $legal } · floor { $floor } · { $margin }
gold-margin-over = { $value } to spare
gold-margin-short = { $value } short of the legal cover
gold-decision-defense = Below the legal cover: the rate { $points } above the Taylor rule from the first week.
gold-decision-no-prewar = No prewar parity in this country's data: it can only adhere at today's rate.
gold-decision-confirm = Confirm
gold-choice-prewar-title = Adhere at the prewar parity
gold-choice-market-title = Adhere at today's rate
gold-choice-float-title = Leave gold and float
gold-choice-control-title = Hold the parity behind exchange control
gold-choice-rule-title = Hand the exchange back to the historical rule
gold-choice-prewar-means = The currency goes back to its prewar value: prestige and credit, but the parity sits above the market and exports lose price.
gold-choice-market-means = The rate stays where it is: no loss of competitiveness, and the cover is today's.
gold-choice-float-means = The rate drifts to purchasing-power parity and the rate goes back to the Taylor rule; holders of our currency lose if it falls, and the risk premium rises for some years.
gold-choice-control-means = The parity holds behind exchange control: no cover rule, and what the country cannot pay stays blocked abroad.
gold-choice-rule-means = The historical rule decides again: it leaves below the floor and returns to gold with low inflation and enough cover.
gold-choice-prewar-confirm = Adhering at the prewar parity takes effect next week. If the cover falls below the legal one, the rate rises and the bank stops creating money. A restore point is saved first.
gold-choice-market-confirm = Adhering at today's rate takes effect next week. From then on, the cover sets the rate. A restore point is saved first.
gold-choice-float-confirm = Leaving gold takes effect next week: the rate floats, the risk premium rises, and holders of our currency lose if it falls. A restore point is saved first.
gold-choice-control-confirm = Exchange control takes effect next week: capital freezes, and what the country cannot pay stays blocked abroad. A restore point is saved first.
gold-choice-rule-confirm = Handing back to the historical rule takes effect next week. A restore point is saved first.

# The gold standard's lens and the journal's notice
lens-ties-gold = Gold standard: who is on gold
legend-gold-off = The gold standard is off in this game.
legend-gold-on = On the gold standard
legend-gold-control = Parity behind exchange control
legend-gold-float = Floating exchange rate
legend-gold-count = { $gold } on gold · { $control } under control · { $float } floating
tip-gold-on = On the gold standard: parity { $parity } (1925 = 100), cover { $cover } against a legal { $legal }
tip-gold-control = Parity { $parity } (1925 = 100) behind exchange control
tip-gold-float = Floating exchange rate
journal-gold-title = Gold standard
journal-gold-cause-below = The reserves cover less than the legal cover: by the rule, the rate rises and the bank creates no money. Below the floor, the country leaves gold.
journal-gold-cause-world = Who joined and who left the gold standard in the last twelve months. When the main issuer of a country's foreign exchange leaves, the country may leave with it.
journal-gold-cond-cover = Cover above the legal one
journal-gold-join = { $country } joined the gold standard
journal-gold-leave = { $country } left the gold standard by its cover
journal-gold-contagion = { $country } left the gold standard with the issuer of its foreign exchange
journal-gold-control = { $country } went to hold its parity behind exchange control
journal-gold-devalue = { $country } devalued its parity and stayed on gold
journal-gold-abandon = { $country } left the gold standard by the government's choice
journal-gold-cond-below = Cover below the legal one
