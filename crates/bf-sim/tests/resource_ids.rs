//! Every resource the scheduler knows has an id of its own. Two systems that write resources with
//! the same id would be ordered as if they touched the same data (and two that do touch the same
//! data could run together if a third id hid it), so a clash is a scheduling bug that no hash
//! shows. The test reads the sources, so a resource added later is checked without a list to keep.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// `const NAME: <path::>ResourceId = <path::>ResourceId(N);`, over any line breaks and paths.
fn declarations(text: &str) -> Vec<(String, u64)> {
    let flat: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = Vec::new();
    let mut rest = flat.as_str();
    while let Some(at) = rest.find("const ") {
        rest = &rest[at + "const ".len()..];
        let Some(end) = rest.find(';') else { break };
        let decl = &rest[..end];
        let Some((name, value)) = decl.split_once(':') else {
            continue;
        };
        let Some((ty, init)) = value.split_once('=') else {
            continue;
        };
        if !ty.trim().ends_with("ResourceId") {
            continue;
        }
        let init = init.trim();
        let Some(open) = init.find("ResourceId(") else {
            continue;
        };
        let n = &init[open + "ResourceId(".len()..];
        let n: u64 = n[..n.find(')').unwrap()].trim().parse().unwrap();
        out.push((name.trim().to_owned(), n));
    }
    out
}

#[test]
fn the_parser_reads_a_declaration_over_lines() {
    let text = "pub const A: laplace_schelling::schedule::ResourceId =\n    laplace_schelling::schedule::ResourceId(15);\nconst B: ResourceId = ResourceId(3);";
    assert_eq!(declarations(text), vec![("A".into(), 15), ("B".into(), 3)]);
}

#[test]
fn no_two_resources_share_an_id() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&src, &mut files);
    let mut seen: BTreeMap<u64, Vec<String>> = BTreeMap::new();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap();
        let file = f.strip_prefix(&src).unwrap().display().to_string();
        for (name, id) in declarations(&text) {
            seen.entry(id).or_default().push(format!("{name} ({file})"));
        }
    }
    assert!(seen.len() >= 8, "the resources were not found: {seen:?}");
    let clashes: Vec<_> = seen.iter().filter(|(_, v)| v.len() > 1).collect();
    assert!(clashes.is_empty(), "resources sharing an id: {clashes:?}");
}
