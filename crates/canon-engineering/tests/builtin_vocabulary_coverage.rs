//! Acceptance 2 of `story:takt-protocols-builtin` (Atlas ADR 0094 § Consequences): every term a
//! built-in names is declared in `protocols/vocabulary.yaml`, in the category it is used in.
//!
//! The built-ins are read from `registry::list` at run time, so a protocol file added later is
//! covered without an edit here. `support.triage/1` and `protocol.adopt/1` are left out: they are the
//! two protocols outside engineering, and their terms stay out of the vocabulary (`AGENTS.md`
//! § Boundary).
//!
//! A term is every artifact kind, evidence kind, claim, action, obligation and outcome id, every
//! decision name an outcome requires, every case input and each value it takes, and every
//! independence dimension. Capabilities are not terms: `vocabulary/1` has no capability category.

use std::collections::BTreeMap;

use b10x_canon::ir::Ir;
use canon_engineering::registry;
use canon_engineering::vocabulary::{self, Category, Marking, Vocabulary, VocabularyError};

/// The built-ins whose terms the vocabulary leaves out.
const OUTSIDE_ENGINEERING: [(&str, u32); 2] = [("protocol-adopt", 1), ("support-triage", 1)];

/// The terms a built-in names that the vocabulary's spelling rule refuses, so no entry can
/// declare them: a claim or action id is `<subject>.<predicate>` of two snake_case tokens
/// (`src/vocabulary.rs`), and Canon accepts a one-token claim id and a three-part action id. The
/// protocols are takt's byte for byte (Atlas ADR 0094 rule 3), so they cannot be renamed here.
/// Each entry is asserted as it is today, undeclared and refused by the reader in its category,
/// so the list goes red when the rule or the protocol changes. Closing the gap is a decision on
/// the spelling rule, not this test's.
const NOT_SPELLABLE: &[(&str, &str, Category)] = &[
    (
        "loop-improvement@1",
        "loop.change.deliver",
        Category::ActionId,
    ),
    (
        "loop-improvement@1",
        "loop.definition.check",
        Category::ActionId,
    ),
    (
        "loop-improvement@1",
        "loop.finding.file",
        Category::ActionId,
    ),
    ("self-improvement@1", "promoted", Category::ClaimId),
];

/// Every term `ir` names, with the category it is used in, section by section in id order.
fn terms(ir: &Ir) -> Vec<(String, Category)> {
    let mut found = Vec::new();
    let mut add = |id: &str, category: Category| found.push((id.to_owned(), category));
    for id in ir.artifacts.keys() {
        add(id.as_str(), Category::ArtifactKind);
    }
    for id in ir.evidence_kinds.keys() {
        add(id.as_str(), Category::EvidenceKind);
    }
    for id in ir.claims.keys() {
        add(id.as_str(), Category::ClaimId);
    }
    for id in ir.actions.keys() {
        add(id.as_str(), Category::ActionId);
    }
    for id in ir.obligations.keys() {
        add(id.as_str(), Category::ObligationId);
    }
    for (id, outcome) in &ir.outcomes {
        add(id.as_str(), Category::OutcomeId);
        if let Some(decision) = outcome.requires.decision() {
            add(decision.as_str(), Category::DecisionName);
        }
    }
    for (id, input) in &ir.inputs {
        add(id.as_str(), Category::CaseInput);
        for value in &input.values {
            add(value.as_str(), Category::InputValue);
        }
    }
    for id in ir.independence_dimensions.keys() {
        add(id.as_str(), Category::IndependenceDimension);
    }
    found
}

/// Why `term`, used as `category`, is not covered, or `None` when one vocabulary entry declares
/// it in that category.
fn gap(term: &str, category: Category) -> Option<String> {
    let entries = vocabulary::terms()
        .iter()
        .filter(|entry| entry.id == term)
        .count();
    match vocabulary::lookup(term) {
        Err(_) => Some("is not declared".to_owned()),
        Ok(_) if entries != 1 => Some(format!("is declared {entries} times")),
        Ok(found) if found.category != category => {
            Some(format!("is declared as {}", found.category))
        }
        Ok(_) => None,
    }
}

