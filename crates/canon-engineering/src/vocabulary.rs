//! The engineering vocabulary: the names the engineering protocols use for their terms.
//!
//! The vocabulary is data. Every term `software.change/1` and `incident.response/1` names is
//! declared once in `protocols/vocabulary.yaml`, with its category and its marking; this module is
//! the typed reader of that document and declares no term itself. The released file is compiled
//! into the library, [`Vocabulary::from_yaml`] reads any vocabulary document given as text, and
//! [`Vocabulary::to_yaml`] writes one.
//!
//! This module holds names and what they mean, nothing more: what a claim, a piece of evidence or
//! an obligation *is*, and how one is decided, is Canon's.
//!
//! Spelling: claim and action ids are dotted, `<subject>.<predicate>`; artifact, evidence and
//! outcome ids, the names of the explicit decisions an outcome may require (`decision_name`), the
//! case inputs a protocol declares (`case_input`) and the values they take (`input_value`) are
//! single snake_case tokens of lowercase ASCII letters and underscores. An obligation id is either:
//! one token (`restore_service`) or dotted (`tests.required`). Every term has a meaning.
//! [`Vocabulary::from_yaml`] refuses a document that breaks either rule, the released one
//! included.

use std::collections::BTreeSet;
use std::fmt;
use std::sync::LazyLock;

use b10x_canon::model::{ActionId, ClaimId};
use serde::{Deserialize, Deserializer, Serialize};

/// The format a vocabulary document declares.
pub const FORMAT: &str = "vocabulary/1";

/// The engineering vocabulary this crate releases, as compiled into this library.
///
/// The path leaves the package on purpose: the released data lives at the repository root,
/// beside the protocols, and this crate embeds it rather than holding a copy.
const RELEASED: &str = include_str!("../../../protocols/vocabulary.yaml");

/// The kind of name a term is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    ArtifactKind,
    EvidenceKind,
    ClaimId,
    ActionId,
    ObligationId,
    OutcomeId,
    DecisionName,
    /// A case input: configuration a case fixes when it opens (Canon's `case: inputs:`).
    CaseInput,
    /// A value a case input may take.
    InputValue,
}

impl Category {
    /// Every category, in declaration order.
    pub const ALL: [Category; 9] = [
        Category::ArtifactKind,
        Category::EvidenceKind,
        Category::ClaimId,
        Category::ActionId,
        Category::ObligationId,
        Category::OutcomeId,
        Category::DecisionName,
        Category::CaseInput,
        Category::InputValue,
    ];

    /// The category as a vocabulary document spells it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Category::ArtifactKind => "artifact_kind",
            Category::EvidenceKind => "evidence_kind",
            Category::ClaimId => "claim_id",
            Category::ActionId => "action_id",
            Category::ObligationId => "obligation_id",
            Category::OutcomeId => "outcome_id",
            Category::DecisionName => "decision_name",
            Category::CaseInput => "case_input",
            Category::InputValue => "input_value",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Where a term belongs: every engineering domain, or only one protocol's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Marking {
    /// Not specific to Git, pull requests or code.
    #[serde(rename = "core")]
    Core,
    /// Only makes sense for Git, pull requests or code.
    #[serde(rename = "software.change")]
    SoftwareChange,
}

impl Marking {
    pub const fn as_str(self) -> &'static str {
        match self {
            Marking::Core => "core",
            Marking::SoftwareChange => "software.change",
        }
    }
}

impl fmt::Display for Marking {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One vocabulary entry, borrowed from the vocabulary that declares it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Term<'a> {
    pub id: &'a str,
    pub category: Category,
    pub marking: Marking,
    pub meaning: &'a str,
}

impl Term<'_> {
    /// The Canon claim id this term names, if it is a claim id.
    pub fn claim_id(&self) -> Option<ClaimId> {
        (self.category == Category::ClaimId).then(|| ClaimId::new(self.id))
    }

    /// The Canon action id this term names, if it is an action id.
    pub fn action_id(&self) -> Option<ActionId> {
        (self.category == Category::ActionId).then(|| ActionId::new(self.id))
    }
}

/// A lookup of a name no vocabulary entry declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownTerm {
    pub term: String,
}

