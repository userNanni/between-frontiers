# The government's Trade tab: the tariff by good and by partner (docs/trade.pt-BR.md, section 13).
gov-tab-trade = Trade
cmd-gov-tab-trade = Government: tariff by good and by partner
    .short = Trade
tariff-mean = Mean tariff { $rate }
tariff-mean-tip = The mean tariff is the plain mean of the goods on sale this year, each good weighing the same, before the agreements.
tariff-revenue = Customs revenue { $amount }
tariff-revenue-would = Customs revenue it would collect { $amount }
tariff-off = The tariff that counts is off in this game: the table weighs only on the choice between sellers, and nothing goes to the treasury.
tariff-law = Law: { $law }
tariff-basis = Each good pays the historical rule (the trade law in force, from the 1925 tariff) or the rate the government fixed. A partner's points add to every good's tariff, and the agreements in force hold the result. The AI keeps the historical rule.
tariff-col-good = Good
tariff-col-rule = Rule
tariff-col-rate = Tariff
tariff-col-imports = Imports/year
tariff-imports-tip = By good: last week's net imports (what the country bought abroad less what it sold), × 52, at base prices. A good the country exports more than it imports shows a dash. By partner: what was bought from it in the year.
tariff-col-revenue = Revenue/year
tariff-revenue-col-tip = The good's tariff on the net imports beside it: what it would collect in a year. The country's revenue, at the top, counts the imports from every partner, the exchange of varieties included, and so is larger than the sum of this column.
tariff-range = { $low } to { $high }
tariff-fixed-count = { $n } of { $of } fixed
tariff-group-apply = Apply to the group:
tariff-apply = Apply
tariff-back-to-rule = Back to the rule
tariff-rate-limits = From 0 % to 200 %, one percentage point at a time.
tariff-partners = By partner
tariff-partners-basis = The { $n } largest sellers to the country in { $year }, at base prices, plus those with an adjustment and those picked by name.
tariff-partners-basis-so-far = The { $n } largest sellers to the country in { $year } so far (the first year has not closed yet), plus those with an adjustment and those picked by name.
tariff-partners-none = No purchases from another country yet; pick one by name to adjust its tariff.
tariff-col-partner = Partner
tariff-col-agreement = Agreement in force
tariff-col-points = Adjustment
tariff-col-effective = Effective tariff
tariff-agreement-none = none
tariff-points = { $sign }{ $n } { $n ->
    [one] point
   *[other] points
}
tariff-points-none = no adjustment
tariff-points-tip = Points on every good's tariff for this partner: positive, a surcharge; negative, a preference. From −100 to +100 points. The effective tariff is the mean of the goods, already within the agreements in force.
tariff-limit-cap = The agreement caps the tariff at { $rate }: the surcharge above it does not count.
tariff-limit-union = The customs union charges this partner no tariff: the surcharge does not count.
tariff-breach-cap = Going past { $rate } with { $partner } ({ $points }) breaks the tariff clause of the agreement in force: your general reputation for honouring agreements falls, { $partner } gains a cause against you and the clause no longer holds between you. A restore point is written first.
tariff-breach-union = Charging { $partner } a tariff ({ $points }) breaks the customs union of the agreement in force: your general reputation for honouring agreements falls, { $partner } gains a cause against you and the union no longer holds between you. A restore point is written first.
tariff-breach-confirm = Break the clause
tariff-breach-cancel = Cancel