#[test]
fn every_builtin_term_is_declared_in_the_category_it_is_used_in() {
    let listed = registry::list();
    for outside in OUTSIDE_ENGINEERING {
        assert!(
            listed.contains(&outside),
            "the registry lists {outside:?}: {listed:?}"
        );
    }
    let mut gaps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut checked = 0;
    let mut unspellable = Vec::new();
    for (name, major) in listed {
        if OUTSIDE_ENGINEERING.contains(&(name, major)) {
            continue;
        }
        let builtin = registry::get(name, major)
            .unwrap_or_else(|error| panic!("registry::get({name}, {major}): {error}"));
        let ir = b10x_canon::ir::compile(&builtin.model)
            .unwrap_or_else(|problems| panic!("{name}@{major} compiles: {problems:?}"));
        let label = format!("{name}@{major}");
        for (term, category) in terms(&ir) {
            checked += 1;
            if NOT_SPELLABLE.contains(&(label.as_str(), term.as_str(), category)) {
                unspellable.push((label.clone(), term, category));
                continue;
            }
            if let Some(reason) = gap(&term, category) {
                gaps.entry(label.clone())
                    .or_default()
                    .push(format!("{category} `{term}` {reason}"));
            }
        }
    }
    assert!(checked > 0, "no term of any built-in was checked");

    // Every NOT_SPELLABLE entry is a term a built-in names, in that category, that no entry
    // declares and that the vocabulary reader refuses to declare there.
    let found: Vec<(&str, &str, Category)> = unspellable
        .iter()
        .map(|(label, term, category)| (label.as_str(), term.as_str(), *category))
        .collect();
    assert_eq!(
        found, NOT_SPELLABLE,
        "NOT_SPELLABLE names exactly the unspellable terms the built-ins use, in order"
    );
    for (builtin, term, category) in NOT_SPELLABLE {
        assert!(
            vocabulary::lookup(term).is_err(),
            "{builtin}: `{term}` is declared now; remove it from NOT_SPELLABLE"
        );
        let document = Vocabulary::default()
            .with_term(
                term,
                *category,
                Marking::Core,
                "a term the spelling rule refuses",
            )
            .to_yaml();
        assert_eq!(
            Vocabulary::from_yaml(&document),
            Err(VocabularyError::Spelling {
                id: (*term).to_owned(),
                category: *category
            }),
            "{builtin}: the vocabulary reader refuses {category} `{term}`; when it reads it, \
             declare the term and remove it from NOT_SPELLABLE"
        );
    }
    let report: Vec<String> = gaps
        .iter()
        .flat_map(|(builtin, lines)| lines.iter().map(move |line| format!("{builtin}: {line}")))
        .collect();
    assert!(
        report.is_empty(),
        "{} terms of the built-ins are not covered by protocols/vocabulary.yaml:\n{}",
        report.len(),
        report.join("\n")
    );
}

/// A marking other than `core` and `software.change` names the one protocol whose terms it marks
/// (`self.improvement`, `loop.improvement`): every term it marks is used by that built-in, and by
/// no other.
#[test]
fn a_protocol_marking_marks_only_that_protocols_terms() {
    let mut users: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, major) in registry::list() {
        if OUTSIDE_ENGINEERING.contains(&(name, major)) {
            continue;
        }
        let builtin = registry::get(name, major)
            .unwrap_or_else(|error| panic!("registry::get({name}, {major}): {error}"));
        let ir = b10x_canon::ir::compile(&builtin.model)
            .unwrap_or_else(|problems| panic!("{name}@{major} compiles: {problems:?}"));
        for (term, _) in terms(&ir) {
            let protocol = ir.protocol.id.as_str().to_owned();
            let entry = users.entry(term).or_default();
            if !entry.contains(&protocol) {
                entry.push(protocol);
            }
        }
    }
    for term in vocabulary::terms() {
        let marking = term.marking.as_str();
        if matches!(marking, "core" | "software.change") {
            continue;
        }
        let used_by = users.get(term.id).cloned().unwrap_or_default();
        assert_eq!(
            used_by,
            [marking],
            "`{}` is marked `{marking}` and used by {used_by:?}",
            term.id
        );
    }
}
