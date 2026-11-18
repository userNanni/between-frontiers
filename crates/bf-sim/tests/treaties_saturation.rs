//! The negotiation table with sums of billions (reference units): the counterproposal lowers a
//! clause without saturating `Fixed` (the account of Britain asking France a tribute of 1 % of
//! its GDP a year for a payment of 1 % once), and, built with `--features saturation-count`, no
//! arithmetic of the treaties hits the bounds over a set of proposals between large economies.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::GameWorld;
use bf_sim::treaty::{Clause, ClauseCatalogue, ClauseKind, DebtTerms, Proposal, Side};
use bf_sim::treaty_table::{ProposalAnswer, account, respond};
use laplace_schelling::Fixed;

fn data() -> &'static GameData {
    static D: OnceLock<GameData> = OnceLock::new();
    D.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn cat() -> &'static ClauseCatalogue {
    data().clauses.as_ref().unwrap()
}

fn gdp(w: &GameWorld, tag: &str) -> Fixed {
    let i = w.countries.iter().position(|(_, c)| c.tag == tag).unwrap();
    w.econ.markets[i].gdp_last_year
}

fn clause(kind: ClauseKind, giver: Side, receiver: Side) -> Clause {
    Clause {
        kind,
        giver,
        receiver,
        until: None,
        note: None,
    }
}

fn proposal(a: &str, b: &str, clauses: Vec<Clause>) -> Proposal {
    Proposal {
        side_a: vec![a.into()],
        side_b: vec![b.into()],
        clauses,
        term_years: None,
        notice_years: 0,
    }
}

/// The case of the table's screen (#210): Britain pays 1 % of its GDP once and asks France a
/// tribute of 1 % of France's GDP a year. France cannot take it as it is, but lowering the
/// tribute closes its account: it counters, it does not refuse.
#[test]
fn a_counterproposal_with_billions_lowers_the_tribute() {
    let w = data().new_world();
    let (g, f) = (gdp(&w, "GBR"), gdp(&w, "FRA"));
    assert!(g > Fixed::from_int(100_000_000_000), "billions: {g:?}");
    let p = proposal(
        "GBR",
        "FRA",
        vec![
            clause(
                ClauseKind::Payment {
                    amount: g.mul_ratio(1, 100),
                },
                Side::Senior,
                Side::Junior,
            ),
            clause(
                ClauseKind::Tribute {
                    yearly: f.mul_ratio(1, 100),
                },
                Side::Junior,
                Side::Senior,
            ),
        ],
    );
    assert!(account(&w, cat(), &p, "FRA").total < Fixed::ZERO);
    let (answer, why) = respond(&w, cat(), &p, "FRA", true);
    let ProposalAnswer::Counter(q) = answer else {
        panic!("France refuses ({why:?}) where lowering the tribute closes its account")
    };
    assert!(account(&w, cat(), &q, "FRA").total > Fixed::ZERO);
    let tribute = |p: &Proposal| {
        p.clauses.iter().find_map(|c| match c.kind {
            ClauseKind::Tribute { yearly } => Some(yearly),
            _ => None,
        })
    };
    // Lowered or dropped, never raised.
    assert!(tribute(&q).is_none_or(|t| t < f.mul_ratio(1, 100)));
}

/// Many proposals between the largest economies, each priced, answered and checked against the
/// limits. With `--features saturation-count` no operation of the treaties may saturate.
#[test]
fn the_table_does_not_saturate_with_large_economies() {
    let w = data().new_world();
    let tags = ["USA", "GBR", "FRA", "GER", "JAP", "SOV", "ITA", "CHN"];
    let mut n = 0;
    for a in tags {
        for b in tags {
            if a == b {
                continue;
            }
            let (ga, gb) = (gdp(&w, a), gdp(&w, b));
            for share in [1i64, 10, 50, 150] {
                let debt = DebtTerms {
                    principal: gb.mul_ratio(share, 100),
                    rate: Fixed::from_ratio(5, 100),
                    annuity: gb.mul_ratio(share, 2000),
                    years: 40,
                    currency: Default::default(),
                    deferrable: Fixed::ZERO,
                    transfer_cap: None,
                    pledge: None,
                    contingent: Fixed::ZERO,
                    grace_years: 0,
                };
                let p = proposal(
                    a,
                    b,
                    vec![
                        clause(
                            ClauseKind::Payment {
                                amount: ga.mul_ratio(share, 100),
                            },
                            Side::Senior,
                            Side::Junior,
                        ),
                        clause(
                            ClauseKind::Tribute {
                                yearly: gb.mul_ratio(share, 1000),
                            },
                            Side::Junior,
                            Side::Senior,
                        ),
                        clause(
                            ClauseKind::Debt { terms: Some(debt) },
                            Side::Junior,
                            Side::Senior,
                        ),
                        clause(ClauseKind::ForeignPolicy, Side::Junior, Side::Senior),
                    ],
                );
                let _ = account(&w, cat(), &p, a);
                let _ = account(&w, cat(), &p, b);
                let _ = respond(&w, cat(), &p, b, true);
                let gdp_map = bf_sim::treaty_table::Inputs::of(&w).gdp;
                let _ = bf_sim::treaty::limit_violations(cat(), &p, &gdp_map);
                n += 1;
            }
        }
    }
    assert!(n > 200);
    #[cfg(feature = "saturation-count")]
    {
        let treaties: Vec<_> = laplace_schelling::fixed::saturation::report()
            .into_iter()
            .filter(|(f, ..)| f.contains("treaty"))
            .collect();
        assert!(treaties.is_empty(), "saturated: {treaties:#?}");
    }
}
