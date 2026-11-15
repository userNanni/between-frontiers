//! States on the scenario start date: provinces grouped by the rules in
//! `data/scenario/1925/states.ron`. A state never mixes owners, nor colonies with the metropole.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatesFile {
    pub start_date: String,
    /// Rule per Natural Earth `adm0_a3` for the provinces no explicit state, attachment or carve
    /// claims. A unit without a rule gives one state per province.
    pub countries: BTreeMap<String, Rule>,
    /// Explicit states. Applied first; a province may be in one only.
    #[serde(default)]
    pub states: Vec<Group>,
    /// Province (selector) → province (selector) whose state it joins. For islets and fragments
    /// too small to stand alone; both must have the same owner and colony status.
    #[serde(default)]
    pub attach: BTreeMap<String, String>,
    /// Keys of states allowed to have no pixel in `provinces.png`.
    #[serde(default)]
    pub tiny: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub enum Rule {
    /// Each admin-1 unit is a state (units sharing an ISO 3166-2 code form one).
    Admin1,
    /// The whole unit is one state, keyed by `key` or else by the `adm0_a3` code.
    Whole {
        name: String,
        #[serde(default)]
        key: Option<String>,
    },
    /// One state per value of a Natural Earth attribute (`region` or `region_sub`). `names`
    /// renames values; values renamed alike form one state.
    Field {
        field: String,
        #[serde(default)]
        names: BTreeMap<String, String>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub key: String,
    pub name: String,
    /// Selectors: a province key (`adm1_code`, carve key, `ADM0-<A3>`) or an ISO 3166-2 code,
    /// which takes every province with that code.
    pub provinces: Vec<String>,
}

/// What the grouping needs to know about a province.
pub struct ProvinceInfo<'a> {
    pub key: &'a str,
    pub name: &'a str,
    pub owner: &'a str,
    pub colony: bool,
    pub adm0: &'a str,
    pub iso: &'a str,
    /// Natural Earth attributes usable by [`Rule::Field`].
    pub fields: &'a [(String, String)],
    pub area_km2: f64,
}

impl ProvinceInfo<'_> {
    fn is_carve(&self) -> bool {
        self.key.contains('~')
    }

    fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// A state before ids and geometry: key, name and member indices (into the province list).
#[derive(Debug, Clone)]
pub struct Grouped {
    pub key: String,
    pub name: String,
    pub members: Vec<usize>,
}

/// Result of [`group`]: states sorted by key, and notes worth printing.
#[derive(Debug)]
pub struct Grouping {
    pub states: Vec<Grouped>,
    /// `state_of[i]`: index into `states` of province `i`.
    pub state_of: Vec<usize>,
    pub notes: Vec<String>,
}

impl StatesFile {
    pub fn parse(text: &str) -> Result<Self, String> {
        ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(text)
            .map_err(|e| format!("states.ron: {e}"))
    }
}

/// Key suffix from a name: ASCII letters and digits, upper case, words joined by dashes.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in name.chars() {
        let mapped: &str = match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ā' | 'ă' | 'ą' | 'Á' | 'À' | 'Â' | 'Ä' | 'Ã'
            | 'Å' | 'Ā' => "A",
            'ç' | 'ć' | 'č' | 'Ç' | 'Ć' | 'Č' => "C",
            'ď' | 'đ' | 'Ď' | 'Đ' => "D",
            'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' | 'ə' | 'É' | 'È' | 'Ê' | 'Ë' | 'Ə' => {
                "E"
            }
            'ğ' | 'Ğ' => "G",
            'í' | 'ì' | 'î' | 'ï' | 'ī' | 'ı' | 'İ' | 'Í' | 'Ì' | 'Î' | 'Ï' => "I",
            'ł' | 'Ł' | 'ľ' | 'Ľ' => "L",
            'ñ' | 'ń' | 'ň' | 'Ñ' | 'Ń' => "N",
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' | 'ō' | 'ő' | 'Ó' | 'Ò' | 'Ô' | 'Ö' | 'Õ' | 'Ø' => {
                "O"
            }
            'ř' | 'Ř' => "R",
            'ś' | 'š' | 'ş' | 'ș' | 'Ś' | 'Š' | 'Ş' | 'Ș' => "S",
            'ß' => "SS",
            'ť' | 'ţ' | 'ț' | 'Ť' | 'Ţ' | 'Ț' => "T",
            'ú' | 'ù' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' | 'Ú' | 'Ù' | 'Û' | 'Ü' => "U",
            'ý' | 'ÿ' | 'Ý' => "Y",
            'ź' | 'ż' | 'ž' | 'Ź' | 'Ż' | 'Ž' => "Z",
            'æ' | 'Æ' => "AE",
            'œ' | 'Œ' => "OE",
            _ => "",
        };
        if !mapped.is_empty() {
            out.push_str(mapped);
            dash = false;
        } else if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_uppercase());
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// Key of a single admin-1 unit: `<adm0>-<ISO 3166-2 suffix>`, or the province key when the
/// ISO code is unusable.
fn admin1_key(p: &ProvinceInfo) -> String {
    match p.iso.split_once('-') {
        Some((_, suffix))
            if !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_alphanumeric()) =>
        {
            format!("{}-{}", p.adm0, suffix)
        }
        _ => p.key.to_string(),
    }
}

