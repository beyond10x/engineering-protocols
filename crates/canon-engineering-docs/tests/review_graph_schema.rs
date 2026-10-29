//! Independent review of `story:incident-response-protocol` (wave 2026-10-04-w8), at 0d4eccc.
//!
//! Invariants under check, for every committed `website/data/protocol-graphs/*.json`:
//!
//! 1. It conforms to the pinned docs-system schema, `schema/b10x.protocol-graph.v1.schema.json`
//!    at docs-system 0.14.0, `d7c8084` (the commit `website/package.json` pins), transcribed
//!    below: closed key sets, required keys, the node and edge kind enums, non-empty strings, the
//!    predicate shapes (the `input` leaf and the order-form leaves `first`, `before`, `after`
//!    and `since` among them, and the renderer's refusal of a `before` of one kind before
//!    itself), the optional top-level `case.inputs`, and `applicable_when`, a predicate on a node
//!    of kind `obligation` only. Nothing else in this
//!    repository checks a generated graph against that schema; the unit tests compare one
//!    reference rendering byte for byte, which has no obligation in it.
//! 2. It says what the compiled protocol says: one node per declared action, evidence kind,
//!    claim, outcome and obligation, ordered by kind then name; each action's effect and
//!    capabilities are its compiled `effect` and `requires`; every claim an obligation's
//!    discharge or an action's precondition tests has an edge into it; `case.inputs` declares
//!    exactly the compiled case inputs with their values, and is absent when there are none; an
//!    obligation carries `applicable_when` exactly when its compiled one is given; and every
//!    `input` leaf names a declared input and only values it declares.

use std::collections::BTreeSet;
use std::path::PathBuf;

use b10x_canon::ir::{Ir, compile};
use b10x_canon::model::{Predicate, parse};
use serde_json::{Map, Value};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn closed(
    object: &Map<String, Value>,
    allowed: &[&str],
    required: &[&str],
    at: &str,
) -> Vec<String> {
    let mut problems = Vec::new();
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            problems.push(format!("{at}: key `{key}` is not in the schema"));
        }
    }
    for key in required {
        if !object.contains_key(*key) {
            problems.push(format!("{at}: required key `{key}` is missing"));
        }
    }
    problems
}

fn nonempty(value: &Value, at: &str) -> Vec<String> {
    match value.as_str() {
        Some(text) if !text.is_empty() => Vec::new(),
        _ => vec![format!("{at}: not a non-empty string: {value}")],
    }
}

