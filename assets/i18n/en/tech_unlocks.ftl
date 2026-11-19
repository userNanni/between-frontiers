# What each technology opens, and the way back
research-units = Units: { $list }
research-gains = Once diffused: { $list }
research-gain-productivity = +{ $pct } productivity in { $group }
research-effect-mortality = { $pct } of deaths avoided
research-effect-fertility = { $pct } of births avoided
research-effect-literacy = literacy grows { $pct } faster
research-effect-research = { $pct } more research per researcher
research-effect-trade = trade { $pct } cheaper
research-leads-to = Leads to: { $list }
research-missing = To research this one, { $n ->
    [one] 1 technology is
   *[other] { $n } technologies are
} missing, { $cost } points in all: { $list }
research-missing-none = The requirements are met: it can be researched now.
research-search-hint = Search: technology, building, method, good or unit
research-search-found = { $n ->
    [0] nothing found
    [one] 1 technology
   *[other] { $n } technologies
}
