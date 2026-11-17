# Restore points, quick save and the campaign menu.

cmd-campaign-quick-save = Quick save
    .short = Quick save
cmd-campaign-quick-load = Quick load
    .short = Quick load
cmd-campaign-save-named = Save with a name
    .short = Save as

settings-autosave-quarterly = Quarterly
settings-autosave-semiannual = Half-yearly
settings-autosave-hint = An automatic restore point, and the campaign saved, on each turn of the calendar.
settings-autosave-keep = Automatic points kept
settings-points-cap = Points per campaign at most
settings-points-warn = Warn when a campaign folder passes
settings-point-before-risk = Restore point before a risky decision
settings-point-before-risk-hint = Before a law change, freeing territory or accepting an agreement (and, when they arrive, declaring war or signing a treaty) the game keeps where you were, so you can go back and try another road.
settings-points-cost = Each point: ~{ $each } MB · { $keep } automatic: ~{ $auto } MB · ceiling of { $cap }: ~{ $full } MB
settings-points-cost-unknown = The size of a point shows once a campaign has been saved.

game-menu-quick-save = Quick save
game-menu-quick-load = Quick load
game-menu-save-named = Save with a name…

points-title = Restore points
points-summary = { $name }: { $count ->
    [one] one point
   *[other] { $count } points
    }, { $manual } out of the rotation · folder { $size } MB
points-empty = No restore point yet. The autosave, a quick save (F5) or “Save with a name” make them.
points-kind-auto = Automatic
points-kind-quick = Quick save
points-kind-manual = Saved by hand
points-reason-law = Before a law change
points-reason-liberate = Before freeing territory
points-reason-accept = Before accepting an agreement
points-reason-war = Before declaring war
points-reason-treaty = Before signing a treaty
points-reason-load = Before going back to another point
points-load = Go back to this point
points-rename = Rename
points-delete = Delete
points-delete-confirm = Delete this point?
points-delete-yes = Delete point
points-name-title = Save with a name
points-name-save = Save
points-name-cancel = Cancel
points-saved-named = Saved “{ $name }”
points-saved-quick = Quick save written
points-no-quick = No quick save in this campaign yet.
points-folder-large = The campaign folder takes { $size } MB, more than the { $limit } MB you set.

saves-menu = Campaign options
saves-menu-rename = Rename
saves-menu-delete = Delete campaign
saves-menu-open-folder = Open folder
saves-menu-play-other = Play another country
saves-rename-ok = Rename
saves-cancel = Cancel
saves-new-cancel = Back to the restore points
