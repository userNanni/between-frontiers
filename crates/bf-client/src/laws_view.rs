//! What the laws screen reads: the laws a country holds, the clout of its interest groups, its
//! legitimacy and movements, and how each interest group weighs each law, effect by effect. A
//! thin layer over `bf_sim::politics` and `bf_sim::polity`, so that the screen never touches the
//! simulation's types.
//!
//! Two sources: the catalogue (`data/politics/*.ron`, with the scenario's legal sources), read
//! once from disk, and the player's politics as the simulation holds them ([`Live`]), copied
//! into each snapshot.

use std::path::Path;
use std::sync::{Arc, OnceLock};

use bf_sim::game::{GameCommand, GameWorld};
use bf_sim::politics::{
    CATEGORIES, Combine, EFFECTS, InterestGroupsFile, LawsFile, LawsInForce, MovementsFile,
    Politics, ScenarioLawsFile, Scope,
};
use bf_sim::polity::{self, Enactor, LawChangeRecord, MovementState, PoliticsState};
use bf_sim::scenario::CountriesFile;
use laplace_schelling::calendar::Date;

/// The player's politics, copied out of the world for the interface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Live {
    pub tag: String,
    /// Law in force per law group (`None`: a colonial group the country does not hold).
    pub laws: Vec<Option<u16>>,
    /// Political weight of each interest group, per mille.
    pub clout: Vec<u16>,
    /// Consent to the laws in force, per mille.
    pub legitimacy: u16,
    /// Movements with any support: (movement kind, state).
    pub movements: Vec<(u16, MovementState)>,
    /// Laws asked for and not yet enacted.
    pub pending: Vec<u16>,
    /// Law changes of the campaign, oldest first.
    pub changes: Vec<LawChangeRecord>,
}

impl Live {
    /// The player's politics; `None` without a player or without the politics data.
    pub fn read(world: &GameWorld) -> Option<Self> {
        let tag = world.player_tag()?;
        let c = world.countries.iter().position(|(_, c)| c.tag == tag)?;
        let p = &world.politics;
        if c >= p.laws.len() {
            return None;
        }
        Some(Self {
            tag: tag.to_owned(),
            laws: p.laws_in_force(c).to_vec(),
            clout: p.clout(c).to_vec(),
            legitimacy: p.legitimacy(c),
            movements: p.active_movements(c).map(|(k, m)| (k, *m)).collect(),
            pending: p.pending_laws(c).collect(),
            changes: p.law_changes(c).cloned().collect(),
        })
    }
}

/// One effect a law declares, per mille.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectValue {
    pub effect: usize,
    pub value: i32,
}

/// An effect whose combined value changes if a law replaces the one in force.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectChange {
    pub effect: usize,
    pub from: i32,
    pub to: i32,
}

/// One term of a stance: the group's weight on an effect times how far the effect moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Term {
    pub effect: usize,
    /// −3 to 3; 0 when the term is not a weight times a change.
    pub weight: i8,
    /// Per mille: the change of the combined effect for a law not in force, the law's distance
    /// from neutral for the law in force.
    pub delta: i32,
    /// `weight × delta`.
    pub value: i64,
}

/// How one interest group stands on a law.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stance {
    pub group: usize,
    /// Political weight, per mille; `None` before the simulation measured it.
    pub clout: Option<u16>,
    /// Sum of the terms.
    pub total: i64,
    /// True for the law in force: the stance is against no law at all, not a change.
    pub in_force: bool,
    pub terms: Vec<Term>,
}

/// Clout on each side of a change, per mille of the country's political weight.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Balance {
    pub for_: u32,
    pub against: u32,
    pub indifferent: u32,
}

/// Why a law cannot be proposed now. Each maps to a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blocked {
    InForce,
    /// A colonial law for a country without colonies.
    NotHeld,
    /// Already asked for.
    Pending,
}

