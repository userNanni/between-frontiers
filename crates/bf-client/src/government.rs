//! The player's treasury, currency and debt, copied out of the world for the government window.
//!
//! The simulation keeps last week's accounts; this view keeps them as they are (weekly, in real
//! dollars at 1925 base prices) and the screens say how they turn them into yearly figures.

use bf_sim::econ::money::Money;
use bf_sim::econ::public::CLASS_OF;
use bf_sim::econ::state::{Budget, Laws, TAX_CLASSES};
use bf_sim::game::GameWorld;
use laplace_schelling::Fixed;
use laplace_schelling::calendar::Date;

/// A work in the player's construction queue.
#[derive(Clone, Debug, PartialEq)]
pub struct Work {
    /// Index into the catalogue's buildings.
    pub kind: u16,
    /// Map state the work is in (0: unknown).
    pub state: u16,
    pub public: bool,
    /// Construction points the work costs and those still missing.
    pub points: f64,
    pub left: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GovernmentView {
    /// Date the view was read on.
    pub date: Date,
    pub budget: Budget,
    pub money: Money,
    pub laws: Laws,
    /// Cash in the treasury (negative: bills nobody would lend for).
    pub treasury: f64,
    /// GDP of the last full year, real dollars.
    pub gdp_last_year: f64,
    /// Workers on the state payroll (civil servants, clergy, soldiers).
    pub gov_workers: i64,
    /// Last week's income after tax of each tax class, and the people in it.
    pub class_income: [f64; TAX_CLASSES],
    pub class_people: [i64; TAX_CLASSES],
    /// Whether the central bank lends the treasury what nobody else will, this year.
    pub subordinate: bool,
    /// The construction queue, oldest first.
    pub works: Vec<Work>,
}

fn f(x: Fixed) -> f64 {
    x.to_f64_lossy()
}

impl GovernmentView {
    /// Tax each class paid last week: its income after tax grossed up by its rate. The rates
    /// may have moved since the week was settled, so the classes are scaled to the treasury's
    /// own total.
    pub fn tax_by_class(&self) -> [f64; TAX_CLASSES] {
        let mut out = [0.0; TAX_CLASSES];
        for (c, o) in out.iter_mut().enumerate() {
            let rate = f(self.budget.tax_rates[c]).min(0.99);
            *o = self.class_income[c] * rate / (1.0 - rate);
        }
        let sum: f64 = out.iter().sum();
        let total = f(self.budget.revenue);
        if sum > 0.0 {
            for o in &mut out {
                *o *= total / sum;
            }
        }
        out
    }

    /// Last week's spending: payroll, goods, public services, public works, interest.
    pub fn spending(&self) -> [f64; 5] {
        let b = &self.budget;
        [
            f(b.payroll),
            f(b.goods),
            f(b.services),
            f(b.works),
            f(b.interest),
        ]
    }

    /// Last week's balance: taxes minus spending (bonds sold cover the rest).
    pub fn balance(&self) -> f64 {
        f(self.budget.revenue) - self.spending().iter().sum::<f64>()
    }

    /// Cash the treasury keeps before it pays for new public works.
    pub fn reserve(&self) -> f64 {
        f(self.budget.revenue_avg) * bf_sim::econ::public::RESERVE_WEEKS as f64
    }
}

/// Reads the player's country; `None` without a player or an economy.
pub fn read(world: &GameWorld, date: Date) -> Option<GovernmentView> {
    let tag = world.player_tag()?;
    let ci = world.countries.iter().position(|(_, c)| c.tag == tag)?;
    let m = world.econ.markets.get(ci)?;
    let mut class_income = [0.0; TAX_CLASSES];
    let mut class_people = [0i64; TAX_CLASSES];
    for r in world
        .econ
        .regions
        .iter()
        .filter(|r| usize::from(r.market) == ci)
    {
        for (pi, p) in r.pops.iter().enumerate() {
            let c = CLASS_OF.get(pi).copied().unwrap_or(0);
            class_income[c] += f(p.income).max(0.0);
            class_people[c] += p.size;
        }
    }
    let works = m
        .queue
        .iter()
        .map(|p| Work {
            kind: p.kind,
            state: world
                .econ
                .regions
                .get(p.region as usize)
                .map_or(0, |r| r.state),
            public: p.public,
            points: f(p.points),
            left: f(p.left),
        })
        .collect();
    Some(GovernmentView {
        date,
        budget: m.budget.clone(),
        money: m.money.clone(),
        laws: m.laws.clone(),
        treasury: f(m.treasury),
        gdp_last_year: f(m.gdp_last_year),
        gov_workers: m.gov_workers,
        class_income,
        class_people,
        subordinate: m.money.subordinate(date.year),
        works,
    })
}
