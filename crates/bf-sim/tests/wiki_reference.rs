//! The reference pages of the wiki (`docs/wiki/referencia/`), written from the data the game
//! loads: the goods, the buildings and their methods, the technologies and what they release,
//! and the rule switches. The test fails when a page differs from what the data give; with
//! `BF_BLESS=1 cargo test -p bf-sim --test wiki_reference` it rewrites the pages. Names come from
//! the pt-BR `.ftl` files; a key without a name shows as the key in backticks.
//!
//! What exists only with a switch on (a method, a good, a technology) is marked "só com a chave
//! X": the data are loaded once with the shipped switches and once with each switch that changes
//! the catalogue (`CATALOGUE_SWITCHES`), and a test checks that list against all of them on.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use bf_sim::data::GameData;
use bf_sim::game::Switches;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The switches read when the data load that add goods, methods or technologies.
const CATALOGUE_SWITCHES: &[&str] = &["radium", "helicopter", "computer_efficiency"];

/// What each switch does for the player, in Portuguese. A switch without a line fails the test.
const SWITCHES: &[(&str, &str)] = &[
    (
        "capital_follows_rate",
        "O capital privado vai para onde o juro rende mais, e o país pode fechar a conta de capital (controle de capital). Precisa da liquidação em reservas.",
    ),
    (
        "clause_tests",
        "Os acordos em vigor são testados todo mês: o tributo é pago e cobrado, e o acordo com prazo acaba quando o prazo vence.",
    ),
    (
        "computer_efficiency",
        "O computador rende mais por pessoa: laboratórios e universidades ganham métodos com computador, e, com a capacidade fiscal, cada servidor arrecada mais.",
    ),
    (
        "crisis",
        "Crises com prazo: um país exige, o outro aceita ou recusa pela força de cada lado, e a recusa pode virar guerra.",
    ),
    (
        "customs_duties",
        "O imposto de importação vai para o Tesouro e encarece o que vem de fora.",
    ),
    (
        "fiscal_capacity",
        "Um Estado com menos servidores do que precisa arrecada menos imposto.",
    ),
    (
        "forces",
        "Os soldados são os homens das formações de 1925 de cada país.",
    ),
    (
        "foreign_contract",
        "As fábricas de uma colônia continuam da antiga metrópole na independência, e o país que fecha o capital estrangeiro estatiza as de fora.",
    ),
    (
        "foreign_flow",
        "O dinheiro que sobra em casa constrói fábricas no exterior, e o lucro delas volta ao dono.",
    ),
    (
        "foreign_technique",
        "A técnica se espalha mais depressa onde há fábricas de donos estrangeiros.",
    ),
    (
        "gold_standard",
        "O padrão-ouro: quem está no ouro segura a paridade com os juros e as reservas, e sai quando a cobertura não dá.",
    ),
    (
        "helicopter",
        "A tecnologia do helicóptero (1936/1942), de que as unidades de helicóptero e o assalto aéreo precisam.",
    ),
    (
        "household_biomass",
        "As famílias do campo colhem a própria lenha, esterco e palha e compram só o resto do combustível.",
    ),
    (
        "independence",
        "Os movimentos de independência das colônias pedem resposta à metrópole, e a recusa pode virar revolta.",
    ),
    (
        "influence_ties",
        "O que um país vende a outro e o que tem investido nele pesam na influência que tem sobre ele.",
    ),
    (
        "jobs_follow_use",
        "A fábrica que vende menos do que pode fazer demite parte da equipe a cada mês.",
    ),
    (
        "ledger_record",
        "Cada linha do livro de dívidas entre governos guarda o que foi pago e perdoado, e a que acaba fica no registro.",
    ),
    (
        "market_stock",
        "O bem estocável que não acha comprador fica guardado no mercado e volta à venda depois.",
    ),
    (
        "media",
        "Mede o alcance da imprensa, do rádio, da televisão e da internet em cada país.",
    ),
    (
        "negotiation",
        "A mesa de negociação: o outro país responde às propostas, faz contrapropostas e pode denunciar acordos.",
    ),
    (
        "occupied_works",
        "As obras em terra ocupada ficam paradas até a terra voltar ao dono.",
    ),
    (
        "payments_settle",
        "O comércio e os pagamentos entre países mexem nas reservas de cada banco central.",
    ),
    (
        "radium",
        "As minas de urânio trabalham pelo rádio, que os hospitais compram, e o urânio sai como subproduto.",
    ),
    (
        "rail_by_line",
        "As ferrovias de 1925 têm o tamanho do que as linhas carregavam, e o resto do transporte vai por carroça, tropa e barco.",
    ),
    (
        "reputation",
        "Cada país guarda a reputação dos acordos que quebrou, que pesa na mesa de negociação.",
    ),
    (
        "reserve_cohorts",
        "A reserva militar por turma: quem termina o serviço entra na reserva, perde treino com o tempo e a mobilização chama os mais novos primeiro.",
    ),
    (
        "schooling",
        "A escola por turma e por nível, e a campanha de alfabetização de adultos.",
    ),
    (
        "small_market_wage_floor",
        "Um país pequeno demais para ter salário próprio usa o menor salário dos outros.",
    ),
    (
        "trade_by_pair",
        "Cada comprador escolhe de quem compra pelo custo de chegada.",
    ),
    (
        "trade_gross",
        "Cada país compra de fora uma parte do que também produz, não só o que falta.",
    ),
    (
        "treaties_in_force",
        "Os tratados de 1925 valem como acordos: chamam aliados para as crises e a quebra fica registrada.",
    ),
    (
        "union_services",
        "Em uma união aduaneira, serviços e finanças também se negociam entre os países.",
    ),
    (
        "war",
        "A guerra em terra: frentes, combate, avanço e controle do território.",
    ),
];

