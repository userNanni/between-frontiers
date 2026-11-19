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
    /// The budget levers (D1, D2, D4, D5): what the rule gives, what is fixed, the limits.
    pub levers: Option<bf_sim::econ::levers::LeversView>,
    /// The state's purchases and their military share (D3); filled where the catalogue is known.
    pub purchases: Option<bf_sim::econ::levers::PurchasesView>,
    /// Where the treasury issues (D9).
    pub issuance: Option<bf_sim::econ::levers::IssuanceView>,
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
    /// The terms of this week's purchases rule.
    pub goods_rule: GoodsRule,
    /// Last week's research and development: what the treasury paid (inside public services)
    /// and what the firms' fund paid, and the treasury's legal share (`research_state_share`).
    pub research: [f64; 2],
    pub research_state_share: f64,
    /// The adult literacy campaign, running or possible; `None` without schooling data.
    pub campaign: Option<bf_sim::demography::education::CampaignOutlook>,
    /// Last week's customs duties (`econ::customs`), inside `Budget::revenue`.
    pub customs: f64,
    /// The balance of payments of the last full year (`econ::payments::view`).
    pub payments: bf_sim::econ::payments::PaymentsView,
}

/// The budget rule that sets the state's purchases (`econ::public::goods_budget`), term by
/// term, weekly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GoodsRule {
    /// Average weekly revenue and the deficit the debt rule allows.
    pub revenue: f64,
    pub deficit_room: f64,
    /// What comes off: payroll, public services, public works, interest.
    pub committed: f64,
    /// The floor: a share of the payroll.
    pub floor: f64,
    /// Share of the plan kept (cash the treasury could not borrow, interest above a tenth of
    /// revenue), 0.25–1.
    pub austerity: f64,
    /// The rule's total, with a tenth of the cash beyond twice the reserve on top.
    pub total: f64,
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
        let total = self.income_tax();
        if sum > 0.0 {
            for o in &mut out {
                *o *= total / sum;
            }
        }
        out
    }

    /// Last week's income tax: the revenue less the tax on consumption, the profit of state
    /// firms and the customs duties, which `Budget::revenue` holds too.
    pub fn income_tax(&self) -> f64 {
        let b = &self.budget;
        (f(b.revenue) - f(b.revenue_indirect) - f(b.revenue_firms) - self.customs).max(0.0)
    }

    /// Last week's revenue by source: income tax, tax on consumption, profit of state firms,
    /// customs duties. They add up to `Budget::revenue`.
    pub fn revenue_sources(&self) -> [f64; 4] {
        let b = &self.budget;
        [
            self.income_tax(),
            f(b.revenue_indirect),
            f(b.revenue_firms),
            self.customs,
        ]
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
pub fn read(
    world: &GameWorld,
    tag: Option<&str>,
    date: Date,
    research: Option<u16>,
    education: Option<(&bf_sim::demography::education::EducationRules, f64)>,
) -> Option<GovernmentView> {
    let tag = tag?;
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
        research: {
            let (state, firms) = bf_sim::econ::public::research_spending_of(m, research);
            [f(state), f(firms)]
        },
        research_state_share: research
            .map_or(0.0, |g| f(m.laws.effects[0].public_share(usize::from(g)))),
        levers: Some(bf_sim::econ::levers::view(m)),
        purchases: None,
        issuance: Some(bf_sim::econ::levers::issuance_view(m)),
        campaign: education.and_then(|(rules, price)| {
            bf_sim::demography::education::campaign_outlook(world, rules, price, ci)
        }),
        customs: world.tariffs.revenue.get(ci).map_or(0.0, |r| f(*r)),
        payments: bf_sim::econ::payments::view(world, ci),
        goods_rule: {
            use bf_sim::econ::public::{GOODS_PER_PAYROLL, austerity, goods_budget};
            let b = &m.budget;
            GoodsRule {
                revenue: f(b.revenue_avg),
                deficit_room: f(bf_sim::econ::money::deficit_room(m)),
                committed: f(b.payroll + b.services + b.works + b.interest),
                floor: f(b.payroll * GOODS_PER_PAYROLL),
                austerity: f(austerity(m)),
                total: f(goods_budget(m)),
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(revenue: i64, indirect: i64, firms: i64) -> GovernmentView {
        let budget = Budget {
            revenue: Fixed::from_int(revenue),
            revenue_indirect: Fixed::from_int(indirect),
            revenue_firms: Fixed::from_int(firms),
            ..Budget::default()
        };
        GovernmentView {
            date: Date::new(1940, 1, 1),
            budget,
            money: Money::default(),
            laws: Laws::default(),
            treasury: 0.0,
            gdp_last_year: 0.0,
            gov_workers: 0,
            class_income: [100.0, 50.0, 20.0],
            class_people: [10, 5, 1],
            subordinate: false,
            works: Vec::new(),
            goods_rule: GoodsRule::default(),
            research: [0.0; 2],
            research_state_share: 0.0,
            campaign: None,
            customs: 0.0,
            payments: Default::default(),
            levers: None,
            purchases: None,
            issuance: None,
        }
    }

    #[test]
    fn the_sources_add_up_to_the_revenue_and_the_classes_to_the_income_tax() {
        let v = view(100, 15, 5);
        let [income, indirect, firms, customs] = v.revenue_sources();
        assert_eq!((income, indirect, firms, customs), (80.0, 15.0, 5.0, 0.0));
        assert!((income + indirect + firms - 100.0).abs() < 1e-9);
        let classes: f64 = v.tax_by_class().iter().sum();
        assert!((classes - income).abs() < 1e-6, "{classes}");
        // Without the new sources, the classes take the whole revenue, as before.
        let v = view(100, 0, 0);
        assert!((v.tax_by_class().iter().sum::<f64>() - 100.0).abs() < 1e-6);
        // The customs duties come off the income tax, not on top of the revenue.
        let v = GovernmentView {
            customs: 10.0,
            ..view(100, 15, 5)
        };
        let s = v.revenue_sources();
        assert_eq!(s, [70.0, 15.0, 5.0, 10.0]);
        assert!((s.iter().sum::<f64>() - 100.0).abs() < 1e-9);
    }
}
