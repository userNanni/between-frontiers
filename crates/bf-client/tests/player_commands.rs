//! Every command a player gives the simulation can be given from the interface: each player
//! variant of `GameCommand` has a command in the keys registry that reaches the screen, and that
//! screen's code issues it. A new variant fails here until it has both, or until it joins the
//! exceptions with its reason. The table is also `docs/wiki/o-que-o-jogador-decide.md`.

use std::collections::BTreeSet;
use std::path::PathBuf;

use bf_client::keys::{Registry, id};
use bf_sim::game::GameCommand;
use serde_reflection::{ContainerFormat, Tracer, TracerConfig};

/// A player's command: the registry command that reaches it and the files that issue it.
struct Surface {
    variant: &'static str,
    command: &'static str,
    files: &'static [&'static str],
}

const PLAYER: &[Surface] = &[
    Surface {
        variant: "ChoosePlayer",
        command: id::PICK_FIND,
        files: &["screens/game.rs"],
    },
    Surface {
        variant: "AnswerDemand",
        command: id::JOURNAL,
        files: &["screens/game.rs", "screens/demand.rs"],
    },
    Surface {
        variant: "SetResearch",
        command: id::RESEARCH,
        files: &["screens/research.rs", "screens/game.rs"],
    },
    Surface {
        variant: "SetTaxRates",
        command: id::TAB_BUDGET,
        files: &["screens/budget.rs"],
    },
    Surface {
        variant: "SetPublicPriority",
        command: id::TAB_BUDGET,
        files: &["screens/budget.rs"],
    },
    Surface {
        variant: "LiteracyCampaign",
        command: id::TAB_BUDGET,
        files: &["screens/budget.rs"],
    },
    Surface {
        variant: "SetMigrationLaw",
        command: id::TAB_BUDGET,
        files: &["screens/budget.rs"],
    },
    Surface {
        variant: "SetPolicyRate",
        command: id::TAB_MONEY,
        files: &["screens/money.rs"],
    },
    Surface {
        variant: "SetCentralBank",
        command: id::TAB_MONEY,
        files: &["screens/money.rs"],
    },
    Surface {
        variant: "SetLaw",
        command: id::PROPOSE_LAW,
        files: &["laws_view.rs", "screens/laws.rs"],
    },
    Surface {
        variant: "SwitchCountry",
        command: id::CAMPAIGN_MENU,
        files: &["screens/game.rs", "screens/saves.rs"],
    },
    Surface {
        variant: "Liberate",
        command: id::JOURNAL,
        files: &["screens/game.rs", "screens/journal.rs"],
    },
    Surface {
        variant: "OrderForces",
        command: id::TAB_FORCES,
        files: &["screens/forces.rs"],
    },
    Surface {
        variant: "DisbandForces",
        command: id::TAB_FORCES,
        files: &["screens/forces.rs"],
    },
    Surface {
        variant: "CancelOrder",
        command: id::TAB_FORCES,
        files: &["screens/forces.rs"],
    },
    Surface {
        variant: "SetPayroll",
        command: id::TAB_BUDGET,
        files: &["screens/levers.rs"],
    },
    Surface {
        variant: "SetPublicPay",
        command: id::TAB_BUDGET,
        files: &["screens/levers.rs"],
    },
    Surface {
        variant: "SetTreasury",
        command: id::TAB_BUDGET,
        files: &["screens/levers.rs"],
    },
    Surface {
        variant: "SetDeficitCap",
        command: id::TAB_BUDGET,
        files: &["screens/levers.rs"],
    },
    Surface {
        variant: "Demand",
        command: id::JOURNAL,
        files: &["screens/game.rs", "screens/crisis.rs"],
    },
    Surface {
        variant: "AnswerCrisis",
        command: id::JOURNAL,
        files: &["screens/game.rs", "screens/crisis.rs"],
    },
    Surface {
        variant: "Declare",
        command: id::JOURNAL,
        files: &["screens/game.rs", "screens/crisis.rs"],
    },
    Surface {
        variant: "SetWorksOrder",
        command: id::TAB_WORKS,
        files: &["screens/works.rs"],
    },
    Surface {
        variant: "AddWork",
        command: id::TAB_WORKS,
        files: &["screens/works.rs"],
    },
    Surface {
        variant: "CancelWork",
        command: id::TAB_WORKS,
        files: &["screens/works.rs"],
    },
    Surface {
        variant: "SetStatePurchases",
        command: id::TAB_BUDGET,
        files: &["screens/levers.rs"],
    },
    Surface {
        variant: "SetIssuance",
        command: id::TAB_MONEY,
        files: &["screens/issuance.rs"],
    },
    Surface {
        variant: "AnswerCall",
        command: id::JOURNAL,
        files: &["screens/game.rs", "screens/crisis.rs"],
    },
    Surface {
        variant: "ProposeTreaty",
        command: id::JOURNAL,
        files: &["screens/game.rs", "screens/treaty_table.rs"],
    },
    Surface {
        variant: "AnswerProposal",
        command: id::JOURNAL,
        files: &["screens/game.rs", "screens/treaty_table.rs"],
    },
    Surface {
        variant: "SetPosture",
        command: id::WAR,
        files: &["screens/war.rs"],
    },
    Surface {
        variant: "Denounce",
        command: id::JOURNAL,
        files: &["screens/game.rs", "screens/agreements.rs"],
    },
    Surface {
        variant: "Mobilize",
        command: id::JOURNAL,
        files: &["screens/game.rs", "screens/journal.rs"],
    },
];