fn predicate(value: &Value, at: &str) -> Vec<String> {
    let Some(object) = value.as_object() else {
        return vec![format!("{at}: predicate is not an object")];
    };
    if object.len() != 1 {
        return vec![format!(
            "{at}: predicate has {} keys, expected one",
            object.len()
        )];
    }
    let (key, inner) = object.iter().next().expect("one key");
    match key.as_str() {
        "all" | "any" => match inner.as_array() {
            Some(members) => members
                .iter()
                .enumerate()
                .flat_map(|(index, member)| predicate(member, &format!("{at}.{key}[{index}]")))
                .collect(),
            None => vec![format!("{at}.{key}: not an array")],
        },
        "not" => predicate(inner, &format!("{at}.not")),
        "claim" => match inner.as_object() {
            Some(claim) => {
                let mut problems = closed(claim, &["id", "is"], &["id"], &format!("{at}.claim"));
                problems.extend(nonempty(&claim["id"], &format!("{at}.claim.id")));
                if let Some(is) = claim.get("is")
                    && !["true", "false", "unknown"].contains(&is.as_str().unwrap_or(""))
                {
                    problems.push(format!("{at}.claim.is: {is} is not true, false or unknown"));
                }
                problems
            }
            None => vec![format!("{at}.claim: not an object")],
        },
        "evidence" => match inner.as_object() {
            Some(evidence) => {
                let mut problems = closed(
                    evidence,
                    &["kind", "result"],
                    &["kind"],
                    &format!("{at}.evidence"),
                );
                problems.extend(nonempty(&evidence["kind"], &format!("{at}.evidence.kind")));
                if let Some(result) = evidence.get("result")
                    && !result.is_null()
                {
                    problems.extend(nonempty(result, &format!("{at}.evidence.result")));
                }
                problems
            }
            None => vec![format!("{at}.evidence: not an object")],
        },
        "input" => match inner.as_object() {
            Some(input) => {
                let mut problems =
                    closed(input, &["id", "in"], &["id", "in"], &format!("{at}.input"));
                if let Some(id) = input.get("id") {
                    problems.extend(nonempty(id, &format!("{at}.input.id")));
                }
                if let Some(values) = input.get("in") {
                    problems.extend(values_list(values, &format!("{at}.input.in")));
                }
                problems
            }
            None => vec![format!("{at}.input: not an object")],
        },
        "first" => match inner.as_object() {
            Some(first) => {
                let mut problems = closed(
                    first,
                    &["kind", "result", "subject"],
                    &["kind", "result"],
                    &format!("{at}.first"),
                );
                for key in ["kind", "result", "subject"] {
                    if let Some(value) = first.get(key) {
                        problems.extend(nonempty(value, &format!("{at}.first.{key}")));
                    }
                }
                problems
            }
            None => vec![format!("{at}.first: not an object")],
        },
        "before" => match inner.as_object() {
            Some(before) => {
                let mut problems = closed(
                    before,
                    &["first", "then", "subject"],
                    &["first", "then"],
                    &format!("{at}.before"),
                );
                for key in ["first", "then", "subject"] {
                    if let Some(value) = before.get(key) {
                        problems.extend(nonempty(value, &format!("{at}.before.{key}")));
                    }
                }
                // Not the schema's rule but its renderer's: `parseProtocolGraph` refuses it.
                if let (Some(first), Some(then)) = (before.get("first"), before.get("then"))
                    && first == then
                {
                    problems.push(format!(
                        "{at}.before: orders kind {first} before itself, which the renderer refuses"
                    ));
                }
                problems
            }
            None => vec![format!("{at}.before: not an object")],
        },
        form @ ("after" | "since") => match inner.as_object() {
            Some(after) => {
                let anchor = if form == "after" { "first" } else { "last" };
                let mut problems = closed(
                    after,
                    &[anchor, "then", "subject"],
                    &[anchor, "then"],
                    &format!("{at}.{form}"),
                );
                for key in [anchor, "subject"] {
                    if let Some(value) = after.get(key) {
                        problems.extend(nonempty(value, &format!("{at}.{form}.{key}")));
                    }
                }
                match after.get("then").map(Value::as_object) {
                    Some(Some(then)) => {
                        problems.extend(closed(
                            then,
                            &["kind", "result"],
                            &["kind", "result"],
                            &format!("{at}.{form}.then"),
                        ));
                        for key in ["kind", "result"] {
                            if let Some(value) = then.get(key) {
                                problems
                                    .extend(nonempty(value, &format!("{at}.{form}.then.{key}")));
                            }
                        }
                    }
                    Some(None) => problems.push(format!("{at}.{form}.then: not an object")),
                    None => {}
                }
                problems
            }
            None => vec![format!("{at}.{form}: not an object")],
        },
        other => vec![format!("{at}: `{other}` is not a predicate form")],
    }
}

/// A list of values as the schema requires one: an array of at least one item, each a
/// non-empty string, no item twice.
fn values_list(value: &Value, at: &str) -> Vec<String> {
    let Some(items) = value.as_array() else {
        return vec![format!("{at}: not an array")];
    };
    let mut problems = Vec::new();
    if items.is_empty() {
        problems.push(format!("{at}: empty"));
    }
    let mut seen = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        problems.extend(nonempty(item, &format!("{at}[{index}]")));
        if !seen.insert(item.to_string()) {
            problems.push(format!("{at}[{index}]: {item} is listed twice"));
        }
    }
    problems
}

/// Problems with the top-level `case` object against the schema.
fn case_problems(case: &Value) -> Vec<String> {
    let Some(case) = case.as_object() else {
        return vec!["$.case: not an object".to_owned()];
    };
    let mut problems = closed(case, &["inputs"], &["inputs"], "$.case");
    let Some(inputs) = case.get("inputs") else {
        return problems;
    };
    let Some(inputs) = inputs.as_object() else {
        problems.push("$.case.inputs: not an object".to_owned());
        return problems;
    };
    for (id, input) in inputs {
        let at = format!("$.case.inputs.{id}");
        if id.is_empty() {
            problems.push(format!("{at}: an empty input name"));
        }
        let Some(input) = input.as_object() else {
            problems.push(format!("{at}: not an object"));
            continue;
        };
        problems.extend(closed(input, &["description", "values"], &["values"], &at));
        if let Some(description) = input.get("description")
            && !description.is_null()
        {
            problems.extend(nonempty(description, &format!("{at}.description")));
        }
        if let Some(values) = input.get("values") {
            problems.extend(values_list(values, &format!("{at}.values")));
        }
    }
    problems
}

