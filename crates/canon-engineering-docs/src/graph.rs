//! A compiled protocol as a `b10x-protocol-graph/1` document, the input of the docs-system
//! `ProtocolGraph` component.
//!
//! Nodes are ordered by kind (action, evidence, claim, outcome, obligation), then by name, and are
//! identified as `<kind>:<name>`. Edges: `produces` (action → evidence kind it may produce),
//! `establishes` (evidence kind → claim whose `true_when` matches it, qualified by the result and
//! the subject),
//! `supports` (claim → claim whose `true_when` tests it), `requires` (claim → outcome whose
//! `requires` tests it, and claim → obligation whose `discharged_when` tests it) and `gates`
//! (claim → action whose precondition tests it). The format's edge kinds are a closed set
//! (docs-system `schema/b10x.protocol-graph.v1.schema.json`), so an obligation's discharge reuses
//! `requires`, and an obligation node carries its `discharged_when` as its predicate. A claim edge
//! carries the value the predicate needs of the claim as its qualifier when that value is not
//! `true`, with polarity carried through `not`: under an odd number of `not`s, `{claim: c}` needs
//! `false`, `{claim: c, is: false}` needs `true` and `{claim: c, is: unknown}` needs
//! `not unknown`; an evidence edge under an odd number of `not`s is qualified `not <result>`, or
//! `not present` for a match without a result. A match bound to an artifact appends
//! `about <subject>` to that qualifier (`pass about implementation`, `about release`), so two
//! matches of one kind about different artifacts are two edges. A predicate's evidence match has no
//! `subject` key in the format, so the subject reaches the graph only through that qualifier. A
//! match that demands independence (Canon 0.4.0 `independent_on`) appends `, independent on
//! <dimensions>` after that, several joined with `and` (`approved about implementation,
//! independent on different_principal`); the format has no key for the demand either. An
//! outcome or a precondition that matches evidence directly keeps that match in its predicate,
//! without its subject or its independence demand; the format has no edge kind for it. A
//! discharge predicate tests claims only: Canon refuses one that matches evidence.
//!
//!
//! Case inputs (docs-system 0.9.0 `schema/b10x.protocol-graph.v1.schema.json`): a protocol that
//! declares any writes a top-level `case` object, `{"inputs": {<id>: {"description": <text or
//! null>, "values": [...]}}}`, each input with its values in `canon-ir/1`'s order; one that
//! declares none writes no `case` key. An input test is the predicate leaf
//! `{"input": {"id": <id>, "in": [...]}}`, as `canon-ir/1` writes it, wherever a predicate holds
//! one. An obligation node carries its `applicable_when` as a predicate of that key, omitted when
//! the obligation always applies; no other node kind carries it. An input is case data, not a
//! node, so an input test draws no edge, and neither does a claim tested only in
//! `applicable_when`: the format's edge kinds have none for applicability.
//!
//! Order forms (docs-system 0.10.0 `schema/b10x.protocol-graph.v1.schema.json`): Canon 0.3.0's
//! `{first: {kind, result, subject?}}` and `{before: {first, then, subject?}}` are the predicate
//! leaves `{"first": {"kind", "result", "subject"?}}` and `{"before": {"first", "then",
//! "subject"?}}`, as `canon-ir/1` writes them, `subject` only when the form names one; unlike an
//! evidence match, an order form keeps its subject in the leaf, because the schema has the key. In
//! a claim's `true_when` an order form draws an `establishes` edge from each kind it reads, as an
//! evidence match does: from a `first`'s kind qualified `first <result>`, and from a `before`'s two
//! kinds qualified `before <then>` and `after <first>`; each followed by `about <subject>` when the
//! form names one, and prefixed `not` under an odd number of `not`s. Anywhere else it draws no
//! edge, as an evidence match does not.

use b10x_canon::ir::Ir;
use b10x_canon::model::{ArtifactId, EvidenceMatch, Predicate, Truth};
use serde_json::{Map, Value, json};

/// The format this module emits.
pub const FORMAT: &str = "b10x-protocol-graph/1";

