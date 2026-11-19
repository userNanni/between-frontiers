//! A surcharge past an agreement's tariff clause (owner's decision CT2, docs/trade.pt-BR.md,
//! section 13): without `breach` the clause still holds the tariff; with it the clause is broken
//! (a breach on the record) and no longer holds the tariff between the two.

use std::path::PathBuf;
use std::sync::OnceLock;

use bf_sim::data::GameData;
use bf_sim::game::{GameCommand, Session};
use laplace_schelling::Date;

fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    })
}

fn session() -> Session {
    let d = data();
    Session::new(
        d.new_world(),
        1925,
        Date::new(1925, 1, 1),
        2,
        Some(d.rules()),
    )
}

/// A tariff clause in force in 1925 (only customs unions are, the caps being frozen tariffs):
/// (giver, receiver).
fn a_tariff_clause(s: &Session) -> (String, String) {
    let tags: Vec<String> = s
        .world()
        .countries
        .iter()
        .map(|(_, c)| c.tag.clone())
        .collect();
    let link = s
        .world()
        .treaties
        .tariff_relations(&tags, "1925-01-05")
        .into_iter()
        .next()
        .expect("a customs union in force in 1925");
    (
        tags[usize::from(link.giver)].clone(),
        tags[usize::from(link.receiver)].clone(),
    )
}

fn links(s: &Session, giver: &str, receiver: &str) -> usize {
    let tags: Vec<String> = s
        .world()
        .countries
        .iter()
        .map(|(_, c)| c.tag.clone())
        .collect();
    let (gi, ri) = (
        tags.iter().position(|t| t == giver).unwrap(),
        tags.iter().position(|t| t == receiver).unwrap(),
    );
    s.world()
        .treaties
        .tariff_relations(&tags, "1925-01-05")
        .iter()
        .filter(|l| usize::from(l.giver) == gi && usize::from(l.receiver) == ri)
        .count()
}

#[test]
fn a_surcharge_breaks_the_clause_only_when_asked() {
    let (mut held, mut broken) = (session(), session());
    held.sim.run_until(Date::new(1925, 1, 2));
    broken.sim.run_until(Date::new(1925, 1, 2));
    let (giver, receiver) = a_tariff_clause(&held);
    assert!(links(&held, &giver, &receiver) > 0);
    for (s, breach) in [(&mut held, false), (&mut broken, true)] {
        s.submit(GameCommand::SetPartnerTariff {
            tag: giver.clone(),
            partner: receiver.clone(),
            per_mille: Some(200),
            breach,
        });
        s.sim.run_until(Date::new(1925, 1, 4));
    }
    assert!(held.world().treaties.breaches.is_empty());
    assert!(
        links(&held, &giver, &receiver) > 0,
        "the clause still holds"
    );
    let b = &broken.world().treaties.breaches;
    assert!(
        b.iter()
            .any(|b| b.by == giver && b.harmed.contains(&receiver)),
        "a breach of {giver} against {receiver}"
    );
    assert_eq!(
        links(&broken, &giver, &receiver),
        0,
        "the clause no longer holds"
    );
}
