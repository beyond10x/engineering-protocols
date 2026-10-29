//! One protocol page, rendered from Canon's compiled form (`canon-ir/1`), never from the YAML text,
//! so the page shows what Canon understood (this repository's `story:protocol-docs-render`).

use std::fmt::Write as _;

use b10x_canon::ir::Ir;
use b10x_canon::model::{InputValue, OutcomeRequirement, Predicate, Truth};

use crate::markdown::{HEADER, code, text, yaml_string};

/// A protocol document found under `protocols/<name>/<major>.yaml`.
pub struct Source {
    pub name: String,
    pub major: u64,
    pub path: String,
    pub text: String,
}

impl Source {
    /// The generated page, relative to `website/docs/`.
    pub fn file(&self) -> String {
        format!("protocols/{}/{}.mdx", self.name, self.major)
    }

    /// The generated graph document, relative to `website/`.
    pub fn graph_file(&self) -> String {
        format!("data/protocol-graphs/{}-{}.json", self.name, self.major)
    }

    /// The page's route below the site base.
    pub fn slug(&self) -> String {
        format!("/protocols/{}/{}", self.name, self.major)
    }
}

fn truth(value: Truth) -> &'static str {
    match value {
        Truth::True => "**TRUE**",
        Truth::False => "**FALSE**",
        Truth::Unknown => "**UNKNOWN**",
    }
}

