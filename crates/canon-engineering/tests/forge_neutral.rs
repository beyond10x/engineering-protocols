//! Acceptance for `story:forge-neutral-delivery`: the built-in protocols and the vocabulary name
//! no organisation's delivery route. A public protocol is read by any coding agent, so a pull
//! request opens through the run's forge, never through one organisation's bot. Since D257 this
//! holds for `story.delivery/1` as well, with takt's agreement (`takt_protocols_builtin.rs`).

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN: &[&str] = &["bot route", "b10x-gates", "b10x-bot"];

fn protocol_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read protocols directory") {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            protocol_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "yaml") {
            out.push(path);
        }
    }
}

fn names_a_bot(text: &str) -> bool {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .any(|word| word.eq_ignore_ascii_case("bot") || word.eq_ignore_ascii_case("bots"))
}

fn protocols_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../protocols")
}

#[test]
fn built_in_protocols_and_vocabulary_name_no_bot_route() {
    let root = protocols_root();
    let mut files = Vec::new();
    protocol_files(&root, &mut files);
    for expected in [
        "vocabulary.yaml",
        "story-delivery/1.yaml",
        "story-delivery/2.yaml",
    ] {
        assert!(
            files.iter().any(|path| path.ends_with(expected)),
            "protocols/{expected} was not found under {}",
            root.display()
        );
    }
    let mut findings = Vec::new();
    for path in &files {
        let text = fs::read_to_string(path).expect("read protocol file");
        for (number, line) in text.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            if FORBIDDEN.iter().any(|term| lower.contains(term)) || names_a_bot(line) {
                let shown = path.strip_prefix(&root).unwrap_or(path);
                findings.push(format!(
                    "protocols/{}:{}: {line}",
                    shown.display(),
                    number + 1
                ));
            }
        }
    }
    assert!(
        findings.is_empty(),
        "a built-in protocol names an organisation's bot; write \"the run's forge\":\n{}",
        findings.join("\n")
    );
}

#[test]
fn pr_open_opens_through_the_runs_forge() {
    let root = protocols_root();
    for file in ["story-delivery/1.yaml", "story-delivery/2.yaml"] {
        let delivery = fs::read_to_string(root.join(file)).expect("read story.delivery");
        let folded = delivery.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            folded.contains("open its pull request through the run's forge"),
            "{file}: pr.open does not open through the run's forge"
        );
    }
    let vocabulary = fs::read_to_string(root.join("vocabulary.yaml")).expect("read vocabulary");
    assert!(
        vocabulary.contains("open its pull request through the run's forge"),
        "vocabulary.yaml: pr.open does not open through the run's forge"
    );
}