impl Blocked {
    pub fn key(self) -> &'static str {
        match self {
            Blocked::InForce => "laws-propose-in-force",
            Blocked::NotHeld => "laws-propose-not-held",
            Blocked::Pending => "laws-propose-pending",
        }
    }
}

/// The politics catalogue and the scenario's laws, read once from `data/`.
pub struct Catalogue {
    politics: Arc<Politics>,
    in_force: LawsInForce,
    /// Country tags in the order of `in_force`, with each country's legal source.
    countries: Vec<(String, String)>,
}

impl Catalogue {
    /// Reads `data/politics/*.ron` and `data/scenario/<scenario>/{countries,laws}.ron`.
    pub fn load(data_dir: &Path, scenario: &str) -> Result<Self, String> {
        fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
            let text =
                std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
            ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
        }
        let politics = data_dir.join("politics");
        let politics = Politics::build(
            &read::<LawsFile>(&politics.join("laws.ron"))?,
            &read::<InterestGroupsFile>(&politics.join("interest_groups.ron"))?,
            &read::<MovementsFile>(&politics.join("movements.ron"))?,
        )
        .map_err(|e| e.to_string())?;
        let scenario = data_dir.join("scenario").join(scenario);
        let countries: CountriesFile = read(&scenario.join("countries.ron"))?;
        let laws: ScenarioLawsFile = read(&scenario.join("laws.ron"))?;
        let in_force = politics
            .resolve(&laws, &countries.countries)
            .map_err(|e| e.to_string())?;
        let countries = countries
            .countries
            .iter()
            .map(|c| {
                let source = laws
                    .countries
                    .iter()
                    .find(|l| l.tag == c.tag)
                    .map(|l| l.source.clone())
                    .unwrap_or_default();
                (c.tag.clone(), source)
            })
            .collect();
        Ok(Self {
            politics: Arc::new(politics),
            in_force,
            countries,
        })
    }

    /// The game's catalogue, read on first use from the game's `data/` folder; `None` when the
    /// files are missing or do not check.
    pub fn shared() -> Option<&'static Catalogue> {
        static CATALOGUE: OnceLock<Option<Catalogue>> = OnceLock::new();
        CATALOGUE
            .get_or_init(|| Catalogue::load(&crate::paths::Paths::discover().data, "1925").ok())
            .as_ref()
    }

    /// One country's laws: the simulation's when `live` is that country's, else the scenario's.
    pub fn country(&self, tag: &str, live: Option<&Live>) -> Option<LawsView> {
        let index = self.countries.iter().position(|(t, _)| t == tag)?;
        let live = live.filter(|l| l.tag == tag && l.laws.len() == self.politics.groups.len());
        let in_force = match live {
            Some(l) => l.laws.clone(),
            None => self.in_force.laws.get(index)?.clone(),
        };
        Some(LawsView {
            politics: Arc::clone(&self.politics),
            tag: tag.to_owned(),
            in_force,
            live: live.cloned(),
            source: Some(self.countries[index].1.clone()).filter(|s| !s.is_empty()),
            sent: Vec::new(),
        })
    }
}

/// The laws of one country, as the laws screen reads them.
pub struct LawsView {
    politics: Arc<Politics>,
    pub tag: String,
    /// Law in force per law group; `None` for a group the country does not hold.
    in_force: Vec<Option<u16>>,
    /// What the simulation holds; `None` outside a campaign with politics.
    live: Option<Live>,
    /// The acts and constitutions behind the laws in force on the start date.
    pub source: Option<String>,
    /// Laws the interface asked for that the snapshot does not show yet.
    sent: Vec<usize>,
}

impl LawsView {
    /// Counts `laws` as pending: commands sent that the simulation has not taken in yet.
    pub fn with_sent(mut self, laws: impl IntoIterator<Item = usize>) -> Self {
        self.sent.extend(laws);
        self
    }

    pub fn politics(&self) -> &Politics {
        &self.politics
    }

    pub fn group_count(&self) -> usize {
        self.politics.groups.len()
    }

    pub fn interest_group_count(&self) -> usize {
        self.politics.interest_groups.len()
    }

