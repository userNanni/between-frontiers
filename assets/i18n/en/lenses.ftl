# The influence and blocs lenses on the map.

cmd-map-mode-influence = Influence map
    .short = Influence
cmd-map-mode-blocs = Blocs map
    .short = Blocs
lens-influence-none = No influence data in this campaign.
legend-influence-focus = { $country } (in focus)
legend-influence-nothing = Nothing measured
tip-influence = { $share } of GDP · largest issue: { $issue } · { $channels }
tip-influence-focus = Country in focus
tip-influence-nothing = No influence measured
lens-blocs-none = No treaty data in this campaign.
lens-blocs-empty = No bloc of this kind on this date.
legend-bloc-line = { $name } · { $count ->
    [one] { $members } member
   *[other] { $members } members
}
legend-blocs-member = Light tone: member; strong tone: leader
legend-blocs-customs = Customs union
bloc-name-subjects = { $leader } and its subjects
tip-bloc-leader = { $name } (leader)
tip-bloc-none = In no bloc of this kind
lens-ties = Ties
lens-ties-influence_over = Influence of the country over others
lens-ties-influence_on = Influence of others over the country
lens-ties-military = Military blocs
lens-ties-monetary = Monetary blocs
lens-ties-subjects = Subjects
legend-influence-unit-over = Influence of { $article ->
    [the] the { $country }
   *[other] { $country }
} over each country, as % of the GDP of the country influenced
legend-influence-unit-on = Influence of each country over { $article ->
    [the] the { $country }
   *[other] { $country }
}, as % of the GDP of { $article ->
    [the] the { $country }
   *[other] { $country }
}
legend-influence-scope = Measured today: credit between governments and goodwill. Subjects: Ties → Subjects.
bloc-name-military = Military bloc of { $article ->
    [the] the { $leader }
   *[other] { $leader }
}
bloc-name-monetary = Monetary bloc of { $article ->
    [the] the { $leader }
   *[other] { $leader }
}
bloc-name-customs = Customs bloc of { $article ->
    [the] the { $leader }
   *[other] { $leader }
}
legend-bloc-leaderless = { $name } · no leader · { $count ->
    [one] { $members } member
   *[other] { $members } members
}