/// Problems with `graph` against the pinned schema.
fn schema_problems(graph: &Value) -> Vec<String> {
    let Some(top) = graph.as_object() else {
        return vec!["the document is not an object".to_owned()];
    };
    let mut problems = closed(
        top,
        &["format", "case", "protocol", "nodes", "edges"],
        &["format", "protocol", "nodes", "edges"],
        "$",
    );
    if let Some(case) = top.get("case") {
        problems.extend(case_problems(case));
    }
    if top.get("format") != Some(&Value::from("b10x-protocol-graph/1")) {
        problems.push(format!("$.format is {:?}", top.get("format")));
    }
    match top.get("protocol").and_then(Value::as_object) {
        Some(protocol) => {
            problems.extend(closed(
                protocol,
                &["id", "revision", "description", "source"],
                &["id", "revision"],
                "$.protocol",
            ));
            problems.extend(nonempty(&protocol["id"], "$.protocol.id"));
            if protocol["revision"]
                .as_u64()
                .is_none_or(|revision| revision < 1)
            {
                problems.push(format!("$.protocol.revision: {}", protocol["revision"]));
            }
            for key in ["description", "source"] {
                if let Some(value) = protocol.get(key) {
                    problems.extend(nonempty(value, &format!("$.protocol.{key}")));
                }
            }
        }
        None => problems.push("$.protocol is not an object".to_owned()),
    }
    let kinds = ["action", "evidence", "claim", "outcome", "obligation"];
    for (index, node) in top["nodes"].as_array().into_iter().flatten().enumerate() {
        let at = format!("$.nodes[{index}]");
        let Some(node) = node.as_object() else {
            problems.push(format!("{at}: not an object"));
            continue;
        };
        problems.extend(closed(
            node,
            &[
                "id",
                "kind",
                "name",
                "description",
                "predicate",
                "effect",
                "capabilities",
                "applicable_when",
            ],
            &["id", "kind", "name"],
            &at,
        ));
        if let Some(value) = node.get("applicable_when") {
            if node.get("kind") != Some(&Value::from("obligation")) {
                problems.push(format!(
                    "{at}.applicable_when: on a node of kind {}, not `obligation`",
                    node.get("kind").unwrap_or(&Value::Null)
                ));
            }
            problems.extend(predicate(value, &format!("{at}.applicable_when")));
        }
        problems.extend(nonempty(&node["id"], &format!("{at}.id")));
        problems.extend(nonempty(&node["name"], &format!("{at}.name")));
        if !kinds.contains(&node["kind"].as_str().unwrap_or("")) {
            problems.push(format!("{at}.kind: {}", node["kind"]));
        }
        for key in ["description", "effect"] {
            if let Some(value) = node.get(key) {
                problems.extend(nonempty(value, &format!("{at}.{key}")));
            }
        }
        if let Some(value) = node.get("predicate") {
            problems.extend(predicate(value, &format!("{at}.predicate")));
        }
        if let Some(capabilities) = node.get("capabilities") {
            match capabilities.as_array() {
                Some(list) => {
                    for (position, capability) in list.iter().enumerate() {
                        problems.extend(nonempty(
                            capability,
                            &format!("{at}.capabilities[{position}]"),
                        ));
                    }
                }
                None => problems.push(format!("{at}.capabilities: not an array")),
            }
        }
    }
    let edge_kinds = ["produces", "establishes", "supports", "requires", "gates"];
    for (index, edge) in top["edges"].as_array().into_iter().flatten().enumerate() {
        let at = format!("$.edges[{index}]");
        let Some(edge) = edge.as_object() else {
            problems.push(format!("{at}: not an object"));
            continue;
        };
        problems.extend(closed(
            edge,
            &["from", "to", "kind", "qualifier"],
            &["from", "to", "kind"],
            &at,
        ));
        problems.extend(nonempty(&edge["from"], &format!("{at}.from")));
        problems.extend(nonempty(&edge["to"], &format!("{at}.to")));
        if !edge_kinds.contains(&edge["kind"].as_str().unwrap_or("")) {
            problems.push(format!("{at}.kind: {}", edge["kind"]));
        }
        if let Some(qualifier) = edge.get("qualifier") {
            problems.extend(nonempty(qualifier, &format!("{at}.qualifier")));
        }
    }
    if top.get("nodes").and_then(Value::as_array).is_none() {
        problems.push("$.nodes is not an array".to_owned());
    }
    if top.get("edges").and_then(Value::as_array).is_none() {
        problems.push("$.edges is not an array".to_owned());
    }
    problems
}

