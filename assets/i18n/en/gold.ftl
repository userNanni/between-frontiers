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
gold-cover-tip = Cover: the central bank's net reserves (gold and foreign exchange, less the country's currency in other reserves) over the money it issued. The white mark is the legal cover; the red one, the floor.
gold-below-legal = Below the legal cover for { $weeks } { $weeks ->
    [one] week
   *[other] weeks
}: the rate { $points } above the Taylor rule, and the bank creates no money. In { $left } { $left ->
    [one] week
   *[other] weeks
} it starts withdrawing money.
gold-below-floor-rule = Below the floor for { $weeks } { $weeks ->
    [one] week
   *[other] weeks
}: by the historical rule, it leaves gold in { $left } { $left ->
    [one] week
   *[other] weeks
}.
gold-below-floor-defend = Below the floor: the government defends the parity with the rate until the reserves run out.
gold-below-floor-devalue = Below the floor: the government devalues the parity by { $share } instead of leaving.
gold-posture = Below the floor
gold-posture-rule = Historical rule: leave gold
gold-posture-fixed = Set by the government
gold-posture-defend = Defend
gold-posture-devalue = Devalue
gold-posture-rule-means = After some weeks below the floor the country leaves gold and the rate floats; the risk premium rises for some years.
gold-posture-defend-means = Holds the parity with the rate past the floor until the reserves run out: deflation and unemployment instead of a devaluation.
gold-posture-devalue-means = Moves the parity by { $share } and stays on gold: holders of our currency lose, and the cover rises again.
gold-premium = Risk premium
gold-premium-tip = Points an exit from gold left on the bonds' rate; they fall to zero within some years.
gold-reserves = Central bank reserves
gold-reserves-tip = Gold and foreign exchange, in dollars at base prices, as in the balance of payments.
gold-reserves-gold = Gold
gold-reserves-fx = In the currency of { $country }
gold-reserves-fx-tip = Foreign exchange of { $country }. If { $country }'s currency devalues by { $share }, these reserves lose { $loss }.
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