/// Case input values as a code-formatted list, in the order given.
fn values(values: &[InputValue]) -> String {
    values
        .iter()
        .map(|value| code(value.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A predicate as one readable line, nested with parentheses.
pub fn predicate(predicate: &Predicate) -> String {
    let list = |members: &[Predicate]| -> String {
        members
            .iter()
            .map(self::predicate)
            .collect::<Vec<_>>()
            .join("; ")
    };
    match predicate {
        Predicate::Evidence(matching) => {
            let mut line = format!("evidence {}", code(matching.kind.as_str()));
            if let Some(result) = &matching.result {
                line.push_str(&format!(" with result {}", code(result)));
            }
            if let Some(subject) = &matching.subject {
                line.push_str(&format!(" about {}", code(subject.as_str())));
            }
            if let Some(dimensions) = matching.independent_on.as_deref().filter(|d| !d.is_empty()) {
                let dimensions: Vec<String> = dimensions.iter().map(|d| code(d.as_str())).collect();
                line.push_str(&format!(
                    ", from a producer independent on {}",
                    dimensions.join(" and ")
                ));
            }
            line
        }
        Predicate::Claim(test) => format!("{} is {}", code(test.claim.as_str()), truth(test.is)),
        Predicate::Input(test) => format!(
            "input {} is one of {}",
            code(test.input.as_str()),
            values(&test.r#in)
        ),
        Predicate::First(form) => {
            let mut line = format!("the first {}", code(form.kind.as_str()));
            if let Some(subject) = &form.subject {
                line.push_str(&format!(" about {}", code(subject.as_str())));
            }
            line.push_str(&format!(" is {}", code(&form.result)));
            line
        }
        Predicate::Before(form) => {
            let mut line = format!(
                "the first {} before the first {}",
                code(form.first.as_str()),
                code(form.then.as_str())
            );
            if let Some(subject) = &form.subject {
                line.push_str(&format!(", both about {}", code(subject.as_str())));
            }
            line
        }
        Predicate::After(form) => {
            let mut line = format!(
                "a {} with result {} after the first {}",
                code(form.then.kind.as_str()),
                code(&form.then.result),
                code(form.first.as_str())
            );
            if let Some(subject) = &form.subject {
                line.push_str(&format!(", both about {}", code(subject.as_str())));
            }
            line
        }
        Predicate::Since(form) => {
            let mut line = format!(
                "a {} with result {} since the latest {}, at its revision",
                code(form.then.kind.as_str()),
                code(&form.then.result),
                code(form.last.as_str())
            );
            if let Some(subject) = &form.subject {
                line.push_str(&format!(", both about {}", code(subject.as_str())));
            }
            line
        }
        Predicate::All(members) if members.is_empty() => "always true".to_owned(),
        Predicate::Any(members) if members.is_empty() => "never true".to_owned(),
        Predicate::All(members) => format!("all of ({})", list(members)),
        Predicate::Any(members) => format!("any of ({})", list(members)),
        Predicate::Not(inner) => format!("not ({})", self::predicate(inner)),
    }
}

/// What an outcome requires as one readable line: its predicate, or the explicit decision it needs.
fn requirement(requirement: &OutcomeRequirement) -> String {
    match requirement {
        OutcomeRequirement::Predicate(requires) => predicate(requires),
        OutcomeRequirement::Decision(name) => format!("decision {}", code(name.as_str())),
    }
}

fn description(value: &Option<String>) -> String {
    value
        .as_deref()
        .map(|value| text(value.trim()))
        .unwrap_or_default()
}

fn joined(items: Vec<String>, empty: &str) -> String {
    if items.is_empty() {
        format!("*{empty}*")
    } else {
        items.join(", ")
    }
}

fn table(out: &mut String, title: &str, lede: &str, head: &[&str], rows: Vec<Vec<String>>) {
    let _ = write!(out, "\n## {title}\n\n{lede}\n\n");
    if rows.is_empty() {
        let _ = writeln!(out, "*This protocol declares no {}.*", title.to_lowercase());
        return;
    }
    let _ = writeln!(out, "| {} |", head.join(" | "));
    let _ = writeln!(out, "|{}", "---|".repeat(head.len()));
    for row in rows {
        let _ = writeln!(out, "| {} |", row.join(" | "));
    }
}

/// The page for one compiled protocol.
pub fn page(ir: &Ir, source: &Source) -> String {
    let header = &ir.protocol;
    let producers = |kind: &str| -> Vec<String> {
        ir.actions
            .iter()
            .filter(|(_, action)| action.may_produce.iter().any(|k| k.as_str() == kind))
            .map(|(id, _)| code(id.as_str()))
            .collect()
    };
    let readers = |kind: &str| -> Vec<String> {
        ir.claims
            .iter()
            .filter(|(_, claim)| {
                let mut found = false;
                claim.true_when.visit(&mut |inner| {
                    if let Predicate::Evidence(matching) = inner {
                        found |= matching.kind.as_str() == kind;
                    }
                });
                found
            })
            .map(|(id, _)| code(id.as_str()))
            .collect()
    };
    let title = format!("{}/{}", header.id, source.major);

    let mut out = String::new();
    let summary = header
        .description
        .as_deref()
        .map(str::trim)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("The engineering protocol {title}."));
    let _ = writeln!(out, "---\n{HEADER}");
    let _ = writeln!(out, "id: {}", yaml_string(&source.major.to_string()));
    let _ = writeln!(out, "title: {}", yaml_string(&title));
    let _ = writeln!(out, "sidebar_label: {}", yaml_string(&title));
    let _ = writeln!(out, "description: {}", yaml_string(&summary));
    let _ = writeln!(out, "slug: {}", source.slug());
    out.push_str("hide_table_of_contents: true\ncustom_edit_url: null\n---\n\n");
    let _ = writeln!(out, "import graph from '@site/{}';\n", source.graph_file());

    if let Some(description) = &header.description {
        let _ = writeln!(out, "{}\n", text(description.trim()));
    }
    let _ = writeln!(out, "- **Id:** {}", code(header.id.as_str()));
    let _ = writeln!(out, "- **Revision:** {}", header.revision);
    let _ = writeln!(
        out,
        "- **Source:** [{}](https://github.com/beyond10x/engineering-protocols/blob/main/{})",
        code(&source.path),
        source.path
    );

    let _ = write!(
        out,
        "\n## Dependency graph\n\nWhich actions may produce which evidence, which claims that evidence establishes, and which outcomes and obligations rest on those claims. Hover or focus a node to trace what it rests on.\n\n<ProtocolGraph data={{graph}} />\n"
    );

    table(
        &mut out,
        "Actions",
        "What can be done in a case: what it needs first, which authority it requires, the class of effect it has and the evidence it may produce.",
        &[
            "Action",
            "Description",
            "Precondition",
            "Requires capability",
            "Effect",
            "May produce",
        ],
        ir.actions
            .iter()
            .map(|(id, action)| {
                let precondition = match &action.precondition {
                    Predicate::All(members) if members.is_empty() => "*none*".to_owned(),
                    other => predicate(other),
                };
                vec![
                    code(id.as_str()),
                    description(&action.description),
                    precondition,
                    joined(
                        action.requires.iter().map(|c| code(c.as_str())).collect(),
                        "no authority",
                    ),
                    action
                        .effect
                        .as_ref()
                        .map(|effect| code(effect.as_str()))
                        .unwrap_or_else(|| "*not declared*".to_owned()),
                    joined(
                        action
                            .may_produce
                            .iter()
                            .map(|k| code(k.as_str()))
                            .collect(),
                        "no evidence",
                    ),
                ]
            })
            .collect(),
    );

    // The "Applies for" column appears only when some evidence kind declares a maximum age, so the
    // page of a protocol without `max_age` is the page it was before.
    let horizons = ir
        .evidence_kinds
        .values()
        .any(|kind| kind.max_age.is_some());
    let kind_head: &[&str] = if horizons {
        &[
            "Evidence kind",
            "Description",
            "Applies for",
            "Produced by",
            "Used by claims",
        ]
    } else {
        &[
            "Evidence kind",
            "Description",
            "Produced by",
            "Used by claims",
        ]
    };
    let kind_lede = if horizons {
        "The kinds of evidence this protocol admits. A kind with a maximum age applies for that long after a record's `observed_at`: evaluated at a later instant, the record is excluded as expired."
    } else {
        "The kinds of evidence this protocol admits."
    };
    table(
        &mut out,
        "Evidence kinds",
        kind_lede,
        kind_head,
        ir.evidence_kinds
            .iter()
            .map(|(id, kind)| {
                let mut row = vec![code(id.as_str()), description(&kind.description)];
                if horizons {
                    row.push(
                        kind.max_age
                            .as_ref()
                            .map(|age| format!("{} after it is observed", code(age.as_str())))
                            .unwrap_or_else(|| "*no limit*".to_owned()),
                    );
                }
                row.push(joined(producers(id.as_str()), "no action"));
                row.push(joined(readers(id.as_str()), "no claim"));
                row
            })
            .collect(),
    );

    table(
        &mut out,
        "Claims",
        "Each claim is TRUE, FALSE or UNKNOWN, established from evidence by its predicate. Missing or stale evidence leaves a claim UNKNOWN, never FALSE.",
        &["Claim", "Description", "True when"],
        ir.claims
            .iter()
            .map(|(id, claim)| {
                vec![
                    code(id.as_str()),
                    description(&claim.description),
                    predicate(&claim.true_when),
                ]
            })
            .collect(),
    );

    // The "Applies when" column appears only when some obligation is conditional, so the page of a
    // protocol without `applicable_when` is the page it was before Canon 0.2.0.
    let conditional = ir
        .obligations
        .values()
        .any(|obligation| obligation.applicable_when.is_some());
    let obligation_head: &[&str] = if conditional {
        &[
            "Obligation",
            "Description",
            "Applies when",
            "Discharged when",
        ]
    } else {
        &["Obligation", "Description", "Discharged when"]
    };
    table(
        &mut out,
        "Obligations",
        "What must be done before a case can be complete, and what discharges it. An obligation stays open until its predicate is TRUE; its predicate being UNKNOWN does not discharge it.",
        obligation_head,
        ir.obligations
            .iter()
            .map(|(id, obligation)| {
                let mut row = vec![code(id.as_str()), description(&obligation.description)];
                if conditional {
                    row.push(
                        obligation
                            .applicable_when
                            .as_ref()
                            .map(predicate)
                            .unwrap_or_else(|| "*always*".to_owned()),
                    );
                }
                row.push(predicate(&obligation.discharged_when));
                row
            })
            .collect(),
    );

    table(
        &mut out,
        "Outcomes",
        "The legitimate terminal interpretations of a case, and what each requires.",
        &["Outcome", "Description", "Requires"],
        ir.outcomes
            .iter()
            .map(|(id, outcome)| {
                vec![
                    code(id.as_str()),
                    description(&outcome.description),
                    requirement(&outcome.requires),
                ]
            })
            .collect(),
    );

    table(
        &mut out,
        "Artifacts",
        "The things whose revisions evidence is bound to.",
        &["Artifact", "Description"],
        ir.artifacts
            .iter()
            .map(|(id, artifact)| vec![code(id.as_str()), description(&artifact.description)])
            .collect(),
    );

    // Written only when the protocol declares a case input, so a protocol without a `case`
    // section keeps the page it had before Canon 0.2.0.
    if !ir.inputs.is_empty() {
        table(
            &mut out,
            "Case inputs",
            "What a case fixes when it opens: each input takes one of the values listed, and is never UNKNOWN.",
            &["Input", "Description", "Values"],
            ir.inputs
                .iter()
                .map(|(id, input)| {
                    vec![
                        code(id.as_str()),
                        description(&input.description),
                        values(&input.values),
                    ]
                })
                .collect(),
        );
    }

    // Written only when the protocol declares an independence dimension, so a protocol without an
    // `independence_dimensions` section keeps the page it had before Canon 0.4.0.
    if !ir.independence_dimensions.is_empty() {
        table(
            &mut out,
            "Independence dimensions",
            "What an evidence match may demand of a record's producer (`independent_on`). The case lists, for each producer, the dimensions on which it is independent of the case's work; a match that demands a dimension reads only records whose producer the case lists on it. A record that fails the demand neither establishes nor contradicts the match, so a match only such records would decide is UNKNOWN.",
            &["Dimension", "Description"],
            ir.independence_dimensions
                .iter()
                .map(|(id, dimension)| vec![code(id.as_str()), description(&dimension.description)])
                .collect(),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use b10x_canon::model::parse;

    fn claim_predicate(yaml: &str) -> String {
        let source = format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{x: {{}}}}\nevidence_kinds: {{e: {{}}}}\nclaims:\n  a:\n    true_when: {{evidence: {{kind: e}}}}\n  b:\n    true_when: {yaml}\n"
        );
        let protocol = parse(&source).expect("parses");
        let (_, claim) = protocol.claims.iter().nth(1).expect("claim b");
        predicate(&claim.true_when)
    }

    #[test]
    fn predicates_read_as_one_nested_line() {
        assert_eq!(
            claim_predicate(
                "{any: [{not: {claim: a, is: unknown}}, {all: []}, {evidence: {kind: e, result: ok}}]}"
            ),
            "any of (not (`a` is **UNKNOWN**); always true; evidence `e` with result `ok`)"
        );
        assert_eq!(
            claim_predicate(
                "{all: [{evidence: {kind: e, result: ok, subject: x}}, {evidence: {kind: e, subject: x}}]}"
            ),
            "all of (evidence `e` with result `ok` about `x`; evidence `e` about `x`)"
        );
        assert_eq!(claim_predicate("{claim: a}"), "`a` is **TRUE**");
        assert_eq!(claim_predicate("{any: []}"), "never true");
    }

    #[test]
    fn order_forms_read_in_words() {
        assert_eq!(
            claim_predicate("{first: {kind: e, result: fail, subject: x}}"),
            "the first `e` about `x` is `fail`"
        );
        assert_eq!(
            claim_predicate("{first: {kind: e, result: fail}}"),
            "the first `e` is `fail`"
        );
        let two_kinds = |yaml: &str| {
            let source = format!(
                "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{x: {{}}}}\nevidence_kinds: {{e: {{}}, f: {{}}}}\nclaims:\n  b:\n    true_when: {yaml}\n"
            );
            let protocol = parse(&source).expect("parses");
            let (_, claim) = protocol.claims.iter().next().expect("claim b");
            predicate(&claim.true_when)
        };
        assert_eq!(
            two_kinds("{before: {first: e, then: f, subject: x}}"),
            "the first `e` before the first `f`, both about `x`"
        );
        assert_eq!(
            two_kinds(
                "{any: [{evidence: {kind: e, result: fail}}, {before: {first: e, then: f}}]}"
            ),
            "any of (evidence `e` with result `fail`; the first `e` before the first `f`)"
        );
    }

    #[test]
    fn the_obligations_section_says_what_discharges_each_obligation() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {e: {}}\nclaims:\n  a:\n    true_when: {evidence: {kind: e}}\nobligations:\n  settle:\n    description: settle it\n    discharged_when: {claim: a}\n";
        let ir = b10x_canon::ir::compile(&parse(source).expect("parses")).expect("compiles");
        let page = page(
            &ir,
            &Source {
                name: "p".to_owned(),
                major: 1,
                path: "protocols/p/1.yaml".to_owned(),
                text: source.to_owned(),
            },
        );
        let start = page.find("## Obligations").expect("an Obligations section");
        let section = &page[start..];
        let section = &section[..section[2..]
            .find("\n## ")
            .map_or(section.len(), |end| end + 2)];
        assert!(
            section.contains("| Obligation | Description | Discharged when |"),
            "{section}"
        );
        assert!(
            section.contains("| `settle` | settle it | `a` is **TRUE** |"),
            "{section}"
        );
    }

    /// The page section that starts at `heading`, up to the next one.
    fn section<'a>(page: &'a str, heading: &str) -> &'a str {
        let start = page
            .find(heading)
            .unwrap_or_else(|| panic!("a {heading} section in\n{page}"));
        let section = &page[start..];
        &section[..section[2..]
            .find("\n## ")
            .map_or(section.len(), |end| end + 2)]
    }

    #[test]
    fn the_page_lists_case_inputs_and_when_an_obligation_applies() {
        let source = include_str!("../tests/fixtures/case-inputs.yaml");
        let ir = b10x_canon::ir::compile(&parse(source).expect("parses")).expect("compiles");
        let page = page(
            &ir,
            &Source {
                name: "p".to_owned(),
                major: 1,
                path: "protocols/p/1.yaml".to_owned(),
                text: source.to_owned(),
            },
        );
        let inputs = section(&page, "## Case inputs");
        assert!(
            inputs.contains("| Input | Description | Values |"),
            "{inputs}"
        );
        assert!(inputs.contains("| `risk` |  | `high`, `low` |"), "{inputs}");
        let obligations = section(&page, "## Obligations");
        assert!(
            obligations.contains("| Obligation | Description | Applies when | Discharged when |"),
            "{obligations}"
        );
        assert!(
            obligations
                .contains("| `review` |  | input `risk` is one of `high` | `a` is **TRUE** |"),
            "{obligations}"
        );
    }

    /// A protocol's independence dimensions are listed with their descriptions, every evidence
    /// match that demands independence says so wherever the page writes it (claim, action and
    /// outcome text), and an evidence kind with a maximum age shows its horizon.
    #[test]
    fn the_page_shows_independence_demands_and_evidence_horizons() {
        let source = include_str!("../tests/fixtures/independence-freshness.yaml");
        let ir = b10x_canon::ir::compile(&parse(source).expect("parses")).expect("compiles");
        let page = page(
            &ir,
            &Source {
                name: "p".to_owned(),
                major: 1,
                path: "protocols/p/1.yaml".to_owned(),
                text: source.to_owned(),
            },
        );

        let dimensions = section(&page, "## Independence dimensions");
        assert!(
            dimensions.contains("| Dimension | Description |"),
            "{dimensions}"
        );
        assert!(
            dimensions.contains(
                "| `different_agent_run` | the producer is another agent run than the one that did the work |"
            ),
            "{dimensions}"
        );
        assert!(
            dimensions.contains(
                "| `different_principal` | the producer is another principal than the one who did the work |"
            ),
            "{dimensions}"
        );

        let claims = section(&page, "## Claims");
        assert!(
            claims.contains(
                "| `reviewed` |  | evidence `security_review` with result `approved` about `implementation`, from a producer independent on `different_principal` |"
            ),
            "{claims}"
        );
        assert!(
            claims.contains(
                "evidence `security_review` with result `approved`, from a producer independent on `different_agent_run` and `different_principal`"
            ),
            "{claims}"
        );
        assert!(
            claims.contains(
                "not (evidence `security_review`, from a producer independent on `different_agent_run`)"
            ),
            "{claims}"
        );
        assert!(
            claims.contains("| `healthy` |  | evidence `observation` with result `healthy` |"),
            "{claims}"
        );

        let actions = section(&page, "## Actions");
        assert!(
            actions.contains(
                "| `ship` |  | evidence `security_review` with result `approved`, from a producer independent on `different_agent_run` |"
            ),
            "{actions}"
        );
        let outcomes = section(&page, "## Outcomes");
        assert!(
            outcomes.contains(
                "evidence `security_review` with result `approved` about `implementation`, from a producer independent on `different_principal`"
            ),
            "{outcomes}"
        );

        let kinds = section(&page, "## Evidence kinds");
        assert!(
            kinds.contains(
                "| Evidence kind | Description | Applies for | Produced by | Used by claims |"
            ),
            "{kinds}"
        );
        assert!(
            kinds.contains(
                "| `observation` | an observation of the running system | `30m` after it is observed | *no action* | `healthy` |"
            ),
            "{kinds}"
        );
        assert!(
            kinds.contains("| `security_review` |  | *no limit* |"),
            "{kinds}"
        );
    }

    /// A protocol that declares no independence dimension and no maximum age keeps the page it
    /// had before Canon 0.4.0: no dimensions section and no horizon column.
    #[test]
    fn a_page_without_independence_or_horizons_is_unchanged() {
        let source = include_str!("../tests/fixtures/case-inputs.yaml");
        let ir = b10x_canon::ir::compile(&parse(source).expect("parses")).expect("compiles");
        let page = page(
            &ir,
            &Source {
                name: "p".to_owned(),
                major: 1,
                path: "protocols/p/1.yaml".to_owned(),
                text: source.to_owned(),
            },
        );
        assert!(!page.contains("## Independence dimensions"), "{page}");
        assert!(!page.contains("Applies for"), "{page}");
        assert!(!page.contains("independent on"), "{page}");
    }
}