/// Key of a carved province: `<adm0>-<SUFFIX>`.
fn carve_key(p: &ProvinceInfo) -> String {
    let suffix = p.key.split_once('~').map_or(p.key, |(_, s)| s);
    format!("{}-{}", p.adm0, slug(suffix))
}

struct Selectors<'a> {
    by_key: BTreeMap<&'a str, usize>,
    by_iso: BTreeMap<&'a str, Vec<usize>>,
}

impl<'a> Selectors<'a> {
    fn new(provinces: &'a [ProvinceInfo<'a>]) -> Self {
        let mut by_key = BTreeMap::new();
        let mut by_iso: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
        for (i, p) in provinces.iter().enumerate() {
            by_key.insert(p.key, i);
            if !p.iso.is_empty() && !p.is_carve() {
                by_iso.entry(p.iso).or_default().push(i);
            }
        }
        Self { by_key, by_iso }
    }

    fn resolve(&self, selector: &str) -> Result<Vec<usize>, String> {
        if let Some(&i) = self.by_key.get(selector) {
            return Ok(vec![i]);
        }
        self.by_iso
            .get(selector)
            .cloned()
            .ok_or_else(|| format!("no province matches `{selector}`"))
    }
}

/// Groups the provinces into states. Errors list every problem found in the rules.
pub fn group(file: &StatesFile, provinces: &[ProvinceInfo]) -> Result<Grouping, String> {
    let selectors = Selectors::new(provinces);
    let mut errors = Vec::new();
    let mut notes = Vec::new();
    let mut groups: Vec<Grouped> = Vec::new();
    let mut assigned: Vec<Option<usize>> = vec![None; provinces.len()];

    // 1. Explicit states.
    for g in &file.states {
        let mut members = BTreeSet::new();
        for sel in &g.provinces {
            match selectors.resolve(sel) {
                Ok(found) => members.extend(found),
                Err(e) => errors.push(format!("state {}: {e}", g.key)),
            }
        }
        let gi = groups.len();
        for &i in &members {
            match assigned[i] {
                Some(other) => errors.push(format!(
                    "{} is in states {} and {}",
                    provinces[i].key, groups[other].key, g.key
                )),
                None => assigned[i] = Some(gi),
            }
        }
        groups.push(Grouped {
            key: g.key.clone(),
            name: g.name.clone(),
            members: members.into_iter().collect(),
        });
    }

    // 2. Attachments are resolved last; reserve their sources.
    let mut attached: Vec<(usize, usize)> = Vec::new();
    for (from, to) in &file.attach {
        let (Ok(sources), Ok(targets)) = (selectors.resolve(from), selectors.resolve(to)) else {
            errors.push(format!("attach {from} -> {to}: unknown province"));
            continue;
        };
        for s in sources {
            if let Some(g) = assigned[s] {
                errors.push(format!(
                    "attach {from}: {} is already in state {}",
                    provinces[s].key, groups[g].key
                ));
                continue;
            }
            attached.push((s, targets[0]));
        }
    }
    let is_attached: BTreeSet<usize> = attached.iter().map(|&(s, _)| s).collect();

    // 3. Country rules and carves.
    let used_rules: BTreeSet<&str> = provinces.iter().map(|p| p.adm0).collect();
    for adm0 in file.countries.keys() {
        if !used_rules.contains(adm0.as_str()) {
            errors.push(format!("rule for {adm0}, which has no province"));
        }
    }
    let mut by_key: BTreeMap<String, usize> = groups
        .iter()
        .enumerate()
        .map(|(i, g)| (g.key.clone(), i))
        .collect();
    let mut without_rule = BTreeSet::new();
    for (i, p) in provinces.iter().enumerate() {
        if assigned[i].is_some() || is_attached.contains(&i) {
            continue;
        }
        let (key, name) = if p.is_carve() {
            (carve_key(p), p.name.to_string())
        } else {
            match file.countries.get(p.adm0) {
                None => {
                    without_rule.insert(p.adm0);
                    (admin1_key(p), p.name.to_string())
                }
                Some(Rule::Admin1) => (admin1_key(p), p.name.to_string()),
                Some(Rule::Whole { name, key }) => (
                    key.clone().unwrap_or_else(|| p.adm0.to_string()),
                    name.clone(),
                ),
                Some(Rule::Field { field, names }) => {
                    let Some(value) = p.field(field) else {
                        errors.push(format!("{}: unknown field `{field}`", p.adm0));
                        continue;
                    };
                    let name = names.get(value).map_or(value, String::as_str);
                    if name.is_empty() {
                        errors.push(format!(
                            "{} ({}): empty `{field}`; name it in `names`",
                            p.key, p.name
                        ));
                        continue;
                    }
                    let suffix = slug(name);
                    (format!("{}-{suffix}", p.adm0), name.to_string())
                }
            }
        };
        let gi = *by_key.entry(key.clone()).or_insert_with(|| {
            groups.push(Grouped {
                key,
                name,
                members: Vec::new(),
            });
            groups.len() - 1
        });
        groups[gi].members.push(i);
        assigned[i] = Some(gi);
    }
    if !without_rule.is_empty() {
        notes.push(format!(
            "no rule, one state per province: {}",
            without_rule.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    for (rule_adm0, rule) in &file.countries {
        if let Rule::Field { field, names } = rule {
            for value in names.keys() {
                let used = provinces
                    .iter()
                    .any(|p| p.adm0 == rule_adm0 && p.field(field) == Some(value.as_str()));
                if !used {
                    errors.push(format!("{rule_adm0}: no province has {field} `{value}`"));
                }
            }
        }
    }

    // 4. Attachments.
    for &(s, t) in &attached {
        let (src, dst) = (&provinces[s], &provinces[t]);
        if src.owner != dst.owner || src.colony != dst.colony {
            errors.push(format!(
                "attach {} -> {}: owners or colony status differ",
                src.key, dst.key
            ));
            continue;
        }
        match assigned[t] {
            Some(g) => {
                groups[g].members.push(s);
                assigned[s] = Some(g);
            }
            None => errors.push(format!(
                "attach {} -> {}: the target is attached too",
                src.key, dst.key
            )),
        }
    }

    // 5. Split groups that mix owners or colony status: the largest part keeps the key.
    let mut states: Vec<Grouped> = Vec::new();
    for g in groups {
        if g.members.is_empty() {
            continue;
        }
        let mut parts: BTreeMap<(&str, bool), Vec<usize>> = BTreeMap::new();
        for &i in &g.members {
            parts
                .entry((provinces[i].owner, provinces[i].colony))
                .or_default()
                .push(i);
        }
        if parts.len() == 1 {
            states.push(g);
            continue;
        }
        let area = |m: &[usize]| m.iter().map(|&i| provinces[i].area_km2).sum::<f64>();
        let main = *parts
            .iter()
            .max_by(|a, b| area(a.1).total_cmp(&area(b.1)))
            .map(|(k, _)| k)
            .expect("at least two parts");
        for ((owner, colony), members) in parts {
            if (owner, colony) == main {
                states.push(Grouped {
                    key: g.key.clone(),
                    name: g.name.clone(),
                    members,
                });
                continue;
            }
            let (key, name) = if let [i] = members[..] {
                (provinces[i].key.to_string(), provinces[i].name.to_string())
            } else {
                let suffix = if colony { "-COL" } else { "" };
                (format!("{}-{owner}{suffix}", g.key), g.name.clone())
            };
            notes.push(format!(
                "split {} ({}): {} province(s) of {owner}{} -> {key}",
                g.key,
                g.name,
                members.len(),
                if colony { " (colony)" } else { "" }
            ));
            states.push(Grouped { key, name, members });
        }
    }

    let mut seen = BTreeSet::new();
    for s in &states {
        if !seen.insert(s.key.as_str()) {
            errors.push(format!("duplicate state key {}", s.key));
        }
    }
    for (i, a) in assigned.iter().enumerate() {
        if a.is_none() && errors.is_empty() {
            errors.push(format!("{} has no state", provinces[i].key));
        }
    }
    if !errors.is_empty() {
        return Err(format!("states.ron:\n  {}", errors.join("\n  ")));
    }

    states.sort_by(|a, b| a.key.cmp(&b.key));
    for s in &mut states {
        s.members.sort_unstable();
    }
    let mut state_of = vec![0; provinces.len()];
    for (si, s) in states.iter().enumerate() {
        for &i in &s.members {
            state_of[i] = si;
        }
    }
    Ok(Grouping {
        states,
        state_of,
        notes,
    })
}

/// Label anchor of every state, in texture pixels: the pixel of its largest 4-connected part
/// farthest (inside the state) from the state's edge. `None` for a state without pixels.
/// `state_of_id[id]` is the state index + 1 of province `id` (0 for water).
pub fn centers(
    buf: &[u16],
    width: u32,
    height: u32,
    state_of_id: &[u32],
    count: usize,
) -> Vec<Option<(f32, f32)>> {
    let (w, h) = (width as usize, height as usize);
    let state: Vec<u32> = buf.iter().map(|&id| state_of_id[id as usize]).collect();
    // Chamfer distance (3-4) to the state's edge, propagated only inside the state.
    const INF: u32 = u32::MAX / 2;
    let mut dist = vec![INF; w * h];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let s = state[i];
            if s == 0 {
                dist[i] = 0;
                continue;
            }
            let edge = x == 0
                || y == 0
                || x + 1 == w
                || y + 1 == h
                || state[i - 1] != s
                || state[i + 1] != s
                || state[i - w] != s
                || state[i + w] != s;
            if edge {
                dist[i] = 3;
            }
        }
    }
    let relax = |dist: &mut [u32], i: usize, j: usize, cost: u32| {
        if state[j] == state[i] && dist[j] + cost < dist[i] {
            dist[i] = dist[j] + cost;
        }
    };
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            if state[i] == 0 {
                continue;
            }
            relax(&mut dist, i, i - 1, 3);
            relax(&mut dist, i, i - w, 3);
            relax(&mut dist, i, i - w - 1, 4);
            relax(&mut dist, i, i - w + 1, 4);
        }
    }
    for y in (1..h - 1).rev() {
        for x in (1..w - 1).rev() {
            let i = y * w + x;
            if state[i] == 0 {
                continue;
            }
            relax(&mut dist, i, i + 1, 3);
            relax(&mut dist, i, i + w, 3);
            relax(&mut dist, i, i + w + 1, 4);
            relax(&mut dist, i, i + w - 1, 4);
        }
    }

    // Largest 4-connected part of each state, and its deepest pixel.
    let mut best: Vec<Option<(usize, usize)>> = vec![None; count]; // (part size, pixel)
    let mut seen = vec![false; w * h];
    let mut stack = Vec::new();
    for start in 0..w * h {
        let s = state[start];
        if s == 0 || seen[start] {
            continue;
        }
        seen[start] = true;
        stack.push(start);
        let (mut size, mut deepest) = (0usize, start);
        while let Some(i) = stack.pop() {
            size += 1;
            if dist[i] > dist[deepest] || (dist[i] == dist[deepest] && i < deepest) {
                deepest = i;
            }
            let (x, y) = (i % w, i / w);
            let neighbours = [
                (x > 0).then(|| i - 1),
                (x + 1 < w).then(|| i + 1),
                (y > 0).then(|| i - w),
                (y + 1 < h).then(|| i + w),
            ];
            for j in neighbours.into_iter().flatten() {
                if !seen[j] && state[j] == s {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        let slot = &mut best[s as usize - 1];
        if slot.is_none_or(|(n, _)| size > n) {
            *slot = Some((size, deepest));
        }
    }
    best.into_iter()
        .map(|b| b.map(|(_, i)| ((i % w) as f32 + 0.5, (i / w) as f32 + 0.5)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info<'a>(
        key: &'a str,
        owner: &'a str,
        adm0: &'a str,
        iso: &'a str,
        fields: &'a [(String, String)],
    ) -> ProvinceInfo<'a> {
        ProvinceInfo {
            key,
            name: key,
            owner,
            colony: false,
            adm0,
            iso,
            fields,
            area_km2: 1.0,
        }
    }

    #[test]
    fn slugs_are_ascii() {
        assert_eq!(slug("Île-de-France"), "ILE-DE-FRANCE");
        assert_eq!(slug("Ostpreußen"), "OSTPREUSSEN");
        assert_eq!(
            slug("  Provence-Alpes-Côte d'Azur "),
            "PROVENCE-ALPES-COTE-D-AZUR"
        );
    }

    #[test]
    fn rules_group_and_split_by_owner() {
        let north = vec![("region".to_string(), "North".to_string())];
        let south = vec![("region".to_string(), "South".to_string())];
        let provinces = vec![
            info("AAA-1", "AAA", "AAA", "AA-01", &north),
            info("AAA-2", "AAA", "AAA", "AA-02", &north),
            info("AAA-3", "BBB", "AAA", "AA-03", &north),
            info("AAA-4", "AAA", "AAA", "AA-04", &south),
            info("BBB-1", "BBB", "BBB", "BB-01", &[]),
            info("BBB-2", "BBB", "BBB", "BB-02", &[]),
            info("BBB-2~cut", "AAA", "BBB", "", &[]),
            info("CCC-1", "CCC", "CCC", "CC-01", &[]),
        ];
        let file = StatesFile::parse(
            r#"(start_date: "1925-01-01",
                countries: {
                    "AAA": Field(field: "region", names: {"South": "Southland"}),
                    "BBB": Whole(name: "Bee"),
                },
                states: [(key: "AAA-X", name: "X", provinces: ["AA-04"])],
                attach: {"CC-01": "BBB-1"},
            )"#,
        )
        .unwrap();
        let err = group(&file, &provinces).unwrap_err();
        assert!(err.contains("owners or colony status differ"), "{err}");

        let file = StatesFile::parse(
            r#"(start_date: "1925-01-01",
                countries: {
                    "AAA": Field(field: "region"),
                    "BBB": Whole(name: "Bee"),
                },
                states: [(key: "AAA-X", name: "X", provinces: ["AA-04"])],
            )"#,
        )
        .unwrap();
        let g = group(&file, &provinces).unwrap();
        let keys: Vec<&str> = g.states.iter().map(|s| s.key.as_str()).collect();
        assert_eq!(
            keys,
            ["AAA-3", "AAA-NORTH", "AAA-X", "BBB", "BBB-CUT", "CCC-01"]
        );
        let north = &g.states[1];
        assert_eq!(north.members, [0, 1]);
        assert_eq!(g.states[0].members, [2]);
        assert!(g.notes.iter().any(|n| n.contains("split AAA-NORTH")));
    }

    #[test]
    fn a_province_belongs_to_one_explicit_state() {
        let provinces = vec![info("AAA-1", "AAA", "AAA", "AA-01", &[])];
        let file = StatesFile::parse(
            r#"(start_date: "1925-01-01", countries: {},
                states: [(key: "A", name: "A", provinces: ["AAA-1"]),
                         (key: "B", name: "B", provinces: ["AA-01"])])"#,
        )
        .unwrap();
        assert!(group(&file, &provinces).is_err());
    }

    #[test]
    fn centre_is_inside_the_largest_part() {
        // State 1: a 5x5 block on the left and one stray pixel on the right.
        let (w, h) = (12u32, 7u32);
        let mut buf = vec![0u16; (w * h) as usize];
        for y in 1..6 {
            for x in 1..6 {
                buf[(y * w + x) as usize] = 1;
            }
        }
        buf[(3 * w + 10) as usize] = 2;
        let c = centers(&buf, w, h, &[0, 1, 1], 1);
        assert_eq!(c, vec![Some((3.5, 3.5))]);
    }
}