/// Variants the player does not give from a screen with a key, and why.
const EXCEPTIONS: &[(&str, &str)] = &[(
    "Borders",
    "issued by treaties, peace, tests and tools; the player frees its land with Liberate",
)];

fn variants() -> BTreeSet<String> {
    let mut tracer = Tracer::new(TracerConfig::default());
    tracer
        .trace_simple_type::<GameCommand>()
        .expect("tracing GameCommand");
    let registry = tracer.registry_unchecked();
    match registry.get("GameCommand") {
        Some(ContainerFormat::Enum(v)) => v.values().map(|n| n.name.clone()).collect(),
        other => panic!("GameCommand is not an enum: {other:?}"),
    }
}

#[test]
fn every_player_command_has_a_key_and_a_screen() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let registry = Registry::builtin();
    let listed: BTreeSet<String> = PLAYER
        .iter()
        .map(|s| s.variant.to_owned())
        .chain(EXCEPTIONS.iter().map(|(v, _)| (*v).to_owned()))
        .collect();
    let all = variants();
    let mut failures = Vec::new();
    for v in all.difference(&listed) {
        failures.push(format!(
            "GameCommand::{v} has no screen and no key: give it both and a line in PLAYER, or \
             a line in EXCEPTIONS with the reason (and in docs/wiki/o-que-o-jogador-decide.md)"
        ));
    }
    for v in listed.difference(&all) {
        failures.push(format!("{v} is listed but GameCommand has no such variant"));
    }
    for s in PLAYER {
        if registry.get(s.command).is_none() {
            failures.push(format!(
                "{}: no command {} in the registry",
                s.variant, s.command
            ));
        }
        let pattern = format!("GameCommand::{}", s.variant);
        let issued = s.files.iter().any(|f| {
            std::fs::read_to_string(src.join(f)).is_ok_and(|text| text.contains(&pattern))
        });
        if !issued {
            failures.push(format!(
                "{}: none of {:?} issues {pattern}",
                s.variant, s.files
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The wiki's list of the player's decisions names every command of the player and every
/// exception, so the page and `PLAYER` cannot drift apart.
#[test]
fn the_wiki_lists_every_player_command() {
    let wiki =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/wiki/o-que-o-jogador-decide.md");
    let text = std::fs::read_to_string(&wiki).expect("reading the wiki page");
    let in_game = text
        .split("## No jogo")
        .nth(1)
        .and_then(|rest| rest.split("\n## ").next())
        .expect("the wiki page has a section \"No jogo\"");
    let missing: Vec<&str> = PLAYER
        .iter()
        .map(|s| s.variant)
        .chain(EXCEPTIONS.iter().map(|(v, _)| *v))
        .filter(|v| !in_game.contains(&format!("`{v}`")))
        .collect();
    assert!(
        missing.is_empty(),
        "docs/wiki/o-que-o-jogador-decide.md, section \"No jogo\", does not name {missing:?}"
    );
    assert!(
        !in_game.contains("em revisão"),
        "a decision in \"No jogo\" is still marked \"em revisão\""
    );
}
