//! The status record: one `b10x-status/1` document that the landing page's status section and the
//! status page's `<StatusTable>` both read. Protocol rows and the vocabulary count come from the
//! repository; the remaining capabilities are listed here, beside the code that ships them.

use b10x_canon::ir::Ir;
use b10x_canon::model::{OutcomeRequirement, Predicate};
use canon_engineering::vocabulary::Vocabulary;
use serde_json::{Value, json};

use crate::markdown::HEADER;
use crate::protocol::Source;

/// The status document, relative to the site.
pub const STATUS_FILE: &str = "data/status.json";

/// The status page, relative to the site.
pub const STATUS_PAGE: &str = "docs/status.mdx";

const FOUNDATIONS: &str = "Foundations";
const PROTOCOLS: &str = "Protocols";
const DOCUMENTATION: &str = "Documentation";

/// One row of the status document.
struct Item {
    label: String,
    status: &'static str,
    area: &'static str,
    detail: String,
    href: Option<String>,
}

impl Item {
    fn json(&self) -> Value {
        let mut row = json!({
            "label": self.label,
            "status": self.status,
            "area": self.area,
            "detail": self.detail,
        });
        if let Some(href) = &self.href {
            row["href"] = json!(href);
        }
        row
    }
}

fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

/// Whether the protocol has at least one evidence match and every match, in every claim,
/// obligation, action precondition and outcome, names the artifact it is about.
fn every_match_bound(ir: &Ir) -> bool {
    let mut matches = 0usize;
    let mut unbound = 0usize;
    let mut count = |predicate: &Predicate| {
        predicate.visit(&mut |node| {
            if let Predicate::Evidence(matching) = node {
                matches += 1;
                if matching.subject.is_none() {
                    unbound += 1;
                }
            }
        });
    };
    ir.claims.values().for_each(|claim| count(&claim.true_when));
    ir.obligations
        .values()
        .for_each(|obligation| count(&obligation.discharged_when));
    ir.actions
        .values()
        .for_each(|action| count(&action.precondition));
    for outcome in ir.outcomes.values() {
        if let OutcomeRequirement::Predicate(requires) = &outcome.requires {
            count(requires);
        }
    }
    matches > 0 && unbound == 0
}

/// The non-zero counts as "a, b and c"; "nothing" when every count is zero.
fn listing(counts: &[(usize, &str, &str)]) -> String {
    let parts: Vec<String> = counts
        .iter()
        .filter(|(count, _, _)| *count > 0)
        .map(|(count, one, many)| plural(*count, one, many))
        .collect();
    match parts.as_slice() {
        [] => "nothing".to_owned(),
        [only] => only.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// The `b10x-status/1` document for the given vocabulary and compiled protocols.
pub fn document(vocabulary: &Vocabulary, protocols: &[(&Source, &Ir)]) -> Value {
    let mut items = vec![
        Item {
            label: "Typed assertion gates and replay".to_owned(),
            status: "shipped",
            area: FOUNDATIONS,
            // Held by assertion_replay::retained_evidence_replays_without_reading_sources_and_expires
            // and extensions::registered_rust_provider_extends_namespace_and_replay_does_not_execute_it.
            detail: "Typed file, process, report and network checks retain source-bound evidence for offline replay. Recipes and explicitly admitted Rust providers extend the catalog. The two assertion crates are frozen at tag 0.16.0 and take no new assertion function or provider.".to_owned(),
            href: Some("/docs/guides/assertions".to_owned()),
        },
        Item {
            label: "Engineering vocabulary".to_owned(),
            status: "shipped",
            area: FOUNDATIONS,
            detail: format!(
                "{}, each with its category, marking and meaning, in protocols/vocabulary.yaml.",
                plural(vocabulary.terms().len(), "term", "terms")
            ),
            href: Some("/docs/vocabulary".to_owned()),
        },
        Item {
            label: "Fixture harness over Canon evaluation".to_owned(),
            status: "shipped",
            area: FOUNDATIONS,
            detail: "Compiles a protocol with Canon and checks each fixture state's claims against Canon's evaluator.".to_owned(),
            href: None,
        },
        Item {
            label: "Built-in protocol registry".to_owned(),
            status: "shipped",
            area: FOUNDATIONS,
            detail: "The b10x-canon-engineering crate embeds every protocol under protocols/; canon-engineering protocols list and canon-engineering protocols show <name>@<major> print them.".to_owned(),
            href: None,
        },
    ];
    for (source, ir) in protocols {
        items.push(Item {
            label: format!("{}/{}", ir.protocol.id, source.major),
            status: "shipped",
            area: PROTOCOLS,
            detail: format!(
                "Canon protocol/1 data declaring {}{}.",
                listing(&[
                    (ir.claims.len(), "claim", "claims"),
                    (ir.evidence_kinds.len(), "evidence kind", "evidence kinds"),
                    (ir.actions.len(), "action", "actions"),
                    (ir.obligations.len(), "obligation", "obligations"),
                    (ir.outcomes.len(), "outcome", "outcomes"),
                ]),
                if every_match_bound(ir) {
                    ", with every evidence match bound to its artifact"
                } else {
                    ""
                }
            ),
            href: Some(format!("/docs{}", source.slug())),
        });
    }
    items.push(Item {
        label: "Protocol pages from canon-ir/1".to_owned(),
        status: "shipped",
        area: DOCUMENTATION,
        detail: "Generated by canon-engineering-docs for every protocol in protocols/, with its graph, and checked for drift.".to_owned(),
        href: Some("/docs/protocols".to_owned()),
    });
    json!({
        "format": "b10x-status/1",
        "source": "Generated by canon-engineering-docs from protocols/ and the engineering protocols crates",
        "items": items.iter().map(Item::json).collect::<Vec<_>>(),
    })
}

/// The status page: the same document, rendered as a table.
pub fn page() -> String {
    format!(
        "---\n{HEADER}\ntitle: \"Status\"\nsidebar_position: 5\ndescription: \"Every engineering protocols capability with its status, from the same file the landing page reads.\"\nstatus: shipped\nlede: \"Shipped means it is in the repository and tested today. The landing page's status strip reads the same file, so the two cannot disagree.\"\nsource: \"website/{STATUS_FILE}, generated by canon-engineering-docs\"\ncustom_edit_url: null\n---\n\nimport status from '@site/{STATUS_FILE}';\n\n<StatusTable data={{status}} />\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plural_names_one_and_many() {
        assert_eq!(plural(1, "claim", "claims"), "1 claim");
        assert_eq!(plural(0, "claim", "claims"), "0 claims");
        assert_eq!(plural(3, "claim", "claims"), "3 claims");
    }

    #[test]
    fn listing_skips_zero_counts() {
        let counts = |a, b, c| {
            [
                (a, "claim", "claims"),
                (b, "action", "actions"),
                (c, "outcome", "outcomes"),
            ]
        };
        assert_eq!(listing(&counts(3, 0, 1)), "3 claims and 1 outcome");
        assert_eq!(
            listing(&counts(1, 2, 3)),
            "1 claim, 2 actions and 3 outcomes"
        );
        assert_eq!(listing(&counts(0, 2, 0)), "2 actions");
        assert_eq!(listing(&counts(0, 0, 0)), "nothing");
    }

    #[test]
    fn the_page_imports_the_status_file() {
        let page = page();
        assert!(page.starts_with(&format!("---\n{HEADER}\n")));
        assert!(page.contains("import status from '@site/data/status.json';\n"));
        assert!(page.contains("<StatusTable data={status} />\n"));
    }
}