/// A predicate in the `b10x-protocol-graph/1` shape: a claim test omits `is` when it is `true`, an
/// evidence match omits `result` when it has none, and an input test is `{"input": {"id": …,
/// "in": [...]}}` as in `canon-ir/1`. Unlike `canon-ir/1`, an evidence match never carries
/// `subject`: the format has no key for it, and a match without one does not mean "about any
/// artifact" here. Nor does it carry `independent_on`. A claim's matches show their subjects and
/// independence demands on its `establishes` edges. An order form is `{"first": {"kind": …,
/// "result": …}}` or `{"before": {"first": …, "then": …}}` as in `canon-ir/1`, with `"subject"`
/// when it names one.
pub fn predicate(predicate: &Predicate) -> Value {
    match predicate {
        Predicate::All(members) => {
            json!({"all": members.iter().map(self::predicate).collect::<Vec<_>>()})
        }
        Predicate::Any(members) => {
            json!({"any": members.iter().map(self::predicate).collect::<Vec<_>>()})
        }
        Predicate::Not(inner) => json!({"not": self::predicate(inner)}),
        Predicate::Claim(test) => {
            let mut claim = Map::new();
            claim.insert("id".into(), json!(test.claim.as_str()));
            if test.is != Truth::True {
                claim.insert("is".into(), json!(test.is.to_string()));
            }
            json!({"claim": claim})
        }
        Predicate::Evidence(matching) => {
            let mut evidence = Map::new();
            evidence.insert("kind".into(), json!(matching.kind.as_str()));
            if let Some(result) = &matching.result {
                evidence.insert("result".into(), json!(result));
            }
            json!({"evidence": evidence})
        }
        Predicate::Input(test) => {
            let values: Vec<&str> = test.r#in.iter().map(|value| value.as_str()).collect();
            json!({"input": {"id": test.input.as_str(), "in": values}})
        }
        Predicate::First(form) => {
            let mut first = Map::new();
            first.insert("kind".into(), json!(form.kind.as_str()));
            first.insert("result".into(), json!(form.result));
            if let Some(subject) = &form.subject {
                first.insert("subject".into(), json!(subject.as_str()));
            }
            json!({"first": first})
        }
        Predicate::Before(form) => {
            let mut before = Map::new();
            before.insert("first".into(), json!(form.first.as_str()));
            before.insert("then".into(), json!(form.then.as_str()));
            if let Some(subject) = &form.subject {
                before.insert("subject".into(), json!(subject.as_str()));
            }
            json!({"before": before})
        }
        Predicate::After(form) => {
            let mut after = Map::new();
            after.insert("first".into(), json!(form.first.as_str()));
            after.insert(
                "then".into(),
                json!({"kind": form.then.kind.as_str(), "result": form.then.result}),
            );
            if let Some(subject) = &form.subject {
                after.insert("subject".into(), json!(subject.as_str()));
            }
            json!({"after": after})
        }
        Predicate::Since(form) => {
            let mut since = Map::new();
            since.insert("last".into(), json!(form.last.as_str()));
            since.insert(
                "then".into(),
                json!({"kind": form.then.kind.as_str(), "result": form.then.result}),
            );
            if let Some(subject) = &form.subject {
                since.insert("subject".into(), json!(subject.as_str()));
            }
            json!({"since": since})
        }
    }
}

/// The top-level `case` object: every case input `ir` declares, with its description (`null`
/// when it has none) and its values; `None` when it declares none.
fn case(ir: &Ir) -> Option<Value> {
    if ir.inputs.is_empty() {
        return None;
    }
    let inputs: Map<String, Value> = ir
        .inputs
        .iter()
        .map(|(id, input)| {
            let description = input
                .description
                .as_deref()
                .map(str::trim)
                .filter(|text| !text.is_empty());
            let values: Vec<&str> = input.values.iter().map(|value| value.as_str()).collect();
            (
                id.as_str().to_owned(),
                json!({"description": description, "values": values}),
            )
        })
        .collect();
    Some(json!({"inputs": inputs}))
}

fn node(kind: &str, name: &str, description: &Option<String>) -> Map<String, Value> {
    let mut node = Map::new();
    node.insert("id".into(), json!(format!("{kind}:{name}")));
    node.insert("kind".into(), json!(kind));
    node.insert("name".into(), json!(name));
    if let Some(text) = description
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        node.insert("description".into(), json!(text));
    }
    node
}

fn is_trivial(predicate: &Predicate) -> bool {
    matches!(predicate, Predicate::All(members) if members.is_empty())
}

#[derive(PartialEq)]
struct Edge {
    from: String,
    to: String,
    kind: &'static str,
    qualifier: Option<String>,
}

impl Edge {
    fn value(&self) -> Value {
        let mut edge = Map::new();
        edge.insert("from".into(), json!(self.from));
        edge.insert("to".into(), json!(self.to));
        edge.insert("kind".into(), json!(self.kind));
        if let Some(qualifier) = &self.qualifier {
            edge.insert("qualifier".into(), json!(qualifier));
        }
        Value::Object(edge)
    }
}