/// Every claim `predicate` tests, through `all`, `any` and `not`.
fn tested(predicate: &Predicate) -> BTreeSet<String> {
    let mut claims = BTreeSet::new();
    predicate.visit(&mut |node| {
        if let Predicate::Claim(test) = node {
            claims.insert(test.claim.as_str().to_owned());
        }
    });
    claims
}

/// Problems with `graph` against what `ir` declares.
fn fidelity_problems(graph: &Value, ir: &Ir) -> Vec<String> {
    let mut problems = Vec::new();
    let mut expected: Vec<String> = Vec::new();
    expected.extend(ir.actions.keys().map(|id| format!("action:{id}")));
    expected.extend(ir.evidence_kinds.keys().map(|id| format!("evidence:{id}")));
    expected.extend(ir.claims.keys().map(|id| format!("claim:{id}")));
    expected.extend(ir.outcomes.keys().map(|id| format!("outcome:{id}")));
    expected.extend(ir.obligations.keys().map(|id| format!("obligation:{id}")));
    let nodes: Vec<&Value> = graph["nodes"].as_array().into_iter().flatten().collect();
    let found: Vec<String> = nodes
        .iter()
        .map(|node| node["id"].as_str().unwrap_or("").to_owned())
        .collect();
    if found != expected {
        problems.push(format!(
            "nodes are {found:?}, the compiled protocol declares {expected:?}"
        ));
    }
    let node = |id: &str| nodes.iter().find(|node| node["id"] == id);
    for (id, action) in &ir.actions {
        let Some(node) = node(&format!("action:{id}")) else {
            continue;
        };
        let effect = action
            .effect
            .as_ref()
            .map(|effect| Value::from(effect.as_str()));
        if node.get("effect").cloned() != effect {
            problems.push(format!(
                "action `{id}`: effect {:?}, compiled {effect:?}",
                node.get("effect")
            ));
        }
        let mut required: Vec<String> = action
            .requires
            .iter()
            .map(|c| c.as_str().to_owned())
            .collect();
        required.sort();
        required.dedup();
        let mut drawn: Vec<String> = node
            .get("capabilities")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|c| c.as_str().map(str::to_owned))
            .collect();
        drawn.sort();
        if drawn != required {
            problems.push(format!(
                "action `{id}`: capabilities {drawn:?}, compiled requires {required:?}"
            ));
        }
    }
    let edges: Vec<(String, String, String)> = graph["edges"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|edge| {
            let text = |key: &str| edge[key].as_str().unwrap_or("").to_owned();
            (text("from"), text("to"), text("kind"))
        })
        .collect();
    let has = |from: &str, to: &str, kind: &str| {
        edges
            .iter()
            .any(|(f, t, k)| f == from && t == to && k == kind)
    };
    for (id, obligation) in &ir.obligations {
        for claim in tested(&obligation.discharged_when) {
            if !has(
                &format!("claim:{claim}"),
                &format!("obligation:{id}"),
                "requires",
            ) {
                problems.push(format!(
                    "obligation `{id}`: no `requires` edge from claim `{claim}`"
                ));
            }
        }
    }
    for (id, action) in &ir.actions {
        for claim in tested(&action.precondition) {
            if !has(&format!("claim:{claim}"), &format!("action:{id}"), "gates") {
                problems.push(format!(
                    "action `{id}`: no `gates` edge from claim `{claim}`"
                ));
            }
        }
    }

    // Case inputs: exactly the compiled ones, each with its compiled values; no `case` without.
    let declared: Vec<(String, Vec<String>)> = ir
        .inputs
        .iter()
        .map(|(id, input)| {
            let values = input.values.iter().map(|v| v.as_str().to_owned()).collect();
            (id.as_str().to_owned(), values)
        })
        .collect();
    match graph.get("case") {
        None if declared.is_empty() => {}
        None => problems.push(format!(
            "no `case`, the compiled protocol declares inputs {declared:?}"
        )),
        Some(case) => {
            let drawn: Vec<(String, Vec<String>)> = case["inputs"]
                .as_object()
                .into_iter()
                .flatten()
                .map(|(id, input)| {
                    let values = input["values"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect();
                    (id.clone(), values)
                })
                .collect();
            if drawn != declared {
                problems.push(format!(
                    "case inputs are {drawn:?}, the compiled protocol declares {declared:?}"
                ));
            }
        }
    }
    for (id, obligation) in &ir.obligations {
        let Some(node) = node(&format!("obligation:{id}")) else {
            continue;
        };
        if node.get("applicable_when").is_some() != obligation.applicable_when.is_some() {
            problems.push(format!(
                "obligation `{id}`: applicable_when drawn {}, compiled {}",
                node.get("applicable_when").is_some(),
                obligation.applicable_when.is_some()
            ));
        }
    }
    let mut leaves = Vec::new();
    for node in &nodes {
        for key in ["predicate", "applicable_when"] {
            if let Some(value) = node.get(key) {
                input_leaves(value, &mut leaves);
            }
        }
    }
    for (input, values) in leaves {
        match declared.iter().find(|(id, _)| *id == input) {
            None => problems.push(format!("an input leaf tests undeclared input `{input}`")),
            Some((_, allowed)) => {
                for value in values {
                    if !allowed.contains(&value) {
                        problems.push(format!(
                            "an input leaf tests input `{input}` for undeclared value `{value}`"
                        ));
                    }
                }
            }
        }
    }
    let mut reads = Vec::new();
    for node in &nodes {
        for key in ["predicate", "applicable_when"] {
            if let Some(value) = node.get(key) {
                order_reads(value, &mut reads);
            }
        }
    }
    for (kind, subject) in reads {
        if !ir.evidence_kinds.keys().any(|id| id.as_str() == kind) {
            problems.push(format!(
                "an order leaf reads undeclared evidence kind `{kind}`"
            ));
        }
        if let Some(subject) = subject
            && !ir.artifacts.keys().any(|id| id.as_str() == subject)
        {
            problems.push(format!(
                "an order leaf is about undeclared artifact `{subject}`"
            ));
        }
    }
    problems
}