    /// Law groups by category, in display order.
    pub fn categories(&self) -> Vec<(&'static str, Vec<usize>)> {
        CATEGORIES
            .iter()
            .enumerate()
            .map(|(c, key)| {
                let groups = self
                    .politics
                    .groups
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| usize::from(g.category) == c)
                    .map(|(i, _)| i)
                    .collect();
                (*key, groups)
            })
            .collect()
    }

    pub fn group_key(&self, group: usize) -> &str {
        &self.politics.groups[group].key
    }

    pub fn law_key(&self, law: usize) -> &str {
        &self.politics.laws[law].key
    }

    pub fn interest_group_key(&self, group: usize) -> &str {
        &self.politics.interest_groups[group].key
    }

    pub fn movement_key(&self, kind: usize) -> &str {
        &self.politics.movements[kind].key
    }

    pub fn law_group(&self, law: usize) -> usize {
        usize::from(self.politics.laws[law].group)
    }

    pub fn is_targeted(&self, law: usize) -> bool {
        self.politics.laws[law].targeted
    }

    /// The laws of a group, in catalogue order.
    pub fn laws_of(&self, group: usize) -> Vec<usize> {
        self.politics.groups[group]
            .laws
            .iter()
            .map(|&l| usize::from(l))
            .collect()
    }

    pub fn in_force(&self, group: usize) -> Option<usize> {
        self.in_force.get(group).copied().flatten().map(usize::from)
    }

    pub fn live(&self) -> Option<&Live> {
        self.live.as_ref()
    }

    pub fn is_pending(&self, law: usize) -> bool {
        self.sent.contains(&law)
            || self
                .live
                .as_ref()
                .is_some_and(|l| l.pending.iter().any(|&p| usize::from(p) == law))
    }

    /// Law changes of the campaign in one group, newest first: (date, from, to, who).
    pub fn changes_in(&self, group: usize) -> Vec<(Date, usize, usize, Enactor)> {
        let Some(live) = &self.live else {
            return Vec::new();
        };
        live.changes
            .iter()
            .rev()
            .filter(|r| self.law_group(usize::from(r.to)) == group)
            .map(|r| (r.date, usize::from(r.from), usize::from(r.to), r.by))
            .collect()
    }

    /// Whether every law of the group is still the one of the start date.
    pub fn unchanged_since_start(&self, group: usize) -> bool {
        self.changes_in(group).is_empty()
    }

    /// Whom the law acts on: metropole, communities it targets, colonies.
    pub fn scopes(&self, law: usize) -> Vec<Scope> {
        let mut out = Vec::new();
        if self.politics.applies(law, Scope::Metropole) {
            out.push(Scope::Metropole);
        } else if self.politics.applies(law, Scope::Targeted) {
            out.push(Scope::Targeted);
        }
        if self.politics.applies(law, Scope::Colonies) {
            out.push(Scope::Colonies);
        }
        out
    }

    /// The effects a law declares, in the order of [`EFFECTS`].
    pub fn effects(&self, law: usize) -> Vec<EffectValue> {
        let mut out: Vec<EffectValue> = self.politics.laws[law]
            .effects
            .iter()
            .map(|&(e, value)| EffectValue {
                effect: usize::from(e),
                value,
            })
            .collect();
        out.sort_by_key(|e| e.effect);
        out
    }

    /// A state holding only this country's laws, for the simulation's stance function.
    fn state(&self) -> PoliticsState {
        PoliticsState {
            laws: vec![self.in_force.clone()],
            ..Default::default()
        }
    }

    /// What changes, effect by effect, for the people the law falls on, if `law` replaces the
    /// law in force in its group: the combined value of each effect under the whole set of
    /// laws, as the simulation reads it. Empty for the law in force.
    pub fn changes(&self, law: usize) -> Vec<EffectChange> {
        let group = self.law_group(law);
        if self.in_force(group).is_none_or(|c| c == law) {
            return Vec::new();
        }
        let scope = polity::scope_of(&self.politics, law);
        let before = self.politics.effects(&self.in_force, scope);
        let mut proposed = self.in_force.clone();
        proposed[group] = Some(law as u16);
        let after = self.politics.effects(&proposed, scope);
        (0..EFFECTS.len())
            .filter(|&e| before[e] != after[e])
            .map(|effect| EffectChange {
                effect,
                from: before[effect],
                to: after[effect],
            })
            .collect()
    }

    /// Every interest group's stance on `law`, strongest support first and the indifferent last, with the sums open term
    /// by term. For a law not in force it is the simulation's stance on the change
    /// (`polity::stance`); for the law in force, what the law gives the group against no law.
    pub fn stances(&self, law: usize) -> Vec<Stance> {
        let p = &self.politics;
        let in_force = self.in_force(self.law_group(law)) == Some(law);
        let state = self.state();
        let mut out: Vec<Stance> = p
            .interest_groups
            .iter()
            .enumerate()
            .map(|(g, group)| {
                let raw: Vec<(u8, i64)> = if in_force {
                    p.terms(g, law)
                } else {
                    polity::stance(p, &state, 0, g, law).terms
                };
                let terms: Vec<Term> = raw
                    .into_iter()
                    .map(|(e, value)| {
                        let weight = group
                            .interests
                            .iter()
                            .find(|(x, _)| *x == e)
                            .map_or(0, |(_, w)| *w);
                        // A term the simulation does not build as weight × change (none
                        // today) keeps only its value: weight 0 says so.
                        let (weight, delta) = if weight != 0 && value % i64::from(weight) == 0 {
                            (weight, (value / i64::from(weight)) as i32)
                        } else {
                            (0, 0)
                        };
                        Term {
                            effect: usize::from(e),
                            weight,
                            delta,
                            value,
                        }
                    })
                    .collect();
                Stance {
                    group: g,
                    clout: self.clout(g),
                    total: terms.iter().map(|t| t.value).sum(),
                    in_force,
                    terms,
                }
            })
            .collect();
        // Support first, then opposition, the indifferent last.
        out.sort_by_key(|s| (s.total == 0, std::cmp::Reverse(s.total), s.group));
        out
    }

    /// Political weight of an interest group, per mille; `None` before it was measured.
    pub fn clout(&self, group: usize) -> Option<u16> {
        let live = self.live.as_ref()?;
        if live.clout.iter().all(|&c| c == 0) {
            return None;
        }
        live.clout.get(group).copied()
    }

    /// How the country's political weight splits on a set of stances.
    pub fn balance(stances: &[Stance]) -> Option<Balance> {
        let mut b = Balance::default();
        for s in stances {
            let c = u32::from(s.clout?);
            match s.total.signum() {
                1 => b.for_ += c,
                -1 => b.against += c,
                _ => b.indifferent += c,
            }
        }
        Some(b)
    }

    /// The command that asks for `law`, or why it cannot be asked for now.
    pub fn propose(&self, law: usize) -> Result<GameCommand, Blocked> {
        match self.in_force(self.law_group(law)) {
            None => Err(Blocked::NotHeld),
            Some(c) if c == law => Err(Blocked::InForce),
            Some(_) if self.is_pending(law) => Err(Blocked::Pending),
            Some(_) => Ok(GameCommand::SetLaw {
                tag: self.tag.clone(),
                law: law as u16,
            }),
        }
    }
}

