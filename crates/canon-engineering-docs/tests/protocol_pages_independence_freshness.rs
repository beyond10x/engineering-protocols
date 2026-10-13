//! The committed pages and graphs of the built-in protocols show what their independence and
//! freshness rules demand: `software.change/1` and `incident.response/1` list their independence
//! dimensions, their independent security review and rollback verification name the dimension on
//! the page and on the graph edge, and every evidence kind with a maximum age shows its horizon.
//! `generate --check` keeps the files equal to the generator's output; this reads them as a site
//! reader would.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// The qualifier of the one edge from `from` to `to`.
fn qualifier(graph: &str, from: &str, to: &str) -> String {
    let graph: Value = serde_json::from_str(&read(graph)).expect("graph JSON");
    let edges: Vec<&Value> = graph["edges"]
        .as_array()
        .expect("edges")
        .iter()
        .filter(|edge| edge["from"] == from && edge["to"] == to)
        .collect();
    assert_eq!(edges.len(), 1, "one edge {from} -> {to}: {edges:?}");
    edges[0]["qualifier"]
        .as_str()
        .expect("a qualifier")
        .to_owned()
}

#[test]
fn software_change_shows_its_independent_security_review_and_observation_horizon() {
    let page = read("website/docs/protocols/software-change/1.mdx");
    assert!(page.contains("## Independence dimensions"), "{page}");
    assert!(
        page.contains("| `different_principal` | the producer is another principal than the one who did the case's work |"),
        "{page}"
    );
    assert!(page.contains("| `different_agent_run` |"), "{page}");
    assert!(
        page.contains(
            "evidence `security_review` with result `approved` about `implementation`, from a producer independent on `different_principal`"
        ),
        "{page}"
    );
    assert!(page.contains("| `operational_observation` |"), "{page}");
    assert!(page.contains("`30m` after it is observed"), "{page}");
    assert_eq!(
        qualifier(
            "website/data/protocol-graphs/software-change-1.json",
            "evidence:security_review",
            "claim:security.reviewed"
        ),
        "approved about implementation, independent on different_principal"
    );
}

#[test]
fn incident_response_shows_its_independent_rollback_verification_and_observation_horizon() {
    let page = read("website/docs/protocols/incident-response/1.mdx");
    assert!(page.contains("## Independence dimensions"), "{page}");
    assert!(page.contains("| `different_principal` |"), "{page}");
    assert!(
        page.contains(
            "evidence `rollback_result` with result `complete` about `release`, from a producer independent on `different_principal`"
        ),
        "{page}"
    );
    assert!(page.contains("`30m` after it is observed"), "{page}");
    assert_eq!(
        qualifier(
            "website/data/protocol-graphs/incident-response-1.json",
            "evidence:rollback_result",
            "claim:rollback.verified"
        ),
        "complete about release, independent on different_principal"
    );
}

#[test]
fn support_triage_shows_its_evidence_horizon() {
    let page = read("website/docs/protocols/support-triage/1.mdx");
    assert!(page.contains("`15m` after it is observed"), "{page}");
    assert!(!page.contains("## Independence dimensions"), "{page}");
}