/// The value a claim test needs of its claim, as an edge qualifier: none for `true`. Under an odd
/// number of `not`s the test is negated: `not {claim: c}` holds exactly when `{claim: c, is: false}`
/// does, `not {claim: c, is: false}` when `c` is TRUE, and `not {claim: c, is: unknown}` when `c`
/// is decided, `not unknown`.
fn claim_qualifier(is: Truth, negated: bool) -> Option<String> {
    match (is, negated) {
        (Truth::True, false) | (Truth::False, true) => None,
        (Truth::True, true) => Some(Truth::False.to_string()),
        (Truth::Unknown, true) => Some(format!("not {}", Truth::Unknown)),
        (is, false) => Some(is.to_string()),
    }
}

/// The result an evidence match needs, as an edge qualifier; under an odd number of `not`s,
/// `not <result>`, or `not present` for a match without a result; followed by `about <subject>`
/// when the match is bound to an artifact; then `, independent on <dimensions>` (joined with `and`)
/// when it demands independence, or that demand alone when nothing precedes it.
fn evidence_qualifier(matching: &EvidenceMatch, negated: bool) -> Option<String> {
    let result = match (&matching.result, negated) {
        (result, false) => result.clone(),
        (Some(result), true) => Some(format!("not {result}")),
        (None, true) => Some("not present".to_owned()),
    };
    let qualifier = match (result, &matching.subject) {
        (result, None) => result,
        (Some(result), Some(subject)) => Some(format!("{result} about {subject}")),
        (None, Some(subject)) => Some(format!("about {subject}")),
    };
    match matching
        .independent_on
        .as_deref()
        .filter(|dimensions| !dimensions.is_empty())
    {
        None => qualifier,
        Some(dimensions) => {
            let dimensions: Vec<&str> = dimensions.iter().map(|d| d.as_str()).collect();
            let demand = format!("independent on {}", dimensions.join(" and "));
            Some(match qualifier {
                Some(qualifier) => format!("{qualifier}, {demand}"),
                None => demand,
            })
        }
    }
}

/// The qualifier of an edge an order form draws from one kind it reads: `relation` (`first fail`,
/// `before K2`, `after K1`), prefixed `not` under an odd number of `not`s and followed by `about
/// <subject>` when the form names one.
fn order_qualifier(relation: String, subject: Option<&ArtifactId>, negated: bool) -> String {
    let relation = if negated {
        format!("not {relation}")
    } else {
        relation
    };
    match subject {
        Some(subject) => format!("{relation} about {subject}"),
        None => relation,
    }
}

/// Calls `f` on every claim test, evidence match, input test and order form of `predicate`, in the
/// order [`Predicate::visit`] reaches them, with whether an odd number of `not`s encloses it.
fn leaves<'a>(predicate: &'a Predicate, negated: bool, f: &mut impl FnMut(&'a Predicate, bool)) {
    match predicate {
        Predicate::All(members) | Predicate::Any(members) => {
            for member in members {
                leaves(member, negated, f);
            }
        }
        Predicate::Not(inner) => leaves(inner, !negated, f),
        Predicate::Claim(_)
        | Predicate::Evidence(_)
        | Predicate::Input(_)
        | Predicate::First(_)
        | Predicate::Before(_)
        | Predicate::After(_)
        | Predicate::Since(_) => f(predicate, negated),
    }
}

/// One edge per claim `predicate` tests, from the claim to `to`, of kind `kind`, qualified by the
/// value the predicate needs of the claim.
fn claim_edges(predicate: &Predicate, to: &str, kind: &'static str, add: &mut impl FnMut(Edge)) {
    leaves(predicate, false, &mut |leaf, negated| {
        if let Predicate::Claim(test) = leaf {
            add(Edge {
                from: format!("claim:{}", test.claim),
                to: to.to_owned(),
                kind,
                qualifier: claim_qualifier(test.is, negated),
            });
        }
    });
}

