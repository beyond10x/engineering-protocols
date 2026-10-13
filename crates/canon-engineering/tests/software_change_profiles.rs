//! Acceptance for `story:software-change-profiles`: `software.change/1` declares the case inputs
//! `risk` and `change_kind`. A risk profile is the value of `risk`, and changes only which of the
//! obligations `tests.required`, `review.required`, `provenance.required` and `health.required`
//! apply; a change-kind profile is the value of `change_kind`, and a `bugfix` may edit the
//! implementation only once `regression.reproduced`, and carries one obligation more,
//! `red_first.required`, at every risk (`story:evidence-order-red-first`). Every profile is one
//! compiled protocol evaluated against case data, through the fixture harness and Canon.
//!
//! The test checks every item of the story's § Acceptance and reports each item that fails, in
//! item order, rather than stopping at the first.

mod support;

use std::collections::{BTreeMap, BTreeSet};

use b10x_canon::model::{Decision, Json};
use canon_engineering::vocabulary;
use serde_yaml_ng::Value;
use support::{Compiled, Fixture};

/// The risks, lowest first.
const RISKS: [&str; 4] = ["trivial", "standard", "elevated", "critical"];

/// The obligations, in the order the risks add them.
const OBLIGATIONS: [&str; 4] = [
    "tests.required",
    "review.required",
    "provenance.required",
    "health.required",
];

/// The obligation the change kind selects: a bugfix's, at every risk
/// (`story:evidence-order-red-first`).
const RED_FIRST: &str = "red_first.required";

/// The obligation a change on a security boundary carries, at every risk
/// (`story:security-independence-rules`); the fixtures of this story are off a security boundary.
const SECURITY_REVIEW: &str = "security_review.required";

/// The change kinds, the one that carries [`RED_FIRST`] last.
const CHANGE_KINDS: [&str; 3] = ["feature", "refactor", "bugfix"];

/// The evaluation instant both fixtures of this story give.
const AT: &str = "2026-10-13T12:00:00Z";

/// The terms item 4 names, each with the category it is declared in.
const RISK_TERMS: [(&str, &str); 9] = [
    ("risk", "case_input"),
    ("trivial", "input_value"),
    ("standard", "input_value"),
    ("elevated", "input_value"),
    ("critical", "input_value"),
    ("tests.required", "obligation_id"),
    ("review.required", "obligation_id"),
    ("provenance.required", "obligation_id"),
    ("health.required", "obligation_id"),
];

/// The terms item 8 names, each with the category it is declared in.
const CHANGE_KIND_TERMS: [(&str, &str); 7] = [
    ("change_kind", "case_input"),
    ("feature", "input_value"),
    ("bugfix", "input_value"),
    ("refactor", "input_value"),
    ("tests.write", "action_id"),
    ("implementation.edit", "action_id"),
    ("regression.reproduced", "claim_id"),
];

/// The failures found so far, each prefixed with its acceptance item.
#[derive(Default)]
struct Failures(Vec<String>);

impl Failures {
    fn that(&mut self, item: u8, holds: bool, what: impl FnOnce() -> String) {
        if !holds {
            self.0.push(format!("item {item}: {}", what()));
        }
    }

    fn equal<T: PartialEq + std::fmt::Debug>(
        &mut self,
        item: u8,
        found: T,
        expected: T,
        what: &str,
    ) {
        if found != expected {
            self.0.push(format!(
                "item {item}: {what}: found {found:?}, expected {expected:?}"
            ));
        }
    }
}

/// The fixture `fixtures/software-change/<id>.fixture.yaml`, loaded, with its text read as YAML.
fn fixture(id: &str) -> (Fixture, Value) {
    let path = format!("fixtures/software-change/{id}.fixture.yaml");
    let fixture = Fixture::load(&path).unwrap_or_else(|error| panic!("{path} loads: {error}"));
    assert_eq!(fixture.at(), AT, "{path}: the evaluation instant");
    let text = std::fs::read_to_string(support::repo_root().join(&path))
        .unwrap_or_else(|error| panic!("{path}: {error}"));
    let document = serde_yaml_ng::from_str(&text).unwrap_or_else(|error| panic!("{path}: {error}"));
    (fixture, document)
}