/// Every evidence kind a drawn `first`, `before`, `after` or `since` leaf reads, each with the
/// subject it names.
fn order_reads(value: &Value, found: &mut Vec<(String, Option<String>)>) {
    match value {
        Value::Object(object) => {
            let text = |value: &Value| value.as_str().unwrap_or("").to_owned();
            if let Some(first) = object.get("first").and_then(Value::as_object) {
                let subject = first.get("subject").map(text);
                found.push((text(&first["kind"]), subject));
            }
            if let Some(before) = object.get("before").and_then(Value::as_object) {
                let subject = before.get("subject").map(text);
                found.push((text(&before["first"]), subject.clone()));
                found.push((text(&before["then"]), subject));
            }
            if let Some(after) = object.get("after").and_then(Value::as_object) {
                let subject = after.get("subject").map(text);
                found.push((text(&after["first"]), subject.clone()));
                found.push((text(&after["then"]["kind"]), subject));
            }
            if let Some(since) = object.get("since").and_then(Value::as_object) {
                let subject = since.get("subject").map(text);
                found.push((text(&since["last"]), subject.clone()));
                found.push((text(&since["then"]["kind"]), subject));
            }
            for member in object.values() {
                order_reads(member, found);
            }
        }
        Value::Array(members) => {
            for member in members {
                order_reads(member, found);
            }
        }
        _ => {}
    }
}