/// The graph document for one compiled protocol; `source` names the document it came from.
pub fn document(ir: &Ir, source: &str) -> Value {
    let mut nodes = Vec::new();
    for (id, action) in &ir.actions {
        let mut value = node("action", id.as_str(), &action.description);
        if !is_trivial(&action.precondition) {
            value.insert("predicate".into(), predicate(&action.precondition));
        }
        if let Some(effect) = &action.effect {
            value.insert("effect".into(), json!(effect.as_str()));
        }
        if !action.requires.is_empty() {
            let capabilities: Vec<_> = action.requires.iter().map(|c| c.as_str()).collect();
            value.insert("capabilities".into(), json!(capabilities));
        }
        nodes.push(Value::Object(value));
    }
    for (id, kind) in &ir.evidence_kinds {
        nodes.push(Value::Object(node(
            "evidence",
            id.as_str(),
            &kind.description,
        )));
    }
    for (id, claim) in &ir.claims {
        let mut value = node("claim", id.as_str(), &claim.description);
        value.insert("predicate".into(), predicate(&claim.true_when));
        nodes.push(Value::Object(value));
    }
    for (id, outcome) in &ir.outcomes {
        let mut value = node("outcome", id.as_str(), &outcome.description);
        if let Some(requires) = outcome.requires.predicate() {
            value.insert("predicate".into(), predicate(requires));
        }
        nodes.push(Value::Object(value));
    }
    for (id, obligation) in &ir.obligations {
        let mut value = node("obligation", id.as_str(), &obligation.description);
        value.insert("predicate".into(), predicate(&obligation.discharged_when));
        if let Some(applicable_when) = &obligation.applicable_when {
            value.insert("applicable_when".into(), predicate(applicable_when));
        }
        nodes.push(Value::Object(value));
    }

    let mut edges: Vec<Edge> = Vec::new();
    let mut add = |edge: Edge| {
        if !edges.contains(&edge) {
            edges.push(edge);
        }
    };
    for (id, action) in &ir.actions {
        for kind in &action.may_produce {
            add(Edge {
                from: format!("action:{id}"),
                to: format!("evidence:{kind}"),
                kind: "produces",
                qualifier: None,
            });
        }
    }
    for (id, claim) in &ir.claims {
        leaves(&claim.true_when, false, &mut |leaf, negated| match leaf {
            Predicate::Evidence(matching) => add(Edge {
                from: format!("evidence:{}", matching.kind),
                to: format!("claim:{id}"),
                kind: "establishes",
                qualifier: evidence_qualifier(matching, negated),
            }),
            Predicate::Claim(test) => add(Edge {
                from: format!("claim:{}", test.claim),
                to: format!("claim:{id}"),
                kind: "supports",
                qualifier: claim_qualifier(test.is, negated),
            }),
            Predicate::First(form) => add(Edge {
                from: format!("evidence:{}", form.kind),
                to: format!("claim:{id}"),
                kind: "establishes",
                qualifier: Some(order_qualifier(
                    format!("first {}", form.result),
                    form.subject.as_ref(),
                    negated,
                )),
            }),
            Predicate::Before(form) => {
                add(Edge {
                    from: format!("evidence:{}", form.first),
                    to: format!("claim:{id}"),
                    kind: "establishes",
                    qualifier: Some(order_qualifier(
                        format!("before {}", form.then),
                        form.subject.as_ref(),
                        negated,
                    )),
                });
                add(Edge {
                    from: format!("evidence:{}", form.then),
                    to: format!("claim:{id}"),
                    kind: "establishes",
                    qualifier: Some(order_qualifier(
                        format!("after {}", form.first),
                        form.subject.as_ref(),
                        negated,
                    )),
                });
            }
            Predicate::After(form) => {
                add(Edge {
                    from: format!("evidence:{}", form.first),
                    to: format!("claim:{id}"),
                    kind: "establishes",
                    qualifier: Some(order_qualifier(
                        format!("before {} {}", form.then.kind, form.then.result),
                        form.subject.as_ref(),
                        negated,
                    )),
                });
                add(Edge {
                    from: format!("evidence:{}", form.then.kind),
                    to: format!("claim:{id}"),
                    kind: "establishes",
                    qualifier: Some(order_qualifier(
                        format!("{} after {}", form.then.result, form.first),
                        form.subject.as_ref(),
                        negated,
                    )),
                });
            }
            Predicate::Since(form) => {
                add(Edge {
                    from: format!("evidence:{}", form.last),
                    to: format!("claim:{id}"),
                    kind: "establishes",
                    qualifier: Some(order_qualifier(
                        format!("latest, before {} {}", form.then.kind, form.then.result),
                        form.subject.as_ref(),
                        negated,
                    )),
                });
                add(Edge {
                    from: format!("evidence:{}", form.then.kind),
                    to: format!("claim:{id}"),
                    kind: "establishes",
                    qualifier: Some(order_qualifier(
                        format!("{} since {}", form.then.result, form.last),
                        form.subject.as_ref(),
                        negated,
                    )),
                });
            }
            Predicate::All(_) | Predicate::Any(_) | Predicate::Not(_) | Predicate::Input(_) => {}
        });
    }
    for (id, outcome) in &ir.outcomes {
        if let Some(requires) = outcome.requires.predicate() {
            claim_edges(requires, &format!("outcome:{id}"), "requires", &mut add);
        }
    }
    for (id, obligation) in &ir.obligations {
        claim_edges(
            &obligation.discharged_when,
            &format!("obligation:{id}"),
            "requires",
            &mut add,
        );
    }
    for (id, action) in &ir.actions {
        claim_edges(
            &action.precondition,
            &format!("action:{id}"),
            "gates",
            &mut add,
        );
    }

    let mut protocol = Map::new();
    protocol.insert("id".into(), json!(ir.protocol.id.as_str()));
    protocol.insert("revision".into(), json!(ir.protocol.revision));
    if let Some(text) = ir
        .protocol
        .description
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        protocol.insert("description".into(), json!(text));
    }
    protocol.insert("source".into(), json!(source));
    let mut document = json!({
        "format": FORMAT,
        "protocol": protocol,
        "nodes": nodes,
        "edges": edges.iter().map(Edge::value).collect::<Vec<_>>(),
    });
    if let Some(case) = case(ir) {
        document["case"] = case;
    }
    document
}

