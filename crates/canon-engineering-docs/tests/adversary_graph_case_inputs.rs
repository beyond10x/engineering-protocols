//! Adversarial review of `story:software-change-profiles`, at `51fb753`: the committed protocol
//! graphs against docs-system 0.9.0's `b10x-protocol-graph/1` additions, read independently of
//! `review_graph_schema.rs`, which checks that an obligation's `applicable_when` is present when
//! it should be but not what it says, and checks a case input's values but not its description.

use std::path::PathBuf;

use b10x_canon::ir::{Ir, compile};
use b10x_canon::model::{Predicate, Truth, parse};
use serde_json::{Value, json};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A predicate as the schema's `$defs/predicate` writes it, transcribed here from the schema: a
/// claim test without `is` when it is `true`, an evidence match without `subject`, an input test
/// as `{"input": {"id", "in"}}`.
fn drawn(predicate: &Predicate) -> Value {
    match predicate {
        Predicate::All(members) => json!({"all": members.iter().map(drawn).collect::<Vec<_>>()}),
        Predicate::Any(members) => json!({"any": members.iter().map(drawn).collect::<Vec<_>>()}),
        Predicate::Not(inner) => json!({"not": drawn(inner)}),
        Predicate::Claim(test) if test.is == Truth::True => {
            json!({"claim": {"id": test.claim.as_str()}})
        }
        Predicate::Claim(test) => {
            json!({"claim": {"id": test.claim.as_str(), "is": test.is.to_string()}})
        }
        Predicate::Evidence(matching) => match &matching.result {
            Some(result) => json!({"evidence": {"kind": matching.kind.as_str(), "result": result}}),
            None => json!({"evidence": {"kind": matching.kind.as_str()}}),
        },
        Predicate::Input(test) => json!({"input": {
            "id": test.input.as_str(),
            "in": test.r#in.iter().map(|value| value.as_str()).collect::<Vec<_>>(),
        }}),
    }
}

/// Every committed graph, with the protocol it was drawn from compiled.
fn graphs() -> Vec<(String, Value, Ir)> {
    let mut found = Vec::new();
    for directory in [
        "website/data/protocol-graphs",
        "website/data/example-graphs",
    ] {
        let Ok(entries) = std::fs::read_dir(root().join(directory)) else {
            continue;
        };
        for entry in entries {
            let path = entry.expect("an entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let name = path.display().to_string();
            let graph: Value =
                serde_json::from_str(&std::fs::read_to_string(&path).expect("reads"))
                    .unwrap_or_else(|error| panic!("{name}: {error}"));
            let source = graph["protocol"]["source"].as_str().expect("a source");
            let text = std::fs::read_to_string(root().join(source)).expect("the source reads");
            let ir = compile(&parse(&text).expect("parses")).expect("compiles");
            found.push((name, graph, ir));
        }
    }
    assert!(!found.is_empty(), "no committed graph");
    found
}

/// Each obligation's drawn `applicable_when` is its compiled one, and no other node carries the
/// key; each case input's description is its compiled one, trimmed, or `null`; `case` is present
/// exactly when the protocol declares an input, and then declares at least one.
#[test]
fn committed_graphs_draw_applicability_and_case_inputs_as_compiled() {
    let mut problems = Vec::new();
    let mut applicable = 0;
    for (name, graph, ir) in graphs() {
        for node in graph["nodes"].as_array().into_iter().flatten() {
            let id = node["id"].as_str().unwrap_or_default();
            let expected = id
                .strip_prefix("obligation:")
                .and_then(|obligation| {
                    ir.obligations
                        .iter()
                        .find(|(key, _)| key.as_str() == obligation)
                })
                .and_then(|(_, obligation)| obligation.applicable_when.as_ref())
                .map(drawn);
            if node.get("applicable_when").cloned() != expected {
                problems.push(format!(
                    "{name}: `{id}` draws applicable_when {:?}, compiled {expected:?}",
                    node.get("applicable_when")
                ));
            }
            applicable += usize::from(expected.is_some());
        }
        match (graph.get("case"), ir.inputs.is_empty()) {
            (None, true) => {}
            (Some(case), false) => {
                let inputs = case["inputs"].as_object().cloned().unwrap_or_default();
                if inputs.len() != ir.inputs.len() {
                    problems.push(format!("{name}: case declares {} inputs", inputs.len()));
                }
                for (id, input) in &ir.inputs {
                    let description = input
                        .description
                        .as_deref()
                        .map(str::trim)
                        .filter(|text| !text.is_empty());
                    if inputs.get(id.as_str()).map(|drawn| &drawn["description"])
                        != Some(&json!(description))
                    {
                        problems.push(format!(
                            "{name}: input `{id}` draws description {:?}, compiled {description:?}",
                            inputs.get(id.as_str()).map(|drawn| &drawn["description"])
                        ));
                    }
                }
            }
            (case, _) => problems.push(format!(
                "{name}: case {case:?} with {} compiled inputs",
                ir.inputs.len()
            )),
        }
    }
    assert!(
        applicable >= 4,
        "{applicable} obligations with applicable_when"
    );
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Every `in` list a committed graph draws, wherever a predicate holds one, is non-empty and
/// lists no value twice, as the schema's `uniqueItems` and `minItems` require.
#[test]
fn committed_graphs_draw_no_empty_or_repeated_in_list() {
    fn walk(value: &Value, at: &str, problems: &mut Vec<String>) {
        match value {
            Value::Object(object) => {
                if let Some(values) = object.get("input").and_then(|input| input.get("in")) {
                    let list = values.as_array().cloned().unwrap_or_default();
                    let mut unique = list.clone();
                    unique.sort_by_key(Value::to_string);
                    unique.dedup();
                    if list.is_empty() || unique.len() != list.len() {
                        problems.push(format!("{at}: in {values}"));
                    }
                }
                for (key, member) in object {
                    walk(member, &format!("{at}.{key}"), problems);
                }
            }
            Value::Array(members) => {
                for (index, member) in members.iter().enumerate() {
                    walk(member, &format!("{at}[{index}]"), problems);
                }
            }
            _ => {}
        }
    }
    let mut problems = Vec::new();
    for (name, graph, _) in graphs() {
        walk(&graph, &name, &mut problems);
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