/// Every `input` leaf of a drawn predicate, as its input and the values it lists.
fn input_leaves(value: &Value, found: &mut Vec<(String, Vec<String>)>) {
    match value {
        Value::Object(object) => {
            if let Some(input) = object.get("input") {
                let values = input["in"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect();
                found.push((input["id"].as_str().unwrap_or("").to_owned(), values));
            }
            for member in object.values() {
                input_leaves(member, found);
            }
        }
        Value::Array(members) => {
            for member in members {
                input_leaves(member, found);
            }
        }
        _ => {}
    }
}

/// The transcription refuses what the 0.9.0 schema refuses: `applicable_when` on a node that is
/// not an obligation, a malformed `case`, and a malformed `input` leaf; and accepts each shape the
/// schema adds.
#[test]
fn the_transcription_follows_the_0_9_0_schema() {
    let valid = serde_json::json!({
        "format": "b10x-protocol-graph/1",
        "case": {"inputs": {
            "risk": {"description": null, "values": ["high", "low"]},
            "size": {"description": "how big", "values": ["big"]},
            "bare": {"values": ["one"]},
        }},
        "protocol": {"id": "p", "revision": 1},
        "nodes": [
            {"id": "action:go", "kind": "action", "name": "go",
             "predicate": {"any": [{"claim": {"id": "a"}}, {"input": {"id": "risk", "in": ["low"]}}]}},
            {"id": "obligation:review", "kind": "obligation", "name": "review",
             "predicate": {"claim": {"id": "a"}},
             "applicable_when": {"not": {"input": {"id": "risk", "in": ["low"]}}}},
        ],
        "edges": [],
    });
    assert_eq!(schema_problems(&valid), Vec::<String>::new());

    let broken = |edit: &dyn Fn(&mut Value)| {
        let mut graph = valid.clone();
        edit(&mut graph);
        schema_problems(&graph)
    };
    let applicable_on_action = broken(&|graph| {
        graph["nodes"][0]["applicable_when"] = serde_json::json!({"claim": {"id": "a"}});
    });
    assert_eq!(
        applicable_on_action,
        ["$.nodes[0].applicable_when: on a node of kind \"action\", not `obligation`"]
    );
    for (what, edit) in [
        (
            "an unknown case key",
            &(|graph: &mut Value| graph["case"]["extra"] = Value::from(1)) as &dyn Fn(&mut Value),
        ),
        ("a case without inputs", &|graph: &mut Value| {
            graph["case"] = serde_json::json!({});
        }),
        ("an input without values", &|graph: &mut Value| {
            graph["case"]["inputs"]["risk"] = serde_json::json!({"description": null});
        }),
        ("an input with no value", &|graph: &mut Value| {
            graph["case"]["inputs"]["risk"]["values"] = serde_json::json!([]);
        }),
        ("an input with a value twice", &|graph: &mut Value| {
            graph["case"]["inputs"]["risk"]["values"] = serde_json::json!(["low", "low"]);
        }),
        ("an empty description", &|graph: &mut Value| {
            graph["case"]["inputs"]["risk"]["description"] = Value::from("");
        }),
        ("an unknown input key", &|graph: &mut Value| {
            graph["case"]["inputs"]["risk"]["default"] = Value::from("low");
        }),
        ("an input leaf without `in`", &|graph: &mut Value| {
            graph["nodes"][0]["predicate"] = serde_json::json!({"input": {"id": "risk"}});
        }),
        ("an input leaf with an empty `in`", &|graph: &mut Value| {
            graph["nodes"][0]["predicate"] = serde_json::json!({"input": {"id": "risk", "in": []}});
        }),
        ("an input leaf with an unknown key", &|graph: &mut Value| {
            graph["nodes"][0]["predicate"] =
                serde_json::json!({"input": {"id": "risk", "in": ["low"], "is": "true"}});
        }),
        (
            "an input leaf written as canon source",
            &|graph: &mut Value| {
                graph["nodes"][0]["predicate"] =
                    serde_json::json!({"input": "risk", "in": ["low"]});
            },
        ),
        ("a malformed applicable_when", &|graph: &mut Value| {
            graph["nodes"][1]["applicable_when"] = serde_json::json!({"when": "always"});
        }),
    ] {
        assert!(!broken(edit).is_empty(), "{what} is not a problem");
    }
}

/// The transcription accepts the order-form leaves docs-system 0.10.0 adds, `first` and
/// `before`, each with or without `subject` and under `all`, `any` and `not`, and refuses what
/// that schema refuses of them: a missing required key, an unknown key, an empty or non-string
/// value, and a `first` without a result; and, as the renderer `parseProtocolGraph` does, a
/// `before` whose `first` and `then` are the same kind.
#[test]
fn the_transcription_follows_the_0_10_0_order_leaves() {
    let valid = serde_json::json!({
        "format": "b10x-protocol-graph/1",
        "protocol": {"id": "p", "revision": 1},
        "nodes": [
            {"id": "claim:a", "kind": "claim", "name": "a",
             "predicate": {"any": [
                 {"first": {"kind": "e", "result": "fail", "subject": "x"}},
                 {"not": {"first": {"kind": "e", "result": "ok"}}},
             ]}},
            {"id": "claim:b", "kind": "claim", "name": "b",
             "predicate": {"all": [
                 {"before": {"first": "e", "then": "f", "subject": "x"}},
                 {"not": {"before": {"first": "f", "then": "e"}}},
             ]}},
        ],
        "edges": [],
    });
    assert_eq!(schema_problems(&valid), Vec::<String>::new());

    let broken = |predicate: Value| {
        let mut graph = valid.clone();
        graph["nodes"][0]["predicate"] = predicate;
        schema_problems(&graph)
    };
    for (what, predicate) in [
        (
            "a first without kind",
            serde_json::json!({"first": {"result": "fail"}}),
        ),
        (
            "a first without result",
            serde_json::json!({"first": {"kind": "e"}}),
        ),
        (
            "a first with a null result",
            serde_json::json!({"first": {"kind": "e", "result": null}}),
        ),
        (
            "a first with an empty kind",
            serde_json::json!({"first": {"kind": "", "result": "fail"}}),
        ),
        (
            "a first with an empty subject",
            serde_json::json!({"first": {"kind": "e", "result": "fail", "subject": ""}}),
        ),
        (
            "a first with an unknown key",
            serde_json::json!({"first": {"kind": "e", "result": "fail", "is": "true"}}),
        ),
        (
            "a first that is not an object",
            serde_json::json!({"first": "e"}),
        ),
        (
            "a before without then",
            serde_json::json!({"before": {"first": "e"}}),
        ),
        (
            "a before without first",
            serde_json::json!({"before": {"then": "f"}}),
        ),
        (
            "a before with an empty then",
            serde_json::json!({"before": {"first": "e", "then": ""}}),
        ),
        (
            "a before with a null subject",
            serde_json::json!({"before": {"first": "e", "then": "f", "subject": null}}),
        ),
        (
            "a before with an unknown key",
            serde_json::json!({"before": {"first": "e", "then": "f", "kind": "e"}}),
        ),
        (
            "a before of a kind before itself",
            serde_json::json!({"before": {"first": "e", "then": "e"}}),
        ),
        (
            "a before written as a canon source list",
            serde_json::json!({"before": ["e", "f"]}),
        ),
    ] {
        assert!(!broken(predicate).is_empty(), "{what} is not a problem");
    }
}

#[test]
fn every_committed_protocol_graph_conforms_to_the_pinned_schema_and_its_protocol() {
    let graphs = root().join("website/data/protocol-graphs");
    let mut checked = 0;
    let mut problems = Vec::new();
    // The example graphs too, when there are any: the directory holds only generated files, so
    // a checkout without a preview protocol has none.
    let examples = std::fs::read_dir(root().join("website/data/example-graphs"))
        .into_iter()
        .flatten();
    for entry in std::fs::read_dir(&graphs)
        .expect("the graph directory reads")
        .chain(examples)
    {
        let path = entry.expect("an entry").path();
        let Some(stem) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".json"))
        else {
            continue;
        };
        let graph: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("reads"))
            .unwrap_or_else(|error| panic!("{stem}: {error}"));
        for problem in schema_problems(&graph) {
            problems.push(format!("{stem}: {problem}"));
        }
        let source = graph["protocol"]["source"]
            .as_str()
            .expect("the graph names its source");
        let text = std::fs::read_to_string(root().join(source)).expect("the source protocol reads");
        let ir = compile(&parse(&text).expect("parses")).expect("compiles");
        for problem in fidelity_problems(&graph, &ir) {
            problems.push(format!("{stem}: {problem}"));
        }
        checked += 1;
    }
    assert!(checked > 0, "no graph was checked");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The transcription accepts the `after` leaf docs-system 0.13.0 adds, with or without `subject`,