/// Every node name of one kind, in document order.
#[cfg(test)]
pub fn names(document: &Value, kind: &str) -> Vec<String> {
    document["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|node| node["kind"] == kind)
        .filter_map(|node| node["name"].as_str().map(str::to_owned))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use b10x_canon::ir::compile;
    use b10x_canon::model::parse;

    const INVESTIGATION: &str = include_str!("../tests/fixtures/investigation.yaml");
    /// docs-system `examples/product-site/data/investigation.protocol-graph.json` at `929f965`,
    /// copied byte for byte: the reference rendering of the same fixture.
    const REFERENCE: &str = include_str!("../tests/fixtures/investigation.protocol-graph.json");

    fn sorted(value: &Value) -> Vec<String> {
        let mut items: Vec<String> = value
            .as_array()
            .expect("array")
            .iter()
            .map(Value::to_string)
            .collect();
        items.sort();
        items
    }

    #[test]
    fn the_investigation_fixture_matches_the_reference_graph() {
        let ir = compile(&parse(INVESTIGATION).expect("parses")).expect("compiles");
        let ours = document(&ir, "protocols/investigation/1.yaml");
        let reference: Value = serde_json::from_str(REFERENCE).expect("reference");
        assert_eq!(ours["format"], reference["format"]);
        assert_eq!(ours["nodes"], reference["nodes"]);
        assert_eq!(sorted(&ours["edges"]), sorted(&reference["edges"]));
        assert_eq!(ours["protocol"]["id"], reference["protocol"]["id"]);
        assert_eq!(
            ours["protocol"]["revision"],
            reference["protocol"]["revision"]
        );
        assert_eq!(
            ours["protocol"]["description"],
            reference["protocol"]["description"]
        );
    }

    #[test]
    fn preconditions_gate_and_negative_tests_are_qualified() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {e: {}}\nclaims:\n  a:\n    true_when: {evidence: {kind: e}}\n  b:\n    true_when: {not: {claim: a, is: unknown}}\nactions:\n  go:\n    precondition: {claim: b}\n    requires: [{capability: c}]\n    effect: write\noutcomes:\n  done:\n    requires: {claim: b, is: false}\n";
        let ir = compile(&parse(source).expect("parses")).expect("compiles");
        let graph = document(&ir, "x");
        let edges = sorted(&graph["edges"]);
        assert!(
            edges.contains(
                &r#"{"from":"claim:a","kind":"supports","qualifier":"not unknown","to":"claim:b"}"#
                    .to_owned()
            ),
            "{edges:?}"
        );
        assert!(
            edges.contains(&r#"{"from":"claim:b","kind":"gates","to":"action:go"}"#.to_owned()),
            "{edges:?}"
        );
        assert!(
            edges.contains(
                &r#"{"from":"claim:b","kind":"requires","qualifier":"false","to":"outcome:done"}"#
                    .to_owned()
            ),
            "{edges:?}"
        );
        let action = &graph["nodes"][0];
        assert_eq!(action["capabilities"], json!(["c"]));
        assert_eq!(action["effect"], json!("write"));
        assert_eq!(action["predicate"], json!({"claim": {"id": "b"}}));
    }

    #[test]
    fn an_obligation_requires_the_claims_its_discharge_tests() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {e: {}}\nclaims:\n  a:\n    true_when: {evidence: {kind: e}}\n  b:\n    true_when: {evidence: {kind: e, result: ok}}\nobligations:\n  settle:\n    discharged_when: {all: [{claim: a}, {claim: b, is: false}]}\n";
        let ir = compile(&parse(source).expect("parses")).expect("compiles");
        let graph = document(&ir, "x");
        let edges: Vec<String> = sorted(&graph["edges"])
            .into_iter()
            .filter(|edge| edge.contains("obligation:settle"))
            .collect();
        assert_eq!(
            edges,
            [
                r#"{"from":"claim:a","kind":"requires","to":"obligation:settle"}"#,
                r#"{"from":"claim:b","kind":"requires","qualifier":"false","to":"obligation:settle"}"#,
            ]
        );
        let obligation = graph["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["id"] == "obligation:settle")
            .expect("the obligation node");
        assert_eq!(
            obligation["predicate"],
            json!({"all": [{"claim": {"id": "a"}}, {"claim": {"id": "b", "is": "false"}}]})
        );
    }

    /// Every edge drawn from a predicate carries the value the predicate needs, with polarity
    /// carried through `not`: once negated, twice not.
    #[test]
    fn a_claim_or_evidence_under_not_is_qualified_by_the_value_it_needs() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {e: {}}\nclaims:\n  a:\n    true_when: {not: {evidence: {kind: e, result: ok}}}\n  b:\n    true_when: {any: [{not: {evidence: {kind: e}}}, {not: {not: {claim: a}}}]}\n  c:\n    true_when: {not: {claim: a, is: false}}\nobligations:\n  settle:\n    discharged_when: {not: {claim: a}}\nactions:\n  go:\n    precondition: {not: {claim: b, is: unknown}}\noutcomes:\n  done:\n    requires: {all: [{not: {claim: c}}, {claim: a}]}\n";
        let ir = compile(&parse(source).expect("parses")).expect("compiles");
        let mut edges = sorted(&document(&ir, "x")["edges"]);
        edges.sort();
        let mut expected = vec![
            r#"{"from":"claim:a","kind":"requires","qualifier":"false","to":"obligation:settle"}"#,
            r#"{"from":"claim:a","kind":"requires","to":"outcome:done"}"#,
            r#"{"from":"claim:a","kind":"supports","to":"claim:b"}"#,
            r#"{"from":"claim:a","kind":"supports","to":"claim:c"}"#,
            r#"{"from":"claim:b","kind":"gates","qualifier":"not unknown","to":"action:go"}"#,
            r#"{"from":"claim:c","kind":"requires","qualifier":"false","to":"outcome:done"}"#,
            r#"{"from":"evidence:e","kind":"establishes","qualifier":"not ok","to":"claim:a"}"#,
            r#"{"from":"evidence:e","kind":"establishes","qualifier":"not present","to":"claim:b"}"#,
        ];
        expected.sort();
        assert_eq!(edges, expected);
    }

    /// A protocol with case inputs draws them: a top-level `case.inputs` declaring each input with
    /// its description (`null` without one) and its values in `canon-ir/1`'s order, an obligation's
    /// `applicable_when` on its node, and an input test as the `input` leaf wherever a predicate
    /// holds one, under `any`, `all` and `not` too. An input test draws no edge.
    #[test]
    fn the_graph_draws_case_inputs_applicability_and_input_tests() {
        const CASE_INPUTS: &str = include_str!("../tests/fixtures/case-inputs.yaml");
        let source = format!(
            "{}actions:\n  go:\n    precondition: {{any: [{{input: risk, in: [low]}}, {{claim: a}}]}}\n  stop:\n    precondition: {{not: {{input: risk, in: [high, low]}}}}\noutcomes:\n  done:\n    requires: {{all: [{{claim: a}}, {{input: risk, in: [high]}}]}}\n",
            CASE_INPUTS.replace(
                "case: {inputs: {risk: {values: [low, high]}}}",
                "case: {inputs: {risk: {values: [low, high]}, size: {description: how big, values: [big]}}}"
            )
        );
        let ir = compile(&parse(&source).expect("parses")).expect("compiles");
        let graph = document(&ir, "x");
        assert_eq!(
            graph["case"],
            json!({"inputs": {
                "risk": {"description": null, "values": ["high", "low"]},
                "size": {"description": "how big", "values": ["big"]},
            }})
        );
        let node = |id: &str| {
            graph["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .find(|node| node["id"] == id)
                .unwrap_or_else(|| panic!("no node `{id}`"))
                .clone()
        };
        let review = node("obligation:review");
        assert_eq!(
            review["applicable_when"],
            json!({"input": {"id": "risk", "in": ["high"]}})
        );
        assert_eq!(review["predicate"], json!({"claim": {"id": "a"}}));
        assert_eq!(
            node("action:go")["predicate"],
            json!({"any": [{"claim": {"id": "a"}}, {"input": {"id": "risk", "in": ["low"]}}]})
        );
        assert_eq!(
            node("action:stop")["predicate"],
            json!({"not": {"input": {"id": "risk", "in": ["high", "low"]}}})
        );
        assert_eq!(
            node("outcome:done")["predicate"],
            json!({"all": [{"claim": {"id": "a"}}, {"input": {"id": "risk", "in": ["high"]}}]})
        );
        for kind in ["action", "evidence", "claim", "outcome"] {
            for name in names(&graph, kind) {
                assert!(
                    node(&format!("{kind}:{name}"))
                        .get("applicable_when")
                        .is_none(),
                    "{kind} `{name}` carries applicable_when"
                );
            }
        }
        let edges = sorted(&graph["edges"]);
        assert!(edges.iter().all(|edge| !edge.contains("risk")), "{edges:?}");
        assert!(
            edges.contains(&r#"{"from":"claim:a","kind":"gates","to":"action:go"}"#.to_owned()),
            "{edges:?}"
        );
    }

    /// A protocol without case inputs writes no `case` key, and an obligation that always applies
    /// carries no `applicable_when`.
    #[test]
    fn a_protocol_without_case_inputs_writes_no_case_and_no_applicability() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {e: {}}\nclaims:\n  a:\n    true_when: {evidence: {kind: e}}\nobligations:\n  settle:\n    discharged_when: {claim: a}\n";
        let ir = compile(&parse(source).expect("parses")).expect("compiles");
        let graph = document(&ir, "x");
        assert!(graph.get("case").is_none(), "{graph}");
        let obligation = graph["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["id"] == "obligation:settle")
            .expect("the obligation node");
        assert!(obligation.get("applicable_when").is_none(), "{obligation}");
    }

    /// A bound match carries its subject on its edge, with or without a result and under `not`,
    /// and two matches of one kind about different artifacts stay two edges.
    #[test]
    fn an_evidence_edge_carries_the_subject_of_its_match() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {x: {}, y: {}}\nevidence_kinds: {e: {}}\nclaims:\n  a:\n    true_when: {all: [{evidence: {kind: e, result: ok, subject: x}}, {evidence: {kind: e, result: ok, subject: y}}]}\n  b:\n    true_when: {any: [{evidence: {kind: e, subject: x}}, {not: {evidence: {kind: e, result: ok, subject: y}}}, {not: {evidence: {kind: e, subject: y}}}]}\n";
        let ir = compile(&parse(source).expect("parses")).expect("compiles");
        let mut edges = sorted(&document(&ir, "x")["edges"]);
        edges.sort();
        let mut expected = vec![
            r#"{"from":"evidence:e","kind":"establishes","qualifier":"ok about x","to":"claim:a"}"#,
            r#"{"from":"evidence:e","kind":"establishes","qualifier":"ok about y","to":"claim:a"}"#,
            r#"{"from":"evidence:e","kind":"establishes","qualifier":"about x","to":"claim:b"}"#,
            r#"{"from":"evidence:e","kind":"establishes","qualifier":"not ok about y","to":"claim:b"}"#,
            r#"{"from":"evidence:e","kind":"establishes","qualifier":"not present about y","to":"claim:b"}"#,
        ];
        expected.sort();
        assert_eq!(edges, expected);
    }

    /// An order form is drawn as its leaf, `{"first": {kind, result, subject?}}` or
    /// `{"before": {first, then, subject?}}` as `canon-ir/1` writes it, with `subject` only when
    /// the form names one, wherever a predicate holds one, under `any`, `all` and `not` too. In a
    /// claim's `true_when` it draws an `establishes` edge from each kind it reads: `first <result>`
    /// from a `first`'s kind, `before <then>` from a `before`'s first kind and `after <first>` from
    /// its then kind, each followed by `about <subject>` when bound and prefixed `not` under an odd
    /// number of `not`s. Elsewhere it draws no edge, as an evidence match does not.
    #[test]
    fn an_order_form_is_drawn_as_its_leaf_with_edges_from_the_kinds_it_reads() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {x: {}}\nevidence_kinds: {e: {}, f: {}}\nclaims:\n  a:\n    true_when: {any: [{evidence: {kind: e, result: fail}}, {first: {kind: e, result: fail, subject: x}}]}\n  b:\n    true_when: {all: [{before: {first: e, then: f, subject: x}}, {not: {first: {kind: f, result: ok}}}]}\n  c:\n    true_when: {not: {before: {first: f, then: e}}}\nactions:\n  go:\n    precondition: {not: {first: {kind: f, result: ok}}}\noutcomes:\n  done:\n    requires: {all: [{claim: a}, {before: {first: e, then: f}}]}\n";
        let ir = compile(&parse(source).expect("parses")).expect("compiles");
        let graph = document(&ir, "protocols/p/1.yaml");
        let node = |id: &str| {
            graph["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .find(|node| node["id"] == id)
                .unwrap_or_else(|| panic!("no node `{id}`"))
                .clone()
        };
        assert_eq!(
            node("claim:a")["predicate"],
            json!({"any": [
                {"evidence": {"kind": "e", "result": "fail"}},
                {"first": {"kind": "e", "result": "fail", "subject": "x"}},
            ]})
        );
        assert_eq!(
            node("claim:b")["predicate"],
            json!({"all": [
                {"before": {"first": "e", "then": "f", "subject": "x"}},
                {"not": {"first": {"kind": "f", "result": "ok"}}},
            ]})
        );
        assert_eq!(
            node("claim:c")["predicate"],
            json!({"not": {"before": {"first": "f", "then": "e"}}})
        );
        assert_eq!(
            node("action:go")["predicate"],
            json!({"not": {"first": {"kind": "f", "result": "ok"}}})
        );
        assert_eq!(
            node("outcome:done")["predicate"],
            json!({"all": [{"before": {"first": "e", "then": "f"}}, {"claim": {"id": "a"}}]})
        );
        let mut edges = sorted(&graph["edges"]);
        edges.sort();
        let mut expected = vec![
            r#"{"from":"claim:a","kind":"requires","to":"outcome:done"}"#,
            r#"{"from":"evidence:e","kind":"establishes","qualifier":"fail","to":"claim:a"}"#,
            r#"{"from":"evidence:e","kind":"establishes","qualifier":"first fail about x","to":"claim:a"}"#,
            r#"{"from":"evidence:e","kind":"establishes","qualifier":"before f about x","to":"claim:b"}"#,
            r#"{"from":"evidence:f","kind":"establishes","qualifier":"after e about x","to":"claim:b"}"#,
            r#"{"from":"evidence:f","kind":"establishes","qualifier":"not first ok","to":"claim:b"}"#,
            r#"{"from":"evidence:f","kind":"establishes","qualifier":"not before e","to":"claim:c"}"#,
            r#"{"from":"evidence:e","kind":"establishes","qualifier":"not after f","to":"claim:c"}"#,
        ];
        expected.sort();
        assert_eq!(edges, expected);
    }

    /// A match that demands independence carries the dimensions it demands on its edge, after its
    /// result and subject (`approved about implementation, independent on different_principal`),
    /// several joined with `and`, under `not` too and on a match with neither result nor subject;
    /// a match that demands none keeps its qualifier.
    #[test]
    fn an_evidence_edge_carries_the_independence_its_match_demands() {
        let source = include_str!("../tests/fixtures/independence-freshness.yaml");
        let ir = compile(&parse(source).expect("parses")).expect("compiles");
        let mut edges: Vec<String> = sorted(&document(&ir, "protocols/p/1.yaml")["edges"])
            .into_iter()
            .filter(|edge| edge.contains(r#""establishes""#))
            .collect();
        edges.sort();
        let mut expected = vec![
            r#"{"from":"evidence:security_review","kind":"establishes","qualifier":"approved about implementation, independent on different_principal","to":"claim:reviewed"}"#,
            r#"{"from":"evidence:security_review","kind":"establishes","qualifier":"approved, independent on different_agent_run and different_principal","to":"claim:doubly_reviewed"}"#,
            r#"{"from":"evidence:security_review","kind":"establishes","qualifier":"not rejected, independent on different_principal","to":"claim:doubly_reviewed"}"#,
            r#"{"from":"evidence:security_review","kind":"establishes","qualifier":"not present, independent on different_agent_run","to":"claim:doubly_reviewed"}"#,
            r#"{"from":"evidence:observation","kind":"establishes","qualifier":"healthy","to":"claim:healthy"}"#,
        ];
        expected.sort();
        assert_eq!(edges, expected);
    }
}
