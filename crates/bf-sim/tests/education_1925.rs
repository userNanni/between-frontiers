//! Schooling of 1925 (`data/scenario/1925/education.ron`, from Lee and Lee 2016): the data are
//! whole, and the start the simulation grows from the enrolment of 1865–1925 lands near the
//! attainment Lee and Lee give for 1925 (`cargo test -p bf-sim --test education_1925 --
//! --nocapture` prints the table).

use std::path::PathBuf;

use bf_sim::data::GameData;
use bf_sim::demography::AGE_GROUPS;
use bf_sim::demography::education::{WHOLE, initial};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A pyramid of 1925's poorer world: each group 8 % smaller than the one below.
fn pyramid() -> [i64; AGE_GROUPS] {
    let mut a = [0i64; AGE_GROUPS];
    let mut n = 1_000_000i64;
    for g in a.iter_mut() {
        *g = n;
        n = n * 92 / 100;
    }
    a
}

#[test]
fn units_are_whole_and_the_grown_start_meets_lee_and_lee() {
    let data = GameData::load(&root()).unwrap();
    let file = data.education_1925.as_ref().expect("education.ron");
    let campaign = data
        .literacy_campaign
        .as_ref()
        .expect("literacy_campaign.ron");
    assert!(campaign.yearly() > 0 && campaign.years > 0);
    assert_eq!((file.first_year, file.step), (1865, 5));
    let tags: Vec<&str> = data
        .countries
        .countries
        .iter()
        .map(|c| c.tag.as_str())
        .collect();
    let ages = pyramid();
    let (mut sum, mut n) = (0.0, 0);
    println!(
        "unit             schooled 15–64: grown / Lee and Lee (primary, secondary, tertiary %)"
    );
    for u in &file.units {
        assert!(tags.contains(&u.tag.as_str()), "{} not a country", u.tag);
        assert_eq!(u.enrolment.len(), 13, "{}", u.tag);
        for &(p, s, t) in &u.enrolment {
            for v in [p, s, t] {
                assert!((0.0..=100.0).contains(&v), "{} {v}", u.tag);
            }
        }
        let path = file.path(&u.tag, u.colonies).unwrap();
        // Literacy only sets those who read without school; Lee and Lee count them unschooled.
        let (_, lp, ls, lh) = u.attainment_15_64;
        let per = |v: f32| (f64::from(v) * 100.0).round() as u32;
        let attained = [per(lp + ls + lh), per(ls + lh), per(lh)];
        let raw = initial(&ages, &path, None, 0);
        let s = initial(&ages, &path, Some(attained), 0);
        let young_ahead = |x: &bf_sim::demography::education::Schooling| {
            let at = |g: usize| x.levels[g][1..].iter().map(|&v| u32::from(v)).sum::<u32>();
            at(3) >= at(12)
        };
        assert!(
            young_ahead(&s) || !young_ahead(&raw),
            "{}: gradient kept",
            u.tag
        );
        let people = s.people(&ages);
        let (mut adults, mut by) = (0i64, [0i64; 3]);
        for row in &people[3..=12] {
            adults += row.iter().sum::<i64>();
            for k in 0..3 {
                by[k] += row[k + 2];
            }
        }
        let grown: Vec<f64> = by
            .iter()
            .map(|&v| v as f64 * 100.0 / adults as f64)
            .collect();
        let lee = [f64::from(lp), f64::from(ls), f64::from(lh)];
        let schooled_grown: f64 = grown.iter().sum();
        let schooled_lee: f64 = lee.iter().sum();
        sum += (schooled_grown - schooled_lee).abs();
        n += 1;
        if [
            "USA", "GBR", "FRA", "GER", "ITA", "JAP", "RAJ", "CHN", "BRA", "SOV",
        ]
        .contains(&u.tag.as_str())
            && !u.colonies
        {
            println!(
                "{:4} {:9} {:5.1} {:5.1} {:5.1} / {:5.1} {:5.1} {:5.1}",
                u.tag,
                if u.colonies { "colonies" } else { "metropole" },
                grown[0],
                grown[1],
                grown[2],
                lee[0],
                lee[1],
                lee[2]
            );
        }
    }
    let mean = sum / f64::from(n);
    println!("mean absolute gap in the schooled share of 15–64: {mean:.1} points over {n} units");
    assert!(mean < 1.0, "{mean}");
    assert!(WHOLE == 10_000);
}