/// with one kind twice and under `all` and `not`, and refuses what that schema refuses of it: a
/// missing required key at either level, an unknown key, an empty or non-string value, and a
/// `then` that is not an object.
#[test]
fn the_transcription_follows_the_0_13_0_after_leaf() {
    let valid = serde_json::json!({
        "format": "b10x-protocol-graph/1",
        "protocol": {"id": "p", "revision": 1},
        "nodes": [
            {"id": "claim:a", "kind": "claim", "name": "a",
             "predicate": {"all": [
                 {"after": {"first": "e", "then": {"kind": "f", "result": "fail"}, "subject": "x"}},
                 {"not": {"after": {"first": "f", "then": {"kind": "f", "result": "fail"}}}},
             ]}},
        ],
        "edges": [],
    });
    assert_eq!(schema_problems(&valid), Vec::<String>::new());

    let broken = |predicate: Value| {
        let mut graph = valid.clone();
        graph["nodes"][0]["predicate"] = predicate;
        schema_problems(&graph)
    };
    for (what, predicate) in [
        (
            "an after without first",
            serde_json::json!({"after": {"then": {"kind": "f", "result": "fail"}}}),
        ),
        (
            "an after without then",
            serde_json::json!({"after": {"first": "e"}}),
        ),
        (
            "a then that is a kind",
            serde_json::json!({"after": {"first": "e", "then": "f"}}),
        ),
        (
            "a then without result",
            serde_json::json!({"after": {"first": "e", "then": {"kind": "f"}}}),
        ),
        (
            "a then without kind",
            serde_json::json!({"after": {"first": "e", "then": {"result": "fail"}}}),
        ),
        (
            "a then with a null result",
            serde_json::json!({"after": {"first": "e", "then": {"kind": "f", "result": null}}}),
        ),
        (
            "a then with an unknown key",
            serde_json::json!({"after": {"first": "e", "then": {"kind": "f", "result": "fail", "subject": "x"}}}),
        ),
        (
            "an after with an unknown key",
            serde_json::json!({"after": {"first": "e", "then": {"kind": "f", "result": "fail"}, "result": "fail"}}),
        ),
        (
            "an after with an empty first",
            serde_json::json!({"after": {"first": "", "then": {"kind": "f", "result": "fail"}}}),
        ),
        (
            "an after with an empty subject",
            serde_json::json!({"after": {"first": "e", "then": {"kind": "f", "result": "fail"}, "subject": ""}}),
        ),
        (
            "an after that is not an object",
            serde_json::json!({"after": "e"}),
        ),
    ] {
        assert!(!broken(predicate).is_empty(), "{what} is not a problem");
    }
}