impl fmt::Display for UnknownTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{}` is not a term of the engineering vocabulary",
            self.term
        )
    }
}

impl std::error::Error for UnknownTerm {}

/// A vocabulary document that cannot be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VocabularyError {
    /// The text is not a vocabulary document: not YAML, a missing or unknown field, or an
    /// unknown category or marking.
    Parse(String),
    /// The document declares a format other than [`FORMAT`].
    Format(String),
    /// The document declares one id more than once.
    DuplicateTerm(String),
    /// An id is not spelled as its category requires.
    Spelling { id: String, category: Category },
    /// A term has an empty or null meaning.
    NoMeaning(String),
}

impl fmt::Display for VocabularyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VocabularyError::Parse(e) => write!(f, "not a vocabulary document: {e}"),
            VocabularyError::Format(format) => {
                write!(f, "vocabulary format `{format}` is not `{FORMAT}`")
            }
            VocabularyError::DuplicateTerm(id) => write!(f, "`{id}` is declared more than once"),
            VocabularyError::Spelling { id, category } => {
                let rule = match spelling(*category) {
                    Spelling::Dotted => "`<subject>.<predicate>` of two snake_case tokens",
                    Spelling::Token => "one snake_case token",
                    Spelling::Either => {
                        "one snake_case token or `<subject>.<predicate>` of two snake_case tokens"
                    }
                };
                write!(f, "`{id}` is a {category}, which is spelled as {rule}")
            }
            VocabularyError::NoMeaning(id) => write!(f, "`{id}` has no meaning"),
        }
    }
}

impl std::error::Error for VocabularyError {}

/// A vocabulary: read from a document, or built in Rust and written as one.
///
/// A vocabulary built with [`Vocabulary::with_term`] is not checked when it is built; it is
/// checked, like every other document, when [`Vocabulary::from_yaml`] reads what
/// [`Vocabulary::to_yaml`] wrote.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Vocabulary {
    terms: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    format: String,
    terms: Vec<Entry>,
}

#[derive(Serialize)]
struct DocumentRef<'a> {
    format: &'a str,
    terms: &'a [Entry],
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: String,
    category: Category,
    marking: Marking,
    #[serde(deserialize_with = "null_as_empty")]
    meaning: String,
}

/// A null meaning reads as an empty one, which [`check`] refuses.
fn null_as_empty<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(Option::<String>::deserialize(d)?.unwrap_or_default())
}

impl Vocabulary {
    /// Reads a vocabulary document given as YAML text, and refuses one that breaks the spelling
    /// rules, leaves a meaning empty or declares an id twice.
    pub fn from_yaml(text: &str) -> Result<Self, VocabularyError> {
        let document: Document =
            serde_yaml_ng::from_str(text).map_err(|e| VocabularyError::Parse(e.to_string()))?;
        if document.format != FORMAT {
            return Err(VocabularyError::Format(document.format));
        }
        check(&document.terms)?;
        Ok(Vocabulary {
            terms: document.terms,
        })
    }

    /// Writes this vocabulary as a document [`Vocabulary::from_yaml`] reads.
    pub fn to_yaml(&self) -> String {
        let document = DocumentRef {
            format: FORMAT,
            terms: &self.terms,
        };
        serde_yaml_ng::to_string(&document)
            .expect("a vocabulary of strings and unit enums always serializes")
    }

    /// This vocabulary with one more term after the others.
    pub fn with_term(
        mut self,
        id: &str,
        category: Category,
        marking: Marking,
        meaning: &str,
    ) -> Self {
        self.terms.push(Entry {
            id: id.to_owned(),
            category,
            marking,
            meaning: meaning.to_owned(),
        });
        self
    }

    /// Every declared term, in document order.
    pub fn terms(&self) -> Vec<Term<'_>> {
        self.terms.iter().map(Entry::term).collect()
    }

    /// The entry `id` names, or a refusal naming `id`.
    pub fn lookup(&self, id: &str) -> Result<Term<'_>, UnknownTerm> {
        self.terms
            .iter()
            .find(|e| e.id == id)
            .map(Entry::term)
            .ok_or_else(|| UnknownTerm {
                term: id.to_owned(),
            })
    }
}

impl Entry {
    fn term(&self) -> Term<'_> {
        Term {
            id: &self.id,
            category: self.category,
            marking: self.marking,
            meaning: &self.meaning,
        }
    }
}

fn check(entries: &[Entry]) -> Result<(), VocabularyError> {
    let mut seen = BTreeSet::new();
    for entry in entries {
        let spelled = match spelling(entry.category) {
            Spelling::Dotted => is_dotted(&entry.id),
            Spelling::Token => is_token(&entry.id),
            Spelling::Either => is_token(&entry.id) || is_dotted(&entry.id),
        };
        if !spelled {
            return Err(VocabularyError::Spelling {
                id: entry.id.clone(),
                category: entry.category,
            });
        }
        if entry.meaning.trim().is_empty() {
            return Err(VocabularyError::NoMeaning(entry.id.clone()));
        }
        if !seen.insert(entry.id.as_str()) {
            return Err(VocabularyError::DuplicateTerm(entry.id.clone()));
        }
    }
    Ok(())
}

/// How a category's ids are spelled.
enum Spelling {
    /// One snake_case token.
    Token,
    /// `<subject>.<predicate>`.
    Dotted,
    /// Either of the two: an obligation id, which `incident.response/1` spells as one token
    /// (`restore_service`) and `software.change/1` dotted (`tests.required`).
    Either,
}

fn spelling(category: Category) -> Spelling {
    match category {
        Category::ClaimId | Category::ActionId => Spelling::Dotted,
        Category::ObligationId => Spelling::Either,
        Category::ArtifactKind
        | Category::EvidenceKind
        | Category::OutcomeId
        | Category::DecisionName
        | Category::CaseInput
        | Category::InputValue => Spelling::Token,
    }
}

/// One snake_case token: lowercase ASCII letters and inner underscores.
fn is_token(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('_')
        && !s.ends_with('_')
        && s.chars().all(|c| c.is_ascii_lowercase() || c == '_')
}

/// `<subject>.<predicate>`, each one token.
fn is_dotted(s: &str) -> bool {
    matches!(s.split_once('.'), Some((a, b)) if is_token(a) && is_token(b))
}

static VOCABULARY: LazyLock<Vocabulary> = LazyLock::new(|| {
    Vocabulary::from_yaml(RELEASED)
        .unwrap_or_else(|e| panic!("the released protocols/vocabulary.yaml: {e}"))
});

static TERMS: LazyLock<Vec<Term<'static>>> = LazyLock::new(|| VOCABULARY.terms());

/// Every term of the released vocabulary, in file order.
pub fn terms() -> &'static [Term<'static>] {
    &TERMS
}

/// The released vocabulary's entry `id` names, or a refusal naming `id`.
pub fn lookup(id: &str) -> Result<&'static Term<'static>, UnknownTerm> {
    TERMS
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| UnknownTerm {
            term: id.to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: &str = "format: vocabulary/1
terms:
  - id: alpha
    category: evidence_kind
    marking: core
    meaning: a first entry
";

    /// `as_str` and `Display` are the spelling a document uses, for every category and marking,
    /// and a spelling refusal prints it rather than the Rust name.
    #[test]
    fn categories_and_markings_print_as_documents_spell_them() {
        for category in Category::ALL {
            let written = serde_yaml_ng::to_string(&category).unwrap();
            assert_eq!(written.trim_end(), category.as_str());
            assert_eq!(category.to_string(), category.as_str());
            let refusal = VocabularyError::Spelling {
                id: "x y".to_owned(),
                category,
            }
            .to_string();
            assert!(refusal.contains(category.as_str()), "{refusal}");
            assert!(!refusal.contains(&format!("{category:?}")), "{refusal}");
        }
        for marking in [Marking::Core, Marking::SoftwareChange] {
            let written = serde_yaml_ng::to_string(&marking).unwrap();
            assert_eq!(written.trim_end(), marking.as_str());
        }
    }

    /// A decision name is one snake_case token, as an outcome id is: dotted or otherwise
    /// misspelled, it is refused naming the category.
    #[test]
    fn a_decision_name_is_one_token() {
        let decision = ONE.replace("evidence_kind", "decision_name");
        let read = Vocabulary::from_yaml(&decision).unwrap();
        assert_eq!(
            read.lookup("alpha").map(|t| t.category),
            Ok(Category::DecisionName)
        );
        for id in [
            "explicitly.declined",
            "Explicitly_declined",
            "_declined",
            "declined2",
        ] {
            assert_eq!(
                Vocabulary::from_yaml(&decision.replace("id: alpha", &format!("id: {id}"))),
                Err(VocabularyError::Spelling {
                    id: id.to_owned(),
                    category: Category::DecisionName
                }),
                "`{id}`"
            );
        }
    }

    /// A case input and an input value are one snake_case token each: dotted or otherwise
    /// misspelled, they are refused naming the category.
    #[test]
    fn a_case_input_and_an_input_value_are_one_token() {
        for category in [Category::CaseInput, Category::InputValue] {
            let document = ONE.replace("evidence_kind", category.as_str());
            let read = Vocabulary::from_yaml(&document).unwrap();
            assert_eq!(read.lookup("alpha").map(|t| t.category), Ok(category));
            for id in ["change.kind", "Risk", "_risk", "risk2", "change-kind"] {
                assert_eq!(
                    Vocabulary::from_yaml(&document.replace("id: alpha", &format!("id: {id}"))),
                    Err(VocabularyError::Spelling {
                        id: id.to_owned(),
                        category
                    }),
                    "`{id}` as {category}"
                );
            }
        }
    }

    /// An obligation id is one token or dotted, and nothing else; its refusal names both forms.
    #[test]
    fn an_obligation_id_is_one_token_or_dotted() {
        let document = ONE.replace("evidence_kind", "obligation_id");
        for id in ["settle_case", "case.settled"] {
            let read = Vocabulary::from_yaml(&document.replace("id: alpha", &format!("id: {id}")))
                .unwrap_or_else(|error| panic!("`{id}`: {error}"));
            assert_eq!(
                read.lookup(id).map(|t| t.category),
                Ok(Category::ObligationId)
            );
        }
        for id in [
            "case.settled.now",
            "case.",
            ".settled",
            "Case.settled",
            "case._settled",
            "settle-case",
        ] {
            let refused =
                Vocabulary::from_yaml(&document.replace("id: alpha", &format!("id: {id}")));
            assert_eq!(
                refused,
                Err(VocabularyError::Spelling {
                    id: id.to_owned(),
                    category: Category::ObligationId
                }),
                "`{id}`"
            );
            let message = refused.unwrap_err().to_string();
            assert!(
                message.contains("one snake_case token or `<subject>.<predicate>`"),
                "{message}"
            );
        }
    }

    #[test]
    fn reads_a_document_and_refuses_what_is_not_one() {
        let read = Vocabulary::from_yaml(ONE).unwrap();
        assert_eq!(read.lookup("alpha").map(|t| t.meaning), Ok("a first entry"));

        let other_format = ONE.replace("vocabulary/1", "vocabulary/2");
        assert_eq!(
            Vocabulary::from_yaml(&other_format),
            Err(VocabularyError::Format("vocabulary/2".to_owned()))
        );

        let twice = format!("{ONE}{}", &ONE[ONE.find("  - id").unwrap()..]);
        assert_eq!(
            Vocabulary::from_yaml(&twice),
            Err(VocabularyError::DuplicateTerm("alpha".to_owned()))
        );

        assert_eq!(
            Vocabulary::from_yaml(&ONE.replace("evidence_kind", "claim_id")),
            Err(VocabularyError::Spelling {
                id: "alpha".to_owned(),
                category: Category::ClaimId
            })
        );
        assert_eq!(
            Vocabulary::from_yaml(&ONE.replace("a first entry", "' '")),
            Err(VocabularyError::NoMeaning("alpha".to_owned()))
        );

        for broken in [
            ONE.replace("meaning:", "meanng:"),
            ONE.replace("evidence_kind", "evidence"),
            ONE.replace("marking: core", "marking: incident.response"),
            ONE.replace("format: vocabulary/1\n", ""),
            format!("{ONE}extra: true\n"),
            format!("{ONE}    note: an undeclared field\n"),
        ] {
            assert!(
                matches!(
                    Vocabulary::from_yaml(&broken),
                    Err(VocabularyError::Parse(_))
                ),
                "read: {broken}"
            );
        }
    }
}
