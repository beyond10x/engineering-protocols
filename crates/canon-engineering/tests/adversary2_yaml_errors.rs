//! Adversary cases (pass 2) for `story:vocabulary-yaml-source`: what the reader's refusals say,
//! the spelling rule at its edges, and the builder's order.
//!
//! The unit's suite compares refusals as enum values and never reads their text, except one
//! `NoMeaning`; a refusal that stops naming the offending id stays green there.

use canon_engineering::vocabulary::{Category, Marking, Vocabulary, VocabularyError};

fn document(entries: &[(&str, &str, &str, &str)]) -> String {
    let mut text = String::from("format: vocabulary/1\nterms:\n");
    for (id, category, marking, meaning) in entries {
        text.push_str(&format!(
            "  - id: {id}\n    category: {category}\n    marking: {marking}\n    meaning: {meaning}\n"
        ));
    }
    text
}

fn refusal(text: &str) -> VocabularyError {
    Vocabulary::from_yaml(text).expect_err("the document must be refused")
}

/// Every refusal the reader decides itself names what it refuses, in its message.
#[test]
fn adversary2_yaml_refusals_name_the_offending_id() {
    let spelling = refusal(&document(&[
        ("plan", "artifact_kind", "core", "fine"),
        ("planned", "claim_id", "core", "an undotted claim"),
    ]));
    assert_eq!(
        spelling,
        VocabularyError::Spelling {
            id: "planned".to_owned(),
            category: Category::ClaimId
        }
    );
    let text = spelling.to_string();
    assert!(text.contains("`planned`"), "{text}");
    assert!(text.contains("<subject>.<predicate>"), "{text}");

    let token = refusal(&document(&[("a.b", "evidence_kind", "core", "dotted")])).to_string();
    assert!(token.contains("`a.b`"), "{token}");
    assert!(token.contains("one snake_case token"), "{token}");

    let twice = refusal(&document(&[
        ("plan", "artifact_kind", "core", "first"),
        ("intent", "artifact_kind", "core", "between"),
        ("plan", "evidence_kind", "core", "second"),
    ]));
    assert_eq!(twice, VocabularyError::DuplicateTerm("plan".to_owned()));
    let text = twice.to_string();
    assert!(text.contains("`plan`"), "{text}");

    let empty = refusal(&document(&[
        ("plan", "artifact_kind", "core", "fine"),
        ("intent", "artifact_kind", "core", "' '"),
    ]));
    assert_eq!(empty, VocabularyError::NoMeaning("intent".to_owned()));
    assert!(empty.to_string().contains("`intent`"), "{empty}");

    let format = refusal("format: vocabulary/9\nterms: []\n");
    assert_eq!(format, VocabularyError::Format("vocabulary/9".to_owned()));
    assert!(format.to_string().contains("vocabulary/9"), "{format}");
}

/// A document the YAML layer refuses still says where: the entry and the field.
#[test]
fn adversary2_yaml_parse_refusal_locates_the_entry() {
    let text = document(&[
        ("plan", "artifact_kind", "core", "fine"),
        ("intent", "artifact", "core", "an unknown category"),
    ]);
    let refused = refusal(&text);
    assert!(matches!(refused, VocabularyError::Parse(_)), "{refused:?}");
    let message = refused.to_string();
    assert!(
        message.contains("terms[1]") && message.contains("category"),
        "the refusal does not locate the second entry's category: {message}"
    );
}

/// The spelling rule at its edges: one case per way `is_token` or `is_dotted` could be widened.
#[test]
fn adversary2_yaml_refuses_each_edge_of_the_spelling_rule() {
    for (id, category, cat) in [
        ("_leading", "evidence_kind", Category::EvidenceKind),
        ("trailing_", "evidence_kind", Category::EvidenceKind),
        ("with2digit", "evidence_kind", Category::EvidenceKind),
        ("kebab-case", "artifact_kind", Category::ArtifactKind),
        ("café", "outcome_id", Category::OutcomeId),
        ("release.", "claim_id", Category::ClaimId),
        (".proven", "claim_id", Category::ClaimId),
        ("release._proven", "action_id", Category::ActionId),
        ("release.proven_", "action_id", Category::ActionId),
        ("release2.proven", "claim_id", Category::ClaimId),
        ("Release.proven", "claim_id", Category::ClaimId),
        // An obligation id is one token or dotted (story:software-change-profiles); past either
        // form it is refused.
        (
            "restore.service.now",
            "obligation_id",
            Category::ObligationId,
        ),
        ("restore._service", "obligation_id", Category::ObligationId),
        ("Restore.service", "obligation_id", Category::ObligationId),
    ] {
        let text = document(&[(id, category, "core", "an id at the edge of the rule")]);
        assert_eq!(
            Vocabulary::from_yaml(&text),
            Err(VocabularyError::Spelling {
                id: id.to_owned(),
                category: cat
            }),
            "`{id}` as {category}"
        );
    }
    // The rule's inner cases still read.
    let fine = document(&[
        (
            "double__underscore",
            "evidence_kind",
            "core",
            "inner underscores",
        ),
        (
            "release_notes.proven_ok",
            "claim_id",
            "core",
            "inner underscores, dotted",
        ),
    ]);
    assert!(Vocabulary::from_yaml(&fine).is_ok());
}

/// `with_term` adds a term "after the others", and the built vocabulary's `lookup` refuses by
/// name like the released one.
#[test]
fn adversary2_yaml_builder_appends_in_order_and_lookup_refuses_by_name() {
    let built = Vocabulary::default()
        .with_term("first", Category::EvidenceKind, Marking::Core, "one")
        .with_term(
            "second",
            Category::ArtifactKind,
            Marking::SoftwareChange,
            "two",
        )
        .with_term("third.thing", Category::ClaimId, Marking::Core, "three");
    let ids: Vec<&str> = built.terms().iter().map(|t| t.id).collect();
    assert_eq!(ids, ["first", "second", "third.thing"]);
    let read_back = Vocabulary::from_yaml(&built.to_yaml()).unwrap();
    let read: Vec<&str> = read_back.terms().iter().map(|t| t.id).collect();
    assert_eq!(read, ["first", "second", "third.thing"]);

    let missing = built
        .lookup("fourth")
        .expect_err("`fourth` is not declared");
    assert_eq!(missing.term, "fourth");
    assert!(missing.to_string().contains("`fourth`"), "{missing}");
    assert_eq!(
        built
            .lookup("second")
            .map(|t| (t.category, t.marking, t.meaning)),
        Ok((Category::ArtifactKind, Marking::SoftwareChange, "two"))
    );
}
