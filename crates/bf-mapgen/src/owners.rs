//! Province ownership on the scenario start date: `data/scenario/1925/province_owners.ron`.

use std::collections::BTreeMap;

use serde::Deserialize;

/// Country tags that exist on 1925-01-01. Colonies under direct rule are provinces of the
/// overlord, so they have no tag of their own.
/// TODO: read this from `data/scenario/1925/countries.ron` once that file exists.
pub const TAGS_1925: &[&str] = &[
    // Europe
    "GBR", "IRL", "FRA", "BEL", "NLD", "LUX", "CHE", "ESP", "POR", "ITA", "SMR", "GER", "SAA",
    "DNZ", "AUT", "HUN", "CZE", "POL", "LTU", "LAT", "EST", "FIN", "SWE", "NOR", "DEN", "ISL",
    "SOV", "ROM", "YUG", "BUL", "ALB", "GRE", "TUR", "LIE", "MCO", "AND", // Americas
    "USA", "CAN", "NFL", "MEX", "GUA", "HON", "SAL", "NIC", "CRI", "PAN", "CUB", "HAI", "DOM",
    "COL", "VEN", "ECU", "PER", "BOL", "CHL", "ARG", "URY", "PRY", "BRA",
    // Asia and the Middle East
    "CHN", "JAP", "MGL", "TUV", "TIB", "SIA", "AFG", "PRS", "NEP", "BHU", "RAJ", "IRQ", "TRJ",
    "HEJ", "NEJ", "YEM", "OMA", "KUW", // Africa
    "EGY", "ETH", "LBR", "SAF", // Oceania
    "AUS", "NZL",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnersFile {
    pub start_date: String,
    /// Natural Earth `adm0_a3` codes left off the map.
    pub excluded_countries: Vec<String>,
    /// Default owner per Natural Earth `adm0_a3`.
    pub countries: BTreeMap<String, String>,
    /// Owner per `adm1_code` where it differs from the country default.
    pub provinces: BTreeMap<String, String>,
    /// Pieces cut out of a province, applied in order.
    pub carves: Vec<Carve>,
    /// Metropole of every owner that rules colonies directly. A province of a listed owner
    /// outside its metropole is a colony; owners not listed have none.
    #[serde(default)]
    pub metropoles: BTreeMap<String, Metropole>,
}

/// The territory legally part of a state on the start date (Victoria 3's incorporated states).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metropole {
    /// Natural Earth `adm0_a3` codes whose provinces, when owned by this tag, are metropole.
    pub countries: Vec<String>,
    /// Province keys (`adm1_code` or carve keys) of other units that are metropole too.
    #[serde(default)]
    pub provinces: Vec<String>,
    /// Province keys inside `countries` that are colonies all the same.
    #[serde(default)]
    pub colonies: Vec<String>,
}

impl Metropole {
    /// Whether a province of this owner, with the given key and `adm0_a3`, is in the metropole.
    pub fn contains(&self, key: &str, adm0: &str) -> bool {
        let listed = |keys: &[String]| keys.iter().any(|k| k == key);
        if self.countries.iter().any(|c| c == adm0) {
            !listed(&self.colonies)
        } else {
            listed(&self.provinces)
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Carve {
    /// Key of the province to cut from (after earlier carves).
    pub from: String,
    /// The piece gets the key `<from>~<suffix>`.
    pub suffix: String,
    pub name: String,
    pub owner: String,
    /// Closed ring in (lon, lat) degrees; the piece is the province inside it.
    pub mask: Vec<(f64, f64)>,
}

impl Carve {
    pub fn key(&self) -> String {
        format!("{}~{}", self.from, self.suffix)
    }
}

pub fn is_tag(tag: &str) -> bool {
    TAGS_1925.contains(&tag)
}

impl OwnersFile {
    pub fn parse(text: &str) -> Result<Self, String> {
        ron::from_str(text).map_err(|e| format!("province_owners.ron: {e}"))
    }

    /// Every owner in the file must be a tag of the start date.
    pub fn check_tags(&self) -> Result<(), String> {
        let owners = self
            .countries
            .iter()
            .chain(&self.provinces)
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .chain(
                self.carves
                    .iter()
                    .map(|c| (c.from.as_str(), c.owner.as_str())),
            )
            .chain(
                self.metropoles
                    .keys()
                    .map(|tag| ("metropoles", tag.as_str())),
            );
        let bad: Vec<String> = owners
            .filter(|(_, tag)| !is_tag(tag))
            .map(|(key, tag)| format!("{key} -> {tag}"))
            .collect();
        if bad.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "owners not on the 1925 tag list: {}",
                bad.join(", ")
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_file() -> OwnersFile {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../data/scenario/1925/province_owners.ron"
        );
        let text = std::fs::read_to_string(path).expect("read province_owners.ron");
        OwnersFile::parse(&text).expect("parse province_owners.ron")
    }

    #[test]
    fn every_owner_is_on_the_tag_list() {
        data_file().check_tags().unwrap();
    }

    #[test]
    fn tag_list_is_well_formed() {
        let mut seen = std::collections::BTreeSet::new();
        for tag in TAGS_1925 {
            assert!(
                tag.len() == 3 && tag.bytes().all(|b| b.is_ascii_uppercase()),
                "{tag}"
            );
            assert!(seen.insert(tag), "duplicate tag {tag}");
        }
    }

    #[test]
    fn carve_masks_are_polygons() {
        let file = data_file();
        assert_eq!(file.start_date, "1925-01-01");
        for carve in &file.carves {
            assert!(carve.mask.len() >= 3, "{}", carve.key());
            for &(lon, lat) in &carve.mask {
                assert!((-180.0..=180.0).contains(&lon) && (-90.0..=90.0).contains(&lat));
            }
        }
    }

    #[test]
    fn metropole_membership() {
        let file = OwnersFile::parse(
            r#"(start_date: "1925-01-01", excluded_countries: [], countries: {}, provinces: {},
                carves: [], metropoles: {"JAP": (countries: ["JPN"],
                provinces: ["RUS-2616~kurils"], colonies: ["JPN-1"])})"#,
        )
        .unwrap();
        let jap = &file.metropoles["JAP"];
        assert!(jap.contains("JPN-2", "JPN"));
        assert!(!jap.contains("JPN-1", "JPN"));
        assert!(jap.contains("RUS-2616~kurils", "RUS"));
        assert!(!jap.contains("RUS-2616~karafuto", "RUS"));
        assert!(!jap.contains("KOR-1", "KOR"));
    }

    #[test]
    fn rejects_unknown_tags() {
        let file = OwnersFile::parse(
            r#"(start_date: "1925-01-01", excluded_countries: [], countries: {"XXA": "ZZZ"},
                provinces: {}, carves: [])"#,
        )
        .unwrap();
        assert!(file.check_tags().is_err());
    }
}
