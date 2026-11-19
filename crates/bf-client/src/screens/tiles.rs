//! The blocks the country figures panel can show: each one an id kept in the HUD layout, an
//! icon, a name and where a click takes the player. The panel shows the blocks the player chose,
//! in their order: the first row large with a bar, the others compact.

use bf_ui_kit::Icon;

/// Where a click on a block goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Open {
    Budget,
    Money,
    Laws,
    Research,
    Journal,
}

#[derive(Clone, Copy, Debug)]
pub struct TileDef {
    /// Id in the layout file.
    pub key: &'static str,
    pub icon: Icon,
    /// Message id of its name.
    pub name: &'static str,
    pub open: Option<Open>,
}

const fn t(key: &'static str, icon: Icon, name: &'static str, open: Option<Open>) -> TileDef {
    TileDef {
        key,
        icon,
        name,
        open,
    }
}

/// Every block, in the order the tray lists them.
pub const TILES: [TileDef; 25] = [
    // The figures the panel showed before it could be arranged, in their order.
    t("quality", Icon::Heart, "bar-quality", None),
    t("literacy", Icon::Book, "bar-literacy", None),
    t("life", Icon::Hourglass, "bar-life", None),
    t("urban", Icon::City, "bar-urban", None),
    t("gdp-per-capita", Icon::Coin, "hud-per-capita", None),
    t("gdp", Icon::Chart, "bar-gdp", None),
    t("population", Icon::People, "bar-population", None),
    t("density", Icon::Density, "hud-density", None),
    t("area", Icon::Map, "hud-area", None),
    t("states", Icon::Grid, "hud-states", None),
    t("gdp-share", Icon::Pie, "hud-gdp-share", None),
    t("pop-share", Icon::Globe, "hud-pop-share", None),
    t("rank-quality", Icon::Heart, "hud-rank-quality", None),
    t("rank-population", Icon::People, "hud-rank-population", None),
    t("subjects", Icon::Crown, "hud-subjects", None),
    // Figures the windows already show.
    t("treasury", Icon::Coins, "tile-treasury", Some(Open::Budget)),
    t("balance", Icon::Coins, "tile-balance", Some(Open::Budget)),
    t("taxes", Icon::Coins, "tile-taxes", Some(Open::Budget)),
    t("debt", Icon::Chart, "tile-debt", Some(Open::Money)),
    t(
        "inflation",
        Icon::Chart,
        "tile-inflation",
        Some(Open::Money),
    ),
    t(
        "policy-rate",
        Icon::Shield,
        "tile-policy-rate",
        Some(Open::Money),
    ),
    t(
        "legitimacy",
        Icon::Crown,
        "tile-legitimacy",
        Some(Open::Laws),
    ),
    t(
        "movements",
        Icon::People,
        "tile-movements",
        Some(Open::Journal),
    ),
    t("works", Icon::City, "tile-works", Some(Open::Budget)),
    t(
        "research",
        Icon::Flask,
        "tile-research",
        Some(Open::Research),
    ),
];

/// The figures the map gives every country, in the order `top_bar::items` builds them.
pub const FIGURES: [&str; 15] = [
    "quality",
    "literacy",
    "life",
    "urban",
    "gdp-per-capita",
    "gdp",
    "population",
    "density",
    "area",
    "states",
    "gdp-share",
    "pop-share",
    "rank-quality",
    "rank-population",
    "subjects",
];

/// The blocks of a player who never arranged them: the panel as it always was, with the public
/// debt beside the GDP.
pub const DEFAULT: [&str; 15] = [
    "quality",
    "literacy",
    "life",
    "urban",
    "gdp-per-capita",
    "gdp",
    // The debt in sight: the owner looked for it and did not find the money tab. The rank in
    // population left the default for it (still in the tray), so the panel keeps three rows.
    "debt",
    "population",
    "density",
    "area",
    "states",
    "gdp-share",
    "pop-share",
    "rank-quality",
    "subjects",
];

/// Blocks per row; the first row is the large one.
pub const COLS: usize = 5;
/// The most blocks the panel takes (four rows) and the fewest.
pub const MAX: usize = 20;
pub const MIN: usize = 1;

pub fn def(key: &str) -> Option<&'static TileDef> {
    TILES.iter().find(|t| t.key == key)
}

/// The sections of the overview panel, in their default order.
pub const SECTIONS: [(&str, Icon, &str); 3] = [
    ("works", Icon::City, "outliner-works"),
    ("research", Icon::Flask, "outliner-research"),
    ("pinned", Icon::Star, "outliner-pinned"),
];