/// The pt-BR names of the `.ftl` files: `key = value` lines.
fn names(root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let dir = root.join("assets/i18n/pt-BR");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "ftl"))
        .collect();
    files.sort();
    for f in files {
        for line in std::fs::read_to_string(&f).unwrap().lines() {
            if line.starts_with(' ') || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once(" = ") {
                out.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }
    out
}

struct Ref {
    data: GameData,
    names: BTreeMap<String, String>,
    /// Goods, methods (`building.method`) and technologies that exist only with a switch on.
    gated: BTreeMap<String, &'static str>,
}

impl Ref {
    fn name(&self, prefix: &str, key: &str) -> String {
        self.names
            .get(&format!("{prefix}-{key}"))
            .cloned()
            .unwrap_or_else(|| format!("`{key}`"))
    }

    fn mark(&self, item: &str) -> String {
        self.gated
            .get(item)
            .map(|s| format!(" (só com a chave `{s}`)"))
            .unwrap_or_default()
    }
}

fn catalogue_items(d: &GameData) -> Vec<String> {
    let cat = &d.catalogue;
    let mut out: Vec<String> = cat
        .goods
        .iter()
        .map(|g| format!("good:{}", g.key))
        .collect();
    for b in &cat.buildings {
        for m in &b.methods {
            out.push(format!("method:{}.{}", b.key, m.key));
        }
    }
    if let Some(t) = &d.tree {
        out.extend(t.techs.iter().map(|t| format!("tech:{}", t.key)));
    }
    out
}

/// The data with every catalogue switch on (all items, in catalogue order), and which switch
/// each item beyond the shipped data needs.
fn load() -> Ref {
    let root = root();
    let base = GameData::load(&root).unwrap();
    let all = GameData::load_with(&root, &CATALOGUE_SWITCHES.join(",")).unwrap();
    let have: std::collections::BTreeSet<String> = catalogue_items(&base).into_iter().collect();
    let mut gated = BTreeMap::new();
    for sw in CATALOGUE_SWITCHES {
        let one = GameData::load_with(&root, sw).unwrap();
        for item in catalogue_items(&one) {
            if !have.contains(&item) {
                gated.insert(item.split_once(':').unwrap().1.to_string(), *sw);
            }
        }
    }
    for item in catalogue_items(&all) {
        let key = item.split_once(':').unwrap().1;
        assert!(
            have.contains(&item) || gated.contains_key(key),
            "{item} exists only with the switches on, but no switch of CATALOGUE_SWITCHES alone \
             brings it: add the switch to the list"
        );
    }
    Ref {
        data: all,
        names: names(&root),
        gated,
    }
}

fn fixed(x: laplace_schelling::Fixed) -> String {
    let v = x.to_f64_lossy();
    if (v - v.round()).abs() < 1e-9 {
        format!("{v:.0}")
    } else {
        let s = format!("{v:.3}");
        s.trim_end_matches('0')
            .trim_end_matches('.')
            .replace('.', ",")
    }
}

const HEADER: &str = "<!-- Página gerada de `data/` por `crates/bf-sim/tests/wiki_reference.rs`. Não edite à mão: \
regere com `BF_BLESS=1 cargo test -p bf-sim --test wiki_reference`. -->\n\n";

fn goods_page(r: &Ref) -> String {
    let mut s = String::from(HEADER);
    s.push_str("# Bens\n\nNa ordem do catálogo. Preço-base em dólares internacionais de 2011 por unidade.\n\n");
    s.push_str("| Bem | Chave | Categoria | Preço-base | Desde | Comércio | Estocável |\n|---|---|---|---|---|---|---|\n");
    for g in &r.data.catalogue.goods {
        let trade = if g.local {
            "local"
        } else if g.tradable {
            "entre países"
        } else {
            "não"
        };
        let stock = g
            .stock
            .map(|(w, f)| format!("até {w} semanas, piso {} do preço-base", fixed(f)))
            .unwrap_or_else(|| "—".into());
        writeln!(
            s,
            "| {}{} | `{}` | {} | {} | {} | {} | {} |",
            r.name("good", &g.key),
            r.mark(&g.key),
            g.key,
            r.name("good-category", &g.category),
            fixed(g.base_price),
            g.available_from,
            trade,
            stock
        )
        .unwrap();
    }
    s
}

fn buildings_page(r: &Ref) -> String {
    let cat = &r.data.catalogue;
    let tree = r.data.tree.as_deref();
    let mut s = String::from(HEADER);
    s.push_str("# Prédios e métodos\n\nNa ordem do catálogo. Insumos, produtos e empregos por nível e por semana, na produtividade 1.\n");
    let good = |g: u16| r.name("good", &cat.goods[usize::from(g)].key);
    for (bi, b) in cat.buildings.iter().enumerate() {
        let mut marks = Vec::new();
        if b.public {
            marks.push("público");
        }
        if b.state_firm {
            marks.push("empresa do Estado");
        }
        if b.informal {
            marks.push("informal");
        }
        if b.traditional {
            marks.push("tradicional: só o arranjo de 1925 com a chave `rail_by_line`");
        }
        writeln!(
            s,
            "\n## {} (`{}`)\n\n{}{}\n",
            r.name("building", &b.key),
            b.key,
            r.name("building-group", &b.group),
            if marks.is_empty() {
                String::new()
            } else {
                format!(" · {}", marks.join(" · "))
            }
        )
        .unwrap();
        s.push_str("| Método | Desde | Insumos | Produtos | Empregos |\n|---|---|---|---|---|\n");
        for (mi, m) in b.methods.iter().enumerate() {
            let since = tree
                .and_then(|t| t.method_tech.get(bi)?.get(mi).copied().flatten())
                .map(|t| {
                    let tk = &tree.unwrap().techs[usize::from(t)].key;
                    format!("com {}", r.name("tech", tk))
                })
                .unwrap_or_else(|| m.available_from.to_string());
            let list = |xs: &[(u16, laplace_schelling::Fixed)]| {
                if xs.is_empty() {
                    "—".to_string()
                } else {
                    xs.iter()
                        .map(|(g, q)| format!("{} {}", good(*g), fixed(*q)))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            };
            let jobs = m
                .jobs
                .iter()
                .map(|(p, n)| {
                    format!(
                        "{} {n}",
                        r.name("profession", bf_sim::economy::PROFESSIONS[usize::from(*p)])
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(
                s,
                "| {}{} | {} | {} | {} | {} |",
                r.name("method", &m.key),
                r.mark(&format!("{}.{}", b.key, m.key)),
                since,
                list(&m.inputs),
                list(&m.outputs),
                jobs
            )
            .unwrap();
        }
    }
    s
}

fn techs_page(r: &Ref) -> String {
    let cat = &r.data.catalogue;
    let Some(tree) = r.data.tree.as_deref() else {
        return String::from(HEADER);
    };
    let mut s = String::from(HEADER);
    s.push_str("# Tecnologias\n\nPor era, na ordem da árvore. Datas históricas: demonstrada é a primeira vez que funcionou em algum lugar, industrial a primeira produção em escala. Produtividade e efeitos no estágio difundido (metade no industrial).\n");
    for (era, start) in bf_sim::research::ERAS.iter().enumerate() {
        let techs: Vec<_> = tree
            .techs
            .iter()
            .filter(|t| usize::from(t.era) == era)
            .collect();
        if techs.is_empty() {
            continue;
        }
        if era == 0 {
            s.push_str("\n## Antes de 1925\n\n");
        } else {
            writeln!(s, "\n## Era {era} (desde {start})\n").unwrap();
        }
        s.push_str("| Tecnologia | Campo | Demonstrada / industrial | Requer | Libera | Produtividade | Efeitos |\n|---|---|---|---|---|---|---|\n");
        for t in techs {
            let requires = t
                .requires
                .iter()
                .map(|q| r.name("tech", &tree.techs[usize::from(*q)].key))
                .collect::<Vec<_>>()
                .join(", ");
            let mut frees = Vec::new();
            if !t.goods.is_empty() {
                frees.push(format!(
                    "bens: {}",
                    t.goods
                        .iter()
                        .map(|g| r.name("good", &cat.goods[usize::from(*g)].key))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            if !t.methods.is_empty() {
                frees.push(format!(
                    "métodos: {}",
                    t.methods
                        .iter()
                        .map(|(b, m)| {
                            let bd = &cat.buildings[usize::from(*b)];
                            r.name("method", &bd.methods[usize::from(*m)].key)
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            let productivity = t
                .productivity
                .iter()
                .map(|(g, v)| format!("{} +{}", r.name("building-group", g), pct(*v)))
                .collect::<Vec<_>>()
                .join(", ");
            let effects = t
                .effects
                .iter()
                .map(|(e, v)| {
                    let key = bf_sim::research::EFFECTS[usize::from(*e)];
                    r.names.get(&format!("research-effect-{key}")).map_or_else(
                        || format!("`{key}` {}", pct(*v)),
                        |x| x.replace("{ $pct }", &pct(*v)),
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let dash = |x: String| if x.is_empty() { "—".to_string() } else { x };
            writeln!(
                s,
                "| {}{}{} | {} | {} / {} | {} | {} | {} | {} |",
                r.name("tech", &t.key),
                r.mark(&t.key),
                if t.programme_only {
                    " (só com programa nuclear)"
                } else {
                    ""
                },
                r.name("tech-field", bf_sim::research::FIELDS[usize::from(t.field)]),
                t.demonstrated,
                t.industrialized,
                dash(requires),
                dash(frees.join("; ")),
                dash(productivity),
                dash(effects)
            )
            .unwrap();
        }
    }
    s
}

fn pct(per_mille: u16) -> String {
    let v = f64::from(per_mille) / 10.0;
    if (v - v.round()).abs() < 1e-9 {
        format!("{v:.0} %")
    } else {
        format!("{v:.1} %").replace('.', ",")
    }
}

fn switches_page(root: &Path) -> String {
    let mut s = String::from(HEADER);
    s.push_str("# Chaves de regra\n\nCada regra nova entra desligada, com o jogo igual ao de antes; ligar é outra decisão, medida contra a faixa (`docs/wiki/principios.md`). O estado é o de `data/rules.ron`.\n\n");
    s.push_str("| Chave | Estado | O que faz |\n|---|---|---|\n");
    let text = std::fs::read_to_string(root.join("data/rules.ron")).unwrap();
    for name in Switches::NAMES {
        let line = SWITCHES
            .iter()
            .find(|(k, _)| k == name)
            .map(|x| x.1)
            .unwrap_or_else(|| panic!("the switch {name} has no sentence in SWITCHES"));
        let on = text
            .lines()
            .find(|l| l.trim_start().starts_with(&format!("{name}:")))
            .is_some_and(|l| l.contains("true"));
        writeln!(
            s,
            "| `{name}` | {} | {line} |",
            if on { "ligada" } else { "desligada" }
        )
        .unwrap();
    }
    s
}

fn readme() -> String {
    let mut s = String::from(HEADER);
    s.push_str("# Referência dos dados\n\nPáginas geradas dos arquivos de `data/` e dos nomes de `assets/i18n/pt-BR/`; o teste `wiki_reference` falha no CI quando estão desatualizadas.\n\n");
    s.push_str("- [Bens](bens.md)\n- [Prédios e métodos](predios.md)\n- [Tecnologias](tecnologias.md)\n- [Chaves de regra](chaves.md)\n");
    s
}

#[test]
fn the_reference_pages_match_the_data() {
    let root = root();
    let r = load();
    let pages = [
        ("README.md", readme()),
        ("bens.md", goods_page(&r)),
        ("predios.md", buildings_page(&r)),
        ("tecnologias.md", techs_page(&r)),
        ("chaves.md", switches_page(&root)),
    ];
    let dir = root.join("docs/wiki/referencia");
    let bless = std::env::var_os("BF_BLESS").is_some();
    if bless {
        std::fs::create_dir_all(&dir).unwrap();
    }
    for (file, page) in pages {
        let path = dir.join(file);
        if bless {
            std::fs::write(&path, &page).unwrap();
            continue;
        }
        let old = std::fs::read_to_string(&path).unwrap_or_default();
        if old != page {
            let line = old
                .lines()
                .zip(page.lines())
                .position(|(a, b)| a != b)
                .unwrap_or(old.lines().count().min(page.lines().count()));
            panic!(
                "docs/wiki/referencia/{file} is out of date with the data (first difference at line {}): \
                 run BF_BLESS=1 cargo test -p bf-sim --test wiki_reference",
                line + 1
            );
        }
    }
}

#[test]
fn every_switch_has_a_sentence() {
    for name in Switches::NAMES {
        assert!(
            SWITCHES.iter().any(|(k, _)| k == name),
            "the switch {name} has no sentence for the wiki (SWITCHES in tests/wiki_reference.rs)"
        );
    }
}