/// The transcription accepts the `since` leaf docs-system 0.14.0 adds, with or without `subject`,
/// with one kind twice and under `all` and `not`, and refuses what that schema refuses of it: a
/// missing required key at either level, an unknown key (an `after`'s `first` among them), an
/// empty or non-string value, and a `then` that is not an object.
#[test]
fn the_transcription_follows_the_0_14_0_since_leaf() {
    let valid = serde_json::json!({
        "format": "b10x-protocol-graph/1",
        "protocol": {"id": "p", "revision": 1},
        "nodes": [
            {"id": "claim:a", "kind": "claim", "name": "a",
             "predicate": {"all": [
                 {"since": {"last": "e", "then": {"kind": "f", "result": "fail"}, "subject": "x"}},
                 {"not": {"since": {"last": "f", "then": {"kind": "f", "result": "fail"}}}},
             ]}},
        ],
        "edges": [],
    });
    assert_eq!(schema_problems(&valid), Vec::<String>::new());

    let broken = |predicate: Value| {
        let mut graph = valid.clone();
        graph["nodes"][0]["predicate"] = predicate;
        schema_problems(&graph)
    };
    for (what, predicate) in [
        (
            "a since without last",
            serde_json::json!({"since": {"then": {"kind": "f", "result": "fail"}}}),
        ),
        (
            "a since with first instead of last",
            serde_json::json!({"since": {"first": "e", "then": {"kind": "f", "result": "fail"}}}),
        ),
        (
            "a since without then",
            serde_json::json!({"since": {"last": "e"}}),
        ),
        (
            "a then that is a kind",
            serde_json::json!({"since": {"last": "e", "then": "f"}}),
        ),
        (
            "a then without result",
            serde_json::json!({"since": {"last": "e", "then": {"kind": "f"}}}),
        ),
        (
            "a then with an unknown key",
            serde_json::json!({"since": {"last": "e", "then": {"kind": "f", "result": "fail", "subject": "x"}}}),
        ),
        (
            "a since with an unknown key",
            serde_json::json!({"since": {"last": "e", "then": {"kind": "f", "result": "fail"}, "result": "fail"}}),
        ),
        (
            "a since with an empty last",
            serde_json::json!({"since": {"last": "", "then": {"kind": "f", "result": "fail"}}}),
        ),
        (
            "a since with an empty subject",
            serde_json::json!({"since": {"last": "e", "then": {"kind": "f", "result": "fail"}, "subject": ""}}),
        ),
        (
            "a since that is not an object",
            serde_json::json!({"since": "e"}),
        ),
        (
            "an after with last instead of first",
            serde_json::json!({"after": {"last": "e", "then": {"kind": "f", "result": "fail"}}}),
        ),
    ] {
        assert!(!broken(predicate).is_empty(), "{what} is not a problem");
    }
}
