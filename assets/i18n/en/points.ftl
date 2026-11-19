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
    [0] no points
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
points-reason-disband = Before disbanding forces
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
cmd-campaign-menu = Campaign options
    .short = Options
saves-list-title = Saved campaigns
month-short-1 = Jan
month-short-2 = Feb
month-short-3 = Mar
month-short-4 = Apr
month-short-5 = May
month-short-6 = Jun
month-short-7 = Jul
month-short-8 = Aug
month-short-9 = Sep
month-short-10 = Oct
month-short-11 = Nov
month-short-12 = Dec
date-short = { $month } { $day }, { $year }
date-month-year = { $month } { $year }
saves-played-today = today, { $time }
saves-played-yesterday = yesterday
saves-last-played-full = Last played on { $date }, { $time }
saves-size = Space the campaign takes on disk, restore points included.
saves-size-large = Past the { $limit } MB you set in the settings to be warned about. The game removes nothing on its own: delete old restore points, or raise the warning.
saves-newer = Saved by a newer version of the game ({ $by }, world format { $found }); this one ({ $this }) reads up to format { $supported }. Update the game to open it, or go back to an older restore point below.
saves-damaged = The current state does not read: { $error }. The restore points below may still be good: go back to one of them.
saves-unreadable-short = Does not open in this version
saves-cause-not-a-save = the file is not a save of this game
saves-cause-container-newer = the file was written by a newer version of the engine
saves-cause-truncated = the file is cut short
saves-cause-checksum = the file is damaged (its checksum does not match)
saves-cause-wrong-game = the file is a save of another game
saves-cause-malformed = a part of the file does not decode
saves-damaged-short = Damaged file
settings-autosave-cost = Time it takes from the simulation (measured in 2025): { $costs }.
settings-autosave-cost-one = { $option } ~{ $percent } %
points-reason-denounce = Before denouncing an agreement
points-reason-embargo = Before imposing an embargo