/// The value state `state` of `document` gives `input` through `set_inputs`, or none.
fn set_input(document: &Value, state: &str, input: &str) -> Option<String> {
    document["states"]
        .as_sequence()
        .into_iter()
        .flatten()
        .find(|candidate| candidate["id"].as_str() == Some(state))
        .and_then(|found| found.get("set_inputs"))
        .and_then(|inputs| inputs.get(input))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

/// Canon's decision for state `state` of `fixture`, or the refusal, as text.
fn decide(fixture: &Fixture, compiled: &Compiled, state: &str) -> Result<Decision, String> {
    fixture
        .evaluate(compiled, state)
        .map_err(|refusal| format!("state `{state}` is refused: {}: {refusal}", refusal.code()))
}

/// The status of every obligation in `decision`, keyed by obligation id.
fn obligations(decision: &Decision) -> BTreeMap<String, String> {
    decision
        .obligations
        .as_ref()
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .map(|entry| {
            (
                entry["id"].as_str().unwrap_or_default().to_owned(),
                entry["status"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect()
}

/// The obligations `decision` reports with `status`.
fn with_status(decision: &Decision, status: &str) -> BTreeSet<String> {
    obligations(decision)
        .into_iter()
        .filter(|(_, found)| found == status)
        .map(|(id, _)| id)
        .collect()
}

/// The entry Canon's decision gives action `id`, or `null`.
fn action(decision: &Decision, id: &str) -> Json {
    decision
        .actions
        .as_ref()
        .and_then(|section| section.get(id))
        .cloned()
        .unwrap_or(Json::Null)
}

/// The status Canon's decision gives action `id`, or `null` when it has none.
fn status(decision: &Decision, id: &str) -> String {
    action(decision, id)["status"]
        .as_str()
        .unwrap_or("null")
        .to_owned()
}

/// `value` with every array sorted by its members' JSON text: order aside, the same document.
fn normalized(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(members) => {
            let mut members: Vec<serde_json::Value> = members.iter().map(normalized).collect();
            members.sort_by_key(serde_json::Value::to_string);
            serde_json::Value::Array(members)
        }
        serde_json::Value::Object(fields) => serde_json::Value::Object(
            fields
                .iter()
                .map(|(key, field)| (key.clone(), normalized(field)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Item 4 and item 8: each of `terms` resolves to exactly one vocabulary entry, in its category.
fn resolve(failures: &mut Failures, item: u8, terms: &[(&str, &str)]) {
    for (term, category) in terms {
        match vocabulary::lookup(term) {
            Ok(found) => failures.equal(
                item,
                found.category.as_str(),
                category,
                &format!("the category of `{term}`"),
            ),
            Err(error) => failures.that(item, false, || error.to_string()),
        }
        let entries = vocabulary::terms()
            .iter()
            .filter(|entry| entry.id == *term)
            .count();
        failures.equal(item, entries, 1, &format!("entries declaring `{term}`"));
    }
}

#[test]
fn profiles_nest_open_obligations() {
    let mut failures = Failures::default();
    let path = support::protocol_path("software-change", 1);
    let compiled = support::compile_protocol(&path)
        .unwrap_or_else(|error| panic!("{path} validates and compiles: {error}"));

    // 1. One `canon-ir/1` document, compiled once, serves every profile: both fixtures evaluate
    //    `software.change/1` and compile to exactly it, and no built-in protocol is a profile's own.
    let (profiles, profiles_document) = fixture("profiles");
    let (kinds, kinds_document) = fixture("change-kinds");
    for loaded in [&profiles, &kinds] {
        failures.equal(
            1,
            loaded.protocol_path(),
            path.as_str(),
            "the fixture's protocol",
        );
        match loaded.compile() {
            Ok(own) => failures.that(1, own.canon_ir() == compiled.canon_ir(), || {
                format!("{} compiles to another canon-ir/1 document", loaded.id())
            }),
            Err(error) => failures.that(1, false, || format!("{}: {error}", loaded.id())),
        }
    }
    let builtins: Vec<(&str, u32)> = canon_engineering::registry::list()
        .into_iter()
        .filter(|(name, _)| name.starts_with("software-change"))
        .collect();
    failures.equal(
        1,
        builtins,
        vec![("software-change", 1)],
        "the software.change built-ins",
    );
    failures.equal(
        1,
        profiles.states(),
        RISKS.to_vec(),
        "the states of `profiles`",
    );
    for risk in RISKS {
        failures.equal(
            1,
            set_input(&profiles_document, risk, "risk").as_deref(),
            Some(risk),
            &format!("the risk state `{risk}` sets"),
        );
    }

    // 2 and 3. The open obligations at each risk are exactly the first one, two, three and four
    //    of OBLIGATIONS, each strictly containing the set below it, and every other obligation is
    //    reported `not_applicable`: the rest of OBLIGATIONS, RED_FIRST, since `profiles` is a
    //    refactor, and SECURITY_REVIEW, since it is off a security boundary.
    if let Err(differences) = profiles.check(&compiled) {
        failures.that(2, false, || {
            format!(
                "`profiles` differs from its expectations:\n{}",
                differences.join("\n")
            )
        });
    }
    let mut below: Option<BTreeSet<String>> = None;
    for (index, risk) in RISKS.into_iter().enumerate() {
        let decision = match decide(&profiles, &compiled, risk) {
            Ok(decision) => decision,
            Err(refusal) => {
                failures.that(2, false, || refusal);
                continue;
            }
        };
        let open = with_status(&decision, "open");
        let expected: BTreeSet<String> = OBLIGATIONS[..=index]
            .iter()
            .map(|id| (*id).to_owned())
            .collect();
        failures.equal(
            2,
            &open,
            &expected,
            &format!("the open obligations at `{risk}`"),
        );
        if let Some(below) = &below {
            failures.that(3, below.is_subset(&open) && below != &open, || {
                format!("{open:?} at `{risk}` does not strictly contain {below:?}")
            });
        }
        let rest: BTreeSet<String> = OBLIGATIONS[index + 1..]
            .iter()
            .chain([&RED_FIRST, &SECURITY_REVIEW])
            .map(|id| (*id).to_owned())
            .collect();
        failures.equal(
            3,
            with_status(&decision, "not_applicable"),
            rest,
            &format!("the obligations not applicable at `{risk}`"),
        );
        failures.equal(
            3,
            obligations(&decision).len(),
            OBLIGATIONS.len() + 2,
            &format!("the obligations reported at `{risk}`"),
        );
        below = Some(open);
    }

    // 3, for each change kind. The same risks with `change_kind` set to each kind in turn: each
    //    risk's open obligations still strictly contain those of the risk below, and a bugfix's
    //    are a feature's or a refactor's with RED_FIRST added, at every risk.
    let profiles_text = std::fs::read_to_string(
        support::repo_root().join("fixtures/software-change/profiles.fixture.yaml"),
    )
    .expect("profiles reads");
    failures.equal(
        3,
        profiles_text.matches("    change_kind: refactor\n").count(),
        1,
        "the change kind `profiles` gives its case",
    );
    for kind in CHANGE_KINDS {
        let text = profiles_text.replacen(
            "    change_kind: refactor\n",
            &format!("    change_kind: {kind}\n"),
            1,
        );
        let kinded = match Fixture::from_yaml(&text) {
            Ok(kinded) => kinded,
            Err(error) => {
                failures.that(3, false, || format!("`profiles` as a {kind}: {error}"));
                continue;
            }
        };
        let mut below: Option<BTreeSet<String>> = None;
        for (index, risk) in RISKS.into_iter().enumerate() {
            let decision = match decide(&kinded, &compiled, risk) {
                Ok(decision) => decision,
                Err(refusal) => {
                    failures.that(3, false, || format!("a {kind}: {refusal}"));
                    continue;
                }
            };
            let open = with_status(&decision, "open");
            let mut expected: BTreeSet<String> = OBLIGATIONS[..=index]
                .iter()
                .map(|id| (*id).to_owned())
                .collect();
            if kind == "bugfix" {
                expected.insert(RED_FIRST.to_owned());
            }
            failures.equal(
                3,
                &open,
                &expected,
                &format!("the open obligations of a {kind} at `{risk}`"),
            );
            if let Some(below) = &below {
                failures.that(3, below.is_subset(&open) && below != &open, || {
                    format!("{open:?} of a {kind} at `{risk}` does not strictly contain {below:?}")
                });
            }
            below = Some(open);
        }
    }

    // 4. Each term of the risk profiles resolves to exactly one vocabulary entry.
    resolve(&mut failures, 4, &RISK_TERMS);

    // 5. A bugfix with no evidence may write tests, but not edit the implementation; the edit
    //    names the claim `regression.reproduced` and the input `change_kind` at `bugfix`.
    if let Err(differences) = kinds.check(&compiled) {
        for difference in differences {
            let item = if difference.contains("`bugfix`") {
                5
            } else if difference.contains("`bugfix-red") {
                6
            } else {
                7
            };
            failures.that(item, false, || format!("`change-kinds`: {difference}"));
        }
    }
    match decide(&kinds, &compiled, "bugfix") {
        Ok(decision) => {
            failures.equal(
                5,
                status(&decision, "tests.write"),
                "admissible".into(),
                "tests.write",
            );
            failures.equal(
                5,
                status(&decision, "implementation.edit"),
                "blocked".into(),
                "implementation.edit",
            );
            let reasons = action(&decision, "implementation.edit")["reasons"].clone();
            let named = |wanted: Json| {
                reasons
                    .as_array()
                    .is_some_and(|reasons| reasons.contains(&wanted))
            };
            failures.that(
                5,
                named(serde_json::json!({"claim": "regression.reproduced", "value": "unknown"})),
                || format!("implementation.edit's reasons {reasons} name no regression.reproduced"),
            );
            failures.that(
                5,
                named(serde_json::json!({"input": "change_kind", "value": "bugfix"})),
                || format!("implementation.edit's reasons {reasons} name no change_kind bugfix"),
            );
        }
        Err(refusal) => failures.that(5, false, || refusal),
    }

    // 6. One failing test on R1 makes the bugfix's edit of R1 admissible; once the
    //    implementation is at R2 that red no longer counts, and the edit is blocked again.
    failures.equal(
        6,
        set_input(&kinds_document, "bugfix-red", "change_kind").as_deref(),
        Some("bugfix"),
        "the change kind `bugfix-red` sets",
    );
    let failing: Vec<String> = kinds
        .evidence("bugfix-red")
        .iter()
        .map(|record| {
            format!(
                "{} {} {}",
                record.kind.as_str(),
                record.result.as_deref().unwrap_or("-"),
                record.subject.as_str()
            )
        })
        .collect();
    failures.equal(
        6,
        failing,
        vec!["test_result fail implementation".to_owned()],
        "the evidence of `bugfix-red`",
    );
    for (state, expected) in [("bugfix-red", "admissible"), ("bugfix-red-r2", "blocked")] {
        match decide(&kinds, &compiled, state) {
            Ok(decision) => failures.equal(
                6,
                status(&decision, "implementation.edit"),
                expected.into(),
                &format!("implementation.edit at `{state}`"),
            ),
            Err(refusal) => failures.that(6, false, || refusal),
        }
    }

    // 7. A refactor and a feature, with no evidence, may edit the implementation.
    for state in ["refactor", "feature"] {
        failures.that(7, kinds.evidence(state).is_empty(), || {
            format!("`{state}` has evidence")
        });
        match decide(&kinds, &compiled, state) {
            Ok(decision) => failures.equal(
                7,
                status(&decision, "implementation.edit"),
                "admissible".into(),
                &format!("implementation.edit at `{state}`"),
            ),
            Err(refusal) => failures.that(7, false, || refusal),
        }
    }

    // 8. Each term of the change-kind profiles resolves to exactly one vocabulary entry, and the
    //    split `repository.edit` to none.
    resolve(&mut failures, 8, &CHANGE_KIND_TERMS);
    failures.that(8, vocabulary::lookup("repository.edit").is_err(), || {
        "`repository.edit` still resolves".to_owned()
    });
    failures.equal(
        8,
        vocabulary::terms()
            .iter()
            .filter(|entry| entry.id == "repository.edit")
            .count(),
        0,
        "entries declaring `repository.edit`",
    );

    // 9. A case that gives no `risk` is refused as `missing-input`.
    let mut case: Value = profiles_document["case"].clone();
    let removed = case
        .get_mut("inputs")
        .and_then(Value::as_mapping_mut)
        .and_then(|inputs| inputs.remove("risk"));
    failures.that(9, removed.is_some(), || {
        "the profiles case gives no risk".to_owned()
    });
    match b10x_canon::eval::case_from_value(&case) {
        Ok(case) => match b10x_canon::eval::evaluate(compiled.ir(), &case, &[]) {
            Ok(_) => failures.that(9, false, || "a case without `risk` is evaluated".to_owned()),
            Err(refusal) => {
                failures.equal(9, refusal.code(), "missing-input", "the refusal");
                failures.that(9, refusal.to_string().contains("`risk`"), || {
                    format!("the refusal does not name `risk`: {refusal}")
                });
            }
        },
        Err(refusal) => failures.that(9, false, || format!("the case does not read: {refusal}")),
    }

    // 10. The committed graph declares both case inputs with their values, every obligation's
    //     `applicable_when`, and the `input` leaf of `implementation.edit`'s precondition.
    let graph_path =
        support::repo_root().join("website/data/protocol-graphs/software-change-1.json");
    let graph: serde_json::Value = std::fs::read_to_string(&graph_path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(serde_json::Value::Null);
    // Canon compiles input values, `in` lists and `any` members in a canonical order, and the
    // graph is drawn from the compiled protocol, so order is compared as Canon writes it: the
    // values in `canon-ir/1`'s order, the predicates with their lists sorted.
    for (input, values) in [
        ("risk", serde_json::json!(RISKS)),
        (
            "change_kind",
            serde_json::json!(["feature", "bugfix", "refactor"]),
        ),
    ] {
        failures.equal(
            10,
            normalized(&graph["case"]["inputs"][input]["values"]),
            normalized(&values),
            &format!("the graph's values of case input `{input}`"),
        );
        let compiled_values: Vec<&str> = compiled
            .ir()
            .inputs
            .iter()
            .find(|(id, _)| id.as_str() == input)
            .map(|(_, declared)| declared.values.iter().map(|value| value.as_str()).collect())
            .unwrap_or_default();
        failures.equal(
            10,
            &graph["case"]["inputs"][input]["values"],
            &serde_json::json!(compiled_values),
            &format!("the graph's values of case input `{input}` in canon-ir/1's order"),
        );
    }
    let node = |id: &str| {
        graph["nodes"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|node| node["id"] == id)
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    };
    for (index, obligation) in OBLIGATIONS.into_iter().enumerate() {
        failures.equal(
            10,
            normalized(&node(&format!("obligation:{obligation}"))["applicable_when"]),
            normalized(&serde_json::json!({"input": {"id": "risk", "in": RISKS[index..]}})),
            &format!("the graph's applicable_when of `{obligation}`"),
        );
    }
    failures.equal(
        10,
        normalized(&node("action:implementation.edit")["predicate"]),
        normalized(&serde_json::json!({"any": [
            {"input": {"id": "change_kind", "in": ["feature", "refactor"]}},
            {"claim": {"id": "regression.reproduced"}},
        ]})),
        "the graph's precondition of implementation.edit",
    );

    assert!(
        failures.0.is_empty(),
        "{} acceptance failures:\n{}",
        failures.0.len(),
        failures.0.join("\n")
    );
}