/// How an effect's value reads: a ceiling, a share or a multiplier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    /// Per mille ceiling below 1000.
    Cap(i32),
    NoCap,
    /// Per mille share, signed.
    Share(i32),
    /// Per mille multiplier.
    Multiplier(i32),
}

pub fn reading(effect: usize, value: i32) -> Reading {
    match EFFECTS[effect].combine {
        Combine::Cap if value >= 1000 => Reading::NoCap,
        Combine::Cap => Reading::Cap(value),
        Combine::Add => Reading::Share(value),
        Combine::Multiply => Reading::Multiplier(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalogue() -> Catalogue {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        Catalogue::load(&root.join("data"), "1925").expect("politics data")
    }

    #[test]
    fn every_held_group_has_a_law_and_colonies_only_with_colonies() {
        let cat = catalogue();
        let bra = cat.country("BRA", None).unwrap();
        let fra = cat.country("FRA", None).unwrap();
        for g in 0..bra.group_count() {
            let colonial = bra.politics().groups[g].colonial;
            assert_eq!(bra.in_force(g).is_none(), colonial, "{}", bra.group_key(g));
            assert!(fra.in_force(g).is_some(), "{}", fra.group_key(g));
        }
        assert!(bra.source.as_deref().is_some_and(|s| s.contains("1891")));
        assert_eq!(bra.categories().len(), 7);
    }

    #[test]
    fn stances_add_up_term_by_term_and_match_the_simulation() {
        let cat = catalogue();
        let bra = cat.country("BRA", None).unwrap();
        let p = bra.politics();
        let taxation = p.group("taxation").unwrap();
        let current = bra.in_force(taxation).unwrap();
        let other = bra
            .laws_of(taxation)
            .into_iter()
            .find(|&l| l != current)
            .unwrap();
        let state = bra.state();
        for s in bra.stances(other) {
            assert!(!s.in_force);
            assert_eq!(s.total, polity::stance(p, &state, 0, s.group, other).total);
            for t in s.terms.iter().filter(|t| t.weight != 0) {
                assert_eq!(t.value, i64::from(t.weight) * i64::from(t.delta));
            }
        }
        for s in bra.stances(current) {
            assert!(s.in_force);
            assert_eq!(s.total, p.score(s.group, current));
        }
        assert_eq!(bra.stances(current).len(), bra.interest_group_count());
        assert!(!bra.changes(other).is_empty());
        assert!(bra.changes(current).is_empty());
        assert_eq!(bra.propose(current), Err(Blocked::InForce));
        assert_eq!(
            bra.propose(other),
            Ok(GameCommand::SetLaw {
                tag: "BRA".into(),
                law: other as u16
            })
        );
        let colonial = p.group("colonial_rule").unwrap();
        assert_eq!(bra.propose(bra.laws_of(colonial)[0]), Err(Blocked::NotHeld));
    }

    #[test]
    fn live_laws_replace_the_scenario() {
        let cat = catalogue();
        let base = cat.country("BRA", None).unwrap();
        let taxation = base.politics().group("taxation").unwrap();
        let other = base
            .laws_of(taxation)
            .into_iter()
            .find(|&l| Some(l) != base.in_force(taxation))
            .unwrap();
        let mut laws: Vec<Option<u16>> = (0..base.group_count())
            .map(|g| base.in_force(g).map(|l| l as u16))
            .collect();
        laws[taxation] = Some(other as u16);
        let live = Live {
            tag: "BRA".into(),
            laws,
            clout: vec![125; base.interest_group_count()],
            legitimacy: 900,
            movements: Vec::new(),
            pending: Vec::new(),
            changes: Vec::new(),
        };
        let view = cat.country("BRA", Some(&live)).unwrap();
        assert_eq!(view.in_force(taxation), Some(other));
        let stances = view.stances(base.in_force(taxation).unwrap());
        let b = LawsView::balance(&stances).unwrap();
        assert_eq!(b.for_ + b.against + b.indifferent, 1000);
        // Another country's figures are not this one's.
        assert_eq!(cat.country("ARG", Some(&live)).unwrap().live(), None);
    }

    #[test]
    fn readings_follow_the_way_effects_combine() {
        let cap = bf_sim::politics::effect_index("tax_cap_lower").unwrap();
        assert_eq!(reading(cap, 150), Reading::Cap(150));
        assert_eq!(reading(cap, 1000), Reading::NoCap);
        let mig = bf_sim::politics::effect_index("migration_within_state").unwrap();
        assert_eq!(reading(mig, 250), Reading::Multiplier(250));
    }
}
