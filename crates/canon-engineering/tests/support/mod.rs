//! The engineering protocols compile-and-evaluate fixture harness. Every protocol test calls it; a
//! story that needs it changed records that as a scope change on itself first.
//!
//! The harness adds no evaluator of its own and reads no clock, network or model. It validates and
//! compiles a `protocol/1` document with Canon (`b10x_canon::validate::validate` and
//! `b10x_canon::ir::compile`, the checks `canon validate` and `canon compile` run), evaluates a
//! fixture's states with `b10x_canon::eval::evaluate`, and returns Canon's `canon-decision/1`
//! decision, or its refusal, unchanged.
//!
//! # Where protocols and fixtures live
//!
//! - A built-in engineering protocol is a `protocol/1` document at `protocols/<name>/<major>.yaml`
//!   ([`protocol_path`]). Each protocol story creates its own file.
//! - A protocol's fixtures live under `fixtures/<protocol-name>/`, one `<fixture-id>.fixture.yaml`
//!   file per fixture. Each protocol story creates its own directory. A protocol that exists only
//!   to test the harness lives beside its fixtures (`fixtures/smoke/protocol.yaml`).
//!
//! Every path a fixture names is relative to the repository root, without a leading `/` or `\`,
//! a drive prefix or a `..` component, and must not resolve outside the repository through a
//! symbolic link.
//!
//! # The `els-fixture/1` format
//!
//! ```yaml
//! format: els-fixture/1
//! id: smoke                              # the fixture's name; the file is <id>.fixture.yaml
//! protocol: fixtures/smoke/protocol.yaml # the protocol/1 document the fixture evaluates
//! at: "2026-10-04T00:00:00Z"             # the evaluation instant, YYYY-MM-DDTHH:MM:SSZ (UTC)
//! case:                                  # a canon-case/1 document: the case and the current
//!   format: canon-case/1                 # revision of every artifact the protocol declares
//!   id: SMOKE-1
//!   protocol: smoke
//!   artifacts:
//!     service: {revision: r1}
//! states:                                # an ordered, non-empty list
//!   - id: initial                        # unique within the fixture
//!     add_evidence: []                   # evidence added to that of the states before it
//!     expect:
//!       claims:                          # every claim the protocol declares, and its value
//!         service.healthy: unknown       # true, false or unknown
//!   - id: healthy-observed
//!     add_evidence:
//!       - observed_at: "2026-10-03T23:00:00Z"   # when it was observed, not after `at`
//!         record: {format: canon-evidence/1, id: observation-1, kind: operational_observation,
//!                  result: healthy, subject: service, subject_revision: r1}
//!     expect:
//!       claims:
//!         service.healthy: true
//! ```
//!
//! A state may also carry, each optional:
//!
//! ```yaml
//!   - id: rolled-back
//!     set_revisions:                     # the case snapshot's current revision of an artifact
//!       service: r2                      # the case lists, from this state on
//!     add_authority:                     # canon-authority/1 decisions added to those of the
//!       - {capability: release.rollback, decision: granted}   # states before it
//!     add_decisions:                     # canon-decisions/1 explicit decisions added to those of
//!       - {decision: explicitly_abandoned, outcome: abandoned,  # the states before it
//!          principal: change-owner, case_revision: c1}
//!     add_evidence: []
//!     expect:
//!       claims: {service.healthy: true}
//!       obligations:                     # every obligation the protocol declares, and its
//!         restore_service: discharged    # status: open or discharged
//!       actions:                         # every action the protocol declares, and its status:
//!         release.rollback: admissible   # admissible, approval-required or blocked
//!       outcomes:                        # every outcome the protocol declares, and its status:
//!         abandoned: legitimate          # legitimate or blocked
//! ```
//!
//! States are cumulative: a state is evaluated over its own `add_evidence`, `add_authority`,
//! `add_decisions` and `set_revisions` and those of every state before it. A fixture cannot carry
//! an input the harness would drop or read as something else: an unknown key is refused, a key
//! written with no value (`add_evidence:`) is refused as Canon refuses one rather than read as its
//! default, a claim, obligation, action, outcome or artifact written twice in one map is refused,
//! naming it, and
//! `set_revisions` naming an artifact the case does not list is refused. [`Fixture::load`]
//! refuses a file not named `<id>.fixture.yaml` after the id it declares, so two files cannot
//! declare one fixture.
//!
//! The case and every evidence record are read with Canon's own readers
//! (`b10x_canon::eval::case_from_value` and `evidence_from_value`, the path `canon evaluate`
//! takes), so a document Canon refuses does not load: an identifier, revision or result written
//! as a number or a boolean is refused with Canon's `malformed-input` refusal. Each state's case
//! snapshot is the fixture's `case` with the revisions set so far, read by Canon again. The
//! authority decisions so far are passed to Canon as one `canon-authority/1` list, the input
//! `canon evaluate --authority` reads, and Canon reads and refuses them (an entry that cannot be
//! written as YAML at all is refused at load, naming its state and position); a fixture in which
//! no state so far gives `add_authority` passes none. Decisions only accumulate: a later state that
//! decides a capability an earlier state already decided (a denial after a grant, say) makes the
//! list decide it twice, which Canon refuses as `duplicate-identifier` for that state and every
//! later one, so a fixture cannot revoke a grant.
//!
//! Explicit decisions are passed the same way, as one `canon-decisions/1` list of every
//! `add_decisions` entry so far, the input `canon evaluate --decisions` reads, and Canon reads and
//! refuses them: an entry given twice is `duplicate-identifier`, and a decision for an outcome the
//! protocol does not declare, or one that outcome does not require, is refused for that state and
//! every later one. A decision applies only at the case's `revision`, so a fixture whose outcomes
//! wait on decisions gives its case one. A fixture in which no state so far gives `add_decisions`
//! passes none.
//!
//! The evaluation instant is an input of the fixture, never read from a clock. The harness does
//! not pass the instant or an observation time to Canon yet, so nothing expires; observation
//! times reach Canon with story:stale-evidence-fixtures. It refuses, when loading, a fixture
//! holding evidence observed after its instant, instead of evaluating evidence the instant
//! excludes. A record may also write its own `observed_at`, which `canon-evidence/1` reads: it
//! must be an instant, not after the fixture's, and the same as its entry's `observed_at`, or the
//! fixture is refused; the harness then drops it from the record, so the evidence it passes to
//! Canon carries no observation time. Instants are written in the one form above and name a day
//! that exists in its month (leap years counted), so they compare as text.
//!
//! Independence decisions are not part of the format yet: Canon does not evaluate them, and a
//! fixture that gives them (`independence:`) is refused as having an unknown key. They join the
//! format with the Canon capability that reads them.
//!
//! The items only later protocol stories call carry `#[allow(dead_code)]`, each marked as
//! later-story API; everything else is used by this story's tests.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use b10x_canon::eval::{self, Refusal};
use b10x_canon::ir::Ir;
use b10x_canon::model::{Case, Decision, EvidenceRecord, ParseError, Truth};
use b10x_canon::validate::Problem;
use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_yaml_ng::Value;

/// The fixture format this harness reads.
pub const FORMAT: &str = "els-fixture/1";

/// The repository root, found at run time from the package's manifest directory.
pub fn repo_root() -> PathBuf {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
    let root = Path::new(&manifest).join("../..");
    root.canonicalize()
        .unwrap_or_else(|error| panic!("the repository root {}: {error}", root.display()))
}

/// The path of built-in protocol `name` at major version `major`, relative to the repository root.
#[allow(dead_code)] // later-story API: the protocol stories compile protocols/<name>/<major>.yaml
pub fn protocol_path(name: &str, major: u64) -> String {
    format!("protocols/{name}/{major}.yaml")
}

/// Why a fixture did not load.
#[derive(Debug)]
pub enum FixtureError {
    /// The fixture path is absolute, leaves the repository, or cannot be read.
    Unreadable(String),
    /// The text is not an `els-fixture/1` document.
    Malformed(String),
    /// The document is well-formed but cannot be evaluated as written.
    Refused(String),
}

impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixtureError::Unreadable(why) | FixtureError::Refused(why) => f.write_str(why),
            FixtureError::Malformed(why) => write!(f, "not an {FORMAT} document: {why}"),
        }
    }
}

impl std::error::Error for FixtureError {}

/// Why a protocol did not compile.
#[derive(Debug)]
pub enum CompileError {
    /// The protocol path is absolute, leaves the repository, or cannot be read.
    Unreadable(String),
    /// The text is not a `protocol/1` document.
    Parse(ParseError),
    /// Canon's validator rejects the document; its problems, unchanged.
    Invalid(Vec<Problem>),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Unreadable(why) => f.write_str(why),
            CompileError::Parse(error) => write!(f, "{error}"),
            CompileError::Invalid(problems) => {
                let lines: Vec<String> = problems.iter().map(ToString::to_string).collect();
                write!(f, "the protocol is invalid: {}", lines.join("; "))
            }
        }
    }
}

impl std::error::Error for CompileError {}

/// A protocol Canon validated and compiled.
#[derive(Debug, Clone)]
pub struct Compiled {
    ir: Ir,
    canon_ir: String,
}

impl Compiled {
    /// The compiled protocol, as Canon's evaluator takes it.
    pub fn ir(&self) -> &Ir {
        &self.ir
    }

    /// The `canon-ir/1` document, the bytes `canon compile` prints.
    pub fn canon_ir(&self) -> &str {
        &self.canon_ir
    }
}

/// Validates and compiles the `protocol/1` document at `path`, relative to the repository root.
pub fn compile_protocol(path: &str) -> Result<Compiled, CompileError> {
    let (_, text) = read_confined(path, "protocol path").map_err(CompileError::Unreadable)?;
    let protocol = b10x_canon::model::parse(&text).map_err(CompileError::Parse)?;
    b10x_canon::validate::validate(&protocol).map_err(CompileError::Invalid)?;
    let ir = b10x_canon::ir::compile(&protocol).map_err(CompileError::Invalid)?;
    let canon_ir = ir.canonical_json();
    Ok(Compiled { ir, canon_ir })
}

/// One loaded `els-fixture/1` document.
#[derive(Debug, Clone)]
pub struct Fixture {
    id: String,
    protocol: String,
    at: String,
    case: Case,
    states: Vec<State>,
}

/// One state as loaded: the evidence it adds, read by Canon, the case snapshot and authority
/// decisions it is evaluated with, and its expectation.
#[derive(Debug, Clone)]
struct State {
    id: String,
    add_evidence: Vec<EvidenceRecord>,
    /// The fixture's case with every revision set by this state and the states before it.
    case: Case,
    /// The `canon-authority/1` list of every decision added by this state and the states before
    /// it, or `None` when none of them gives `add_authority`.
    authority: Option<String>,
    /// The `canon-decisions/1` list of every explicit decision added by this state and the states
    /// before it, or `None` when none of them gives `add_decisions`.
    decisions: Option<String>,
    expect: Expectation,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expectation {
    #[serde(deserialize_with = "claims_once")]
    claims: BTreeMap<String, Truth>,
    #[serde(default, deserialize_with = "statuses_once")]
    obligations: Option<BTreeMap<String, String>>,
    #[serde(default, deserialize_with = "statuses_once")]
    actions: Option<BTreeMap<String, String>>,
    #[serde(default, deserialize_with = "statuses_once")]
    outcomes: Option<BTreeMap<String, String>>,
}

/// The document as written. The case and the evidence records stay YAML values until Canon's
/// readers read them. Every key is required and must have a value.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    #[serde(deserialize_with = "required")]
    format: String,
    #[serde(deserialize_with = "required")]
    id: String,
    #[serde(deserialize_with = "required")]
    protocol: String,
    #[serde(deserialize_with = "required")]
    at: String,
    #[serde(deserialize_with = "required")]
    case: Value,
    #[serde(deserialize_with = "required")]
    states: Vec<WrittenState>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenState {
    #[serde(deserialize_with = "required")]
    id: String,
    #[serde(default, deserialize_with = "revisions_once")]
    set_revisions: Option<BTreeMap<String, Value>>,
    #[serde(default, deserialize_with = "present")]
    add_authority: Option<Vec<Value>>,
    #[serde(default, deserialize_with = "present")]
    add_decisions: Option<Vec<Value>>,
    #[serde(deserialize_with = "required")]
    add_evidence: Vec<WrittenObservation>,
    #[serde(deserialize_with = "required")]
    expect: Expectation,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenObservation {
    #[serde(deserialize_with = "required")]
    observed_at: String,
    #[serde(deserialize_with = "required")]
    record: Value,
}

/// Reads a key's value, refusing a key written with no value (`add_evidence:`) instead of reading
/// it as an empty list, an empty text or `~`, as Canon refuses one.
fn required<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    Option::<T>::deserialize(deserializer)?
        .ok_or_else(|| D::Error::custom("a key is written with no value"))
}

/// Reads an optional key's value: absent is `None`, and a key written with no value is refused as
/// [`required`] refuses it.
fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    required(deserializer).map(Some)
}

/// A map read by [`Once`]: each key at most once, the repetition refused as `repeated` words it.
struct Once<V> {
    expecting: &'static str,
    repeated: fn(&str) -> String,
    value: std::marker::PhantomData<V>,
}

impl<'de, V: Deserialize<'de>> Visitor<'de> for Once<V> {
    type Value = BTreeMap<String, V>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.expecting)
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut entries = BTreeMap::new();
        while let Some((key, value)) = map.next_entry::<String, V>()? {
            if entries.contains_key(&key) {
                return Err(A::Error::custom((self.repeated)(&key)));
            }
            entries.insert(key, value);
        }
        Ok(entries)
    }
}

/// Reads a map with [`Once`], refusing a key written with no value as [`required`] refuses it.
fn map_once<'de, D: Deserializer<'de>, V: Deserialize<'de>>(
    deserializer: D,
    visitor: Once<V>,
) -> Result<BTreeMap<String, V>, D::Error> {
    struct Wrapped<V>(Once<V>);

    impl<'de, V: Deserialize<'de>> serde::de::DeserializeSeed<'de> for Wrapped<V> {
        type Value = BTreeMap<String, V>;

        fn deserialize<D: Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, D::Error> {
            deserializer.deserialize_map(self.0)
        }
    }

    struct Optional<V>(Once<V>);

    impl<'de, V: Deserialize<'de>> Visitor<'de> for Optional<V> {
        type Value = Option<BTreeMap<String, V>>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0.expecting)
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D: Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, D::Error> {
            serde::de::DeserializeSeed::deserialize(Wrapped(self.0), deserializer).map(Some)
        }
    }

    deserializer
        .deserialize_option(Optional(visitor))?
        .ok_or_else(|| D::Error::custom("a key is written with no value"))
}

/// Reads `expect.claims`, refusing a claim written more than once instead of keeping the last.
fn claims_once<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Truth>, D::Error> {
    map_once(
        deserializer,
        Once {
            expecting: "a map of claim ids to true, false or unknown",
            repeated: |claim| format!("claim `{claim}` is expected more than once"),
            value: std::marker::PhantomData,
        },
    )
}

/// Reads `expect.obligations`, `expect.actions` or `expect.outcomes`, refusing an id written more
/// than once.
fn statuses_once<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<BTreeMap<String, String>>, D::Error> {
    map_once(
        deserializer,
        Once {
            expecting: "a map of obligation, action or outcome ids to their status",
            repeated: |id| format!("`{id}` is expected more than once"),
            value: std::marker::PhantomData,
        },
    )
    .map(Some)
}

/// Reads `set_revisions`, refusing an artifact written more than once. The revisions stay YAML
/// values until Canon's case reader reads them.
fn revisions_once<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<BTreeMap<String, Value>>, D::Error> {
    map_once(
        deserializer,
        Once {
            expecting: "a map of artifact ids to revisions",
            repeated: |artifact| format!("artifact `{artifact}` is set more than once"),
            value: std::marker::PhantomData,
        },
    )
    .map(Some)
}

impl Fixture {
    /// Loads the fixture at `path`, relative to the repository root. The file must be named
    /// `<id>.fixture.yaml` after the id the fixture declares; a symbolic link is judged by the
    /// name of the file it resolves to, which is the fixture it loads.
    pub fn load(path: &str) -> Result<Self, FixtureError> {
        let (resolved, text) =
            read_confined(path, "fixture path").map_err(FixtureError::Unreadable)?;
        let fixture = Self::from_yaml(&text)?;
        let file = resolved
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let named = format!("{}.fixture.yaml", fixture.id);
        if file != named {
            return Err(FixtureError::Refused(format!(
                "fixture file `{file}` declares id `{}`; it must be named `{named}`",
                fixture.id
            )));
        }
        Ok(fixture)
    }

    /// Reads an `els-fixture/1` document from its text.
    pub fn from_yaml(text: &str) -> Result<Self, FixtureError> {
        let written: Written = serde_yaml_ng::from_str(text)
            .map_err(|error| FixtureError::Malformed(error.to_string()))?;
        let refuse = |why: String| Err(FixtureError::Refused(why));
        if written.format != FORMAT {
            return refuse(format!(
                "format is `{}`, expected `{FORMAT}`",
                written.format
            ));
        }
        if written.protocol.is_empty() || !b10x_canon::conform::is_confined(&written.protocol) {
            return refuse(format!(
                "protocol path `{}` is absolute or leaves the repository",
                written.protocol
            ));
        }
        if !is_instant(&written.at) {
            return refuse(format!(
                "evaluation instant `{}` is not written YYYY-MM-DDTHH:MM:SSZ",
                written.at
            ));
        }
        let case = eval::case_from_value(&written.case).map_err(|refusal| {
            FixtureError::Refused(format!("case: {}: {refusal}", refusal.code()))
        })?;
        if written.states.is_empty() {
            return refuse("the fixture lists no states".to_owned());
        }
        let mut seen = BTreeSet::new();
        let mut states = Vec::with_capacity(written.states.len());
        let mut snapshot = written.case.clone();
        let mut state_case = case.clone();
        let mut authority: Option<Vec<Value>> = None;
        let mut decisions: Option<Vec<Value>> = None;
        for state in written.states {
            if !seen.insert(state.id.clone()) {
                return refuse(format!("state `{}` is given more than once", state.id));
            }
            if let Some(revisions) = &state.set_revisions {
                for (artifact, revision) in revisions {
                    let entry = snapshot
                        .get_mut("artifacts")
                        .and_then(|artifacts| artifacts.get_mut(artifact.as_str()))
                        .and_then(|entry| entry.get_mut("revision"));
                    let Some(entry) = entry else {
                        return refuse(format!(
                            "state `{}`: sets the revision of artifact `{artifact}`, which the \
                             case does not list",
                            state.id
                        ));
                    };
                    *entry = revision.clone();
                }
                state_case = eval::case_from_value(&snapshot).map_err(|refusal| {
                    FixtureError::Refused(format!(
                        "state `{}`: case: {}: {refusal}",
                        state.id,
                        refusal.code()
                    ))
                })?;
            }
            let authority_text = accumulate(
                &state.id,
                "authority decision",
                "canon-authority/1",
                &mut authority,
                state.add_authority.as_deref(),
            )?;
            let decisions_text = accumulate(
                &state.id,
                "explicit decision",
                "canon-decisions/1",
                &mut decisions,
                state.add_decisions.as_deref(),
            )?;
            let mut add_evidence = Vec::with_capacity(state.add_evidence.len());
            for (index, observation) in state.add_evidence.iter().enumerate() {
                let mut record =
                    eval::evidence_from_value(&observation.record).map_err(|refusal| {
                        FixtureError::Refused(format!(
                            "state `{}`: evidence {}: {}: {refusal}",
                            state.id,
                            index + 1,
                            refusal.code()
                        ))
                    })?;
                let evidence = record.id.as_str().to_owned();
                if !is_instant(&observation.observed_at) {
                    return refuse(format!(
                        "state `{}`: evidence `{evidence}` observation instant `{}` is not \
                         written YYYY-MM-DDTHH:MM:SSZ",
                        state.id, observation.observed_at
                    ));
                }
                if observation.observed_at > written.at {
                    return refuse(format!(
                        "state `{}`: evidence `{evidence}` is observed at `{}`, after the \
                         evaluation instant `{}`",
                        state.id, observation.observed_at, written.at
                    ));
                }
                // A record may write its own `observed_at` (`canon-evidence/1` reads one). It is
                // held to the same rules as the entry's and must agree with it; then it is dropped,
                // so no observation time reaches Canon (story:stale-evidence-fixtures).
                if let Some(own) = record.observed_at.take() {
                    let own = own.as_str();
                    if !is_instant(own) {
                        return refuse(format!(
                            "state `{}`: evidence `{evidence}` record observation instant \
                             `{own}` is not written YYYY-MM-DDTHH:MM:SSZ",
                            state.id
                        ));
                    }
                    if own > written.at.as_str() {
                        return refuse(format!(
                            "state `{}`: evidence `{evidence}` record is observed at `{own}`, \
                             after the evaluation instant `{}`",
                            state.id, written.at
                        ));
                    }
                    if own != observation.observed_at {
                        return refuse(format!(
                            "state `{}`: evidence `{evidence}` record is observed at `{own}`, its \
                             entry at `{}`",
                            state.id, observation.observed_at
                        ));
                    }
                }
                add_evidence.push(record);
            }
            states.push(State {
                id: state.id,
                add_evidence,
                case: state_case.clone(),
                authority: authority_text,
                decisions: decisions_text,
                expect: state.expect,
            });
        }
        Ok(Fixture {
            id: written.id,
            protocol: written.protocol,
            at: written.at,
            case,
            states,
        })
    }

    /// The fixture's name.
    #[allow(dead_code)] // later-story API: protocol stories name the fixture in their messages
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The protocol the fixture evaluates, relative to the repository root.
    pub fn protocol_path(&self) -> &str {
        &self.protocol
    }

    /// The evaluation instant.
    pub fn at(&self) -> &str {
        &self.at
    }

    /// The fixture's case snapshot, before any state sets a revision.
    #[allow(dead_code)] // later-story API: protocol stories read the case they evaluate
    pub fn case(&self) -> &Case {
        &self.case
    }

    /// The state ids, in order.
    #[allow(dead_code)] // later-story API: protocol stories walk a fixture state by state
    pub fn states(&self) -> Vec<&str> {
        self.states.iter().map(|state| state.id.as_str()).collect()
    }

    /// Validates and compiles the fixture's protocol.
    pub fn compile(&self) -> Result<Compiled, CompileError> {
        compile_protocol(&self.protocol)
    }

    /// The evidence state `state` is evaluated over: its own and that of every state before it.
    /// Panics when the fixture has no such state.
    pub fn evidence(&self, state: &str) -> Vec<EvidenceRecord> {
        let end = self
            .states
            .iter()
            .position(|candidate| candidate.id == state)
            .unwrap_or_else(|| panic!("fixture `{}` has no state `{state}`", self.id));
        self.states[..=end]
            .iter()
            .flat_map(|state| state.add_evidence.iter().cloned())
            .collect()
    }

    /// Canon's evaluation of state `state`, unchanged: of the state's case snapshot, its evidence,
    /// its authority decisions and its explicit decisions. Panics when the fixture has no such
    /// state.
    pub fn evaluate(&self, compiled: &Compiled, state: &str) -> Result<Decision, Refusal> {
        let found = self
            .states
            .iter()
            .find(|candidate| candidate.id == state)
            .unwrap_or_else(|| panic!("fixture `{}` has no state `{state}`", self.id));
        let supplied = eval::Supplied {
            authority: found.authority.as_deref(),
            decisions: found.decisions.as_deref(),
            ..eval::Supplied::default()
        };
        eval::evaluate_with(compiled.ir(), &found.case, &self.evidence(state), supplied)
    }

    /// Evaluates every state and compares each decision with the state's expectation: every
    /// claim the decision holds must be expected with its value, and every expected claim must
    /// be in the decision; the same for obligations, actions and outcomes and their statuses,
    /// when the state expects them. Returns one line per difference, in state order, then claims,
    /// obligations, actions and outcomes, each in id order.
    pub fn check(&self, compiled: &Compiled) -> Result<(), Vec<String>> {
        let mut differences = Vec::new();
        for state in &self.states {
            let decision = match self.evaluate(compiled, &state.id) {
                Ok(decision) => decision,
                Err(refusal) => {
                    differences.push(format!(
                        "state `{}`: evaluation refused: {}: {refusal}",
                        state.id,
                        refusal.code()
                    ));
                    continue;
                }
            };
            let found: BTreeMap<String, Truth> = decision
                .claims
                .iter()
                .map(|(id, entry)| (id.as_str().to_owned(), entry.value))
                .collect();
            let mut report = |noun: &str,
                              found: &BTreeMap<String, String>,
                              expected: &BTreeMap<String, String>| {
                compare(&state.id, noun, found, expected, &mut differences);
            };
            let claims = |values: &BTreeMap<String, Truth>| -> BTreeMap<String, String> {
                values
                    .iter()
                    .map(|(id, value)| (id.clone(), value.to_string()))
                    .collect()
            };
            report("claim", &claims(&found), &claims(&state.expect.claims));
            if let Some(expected) = &state.expect.obligations {
                report("obligation", &obligation_statuses(&decision), expected);
            }
            if let Some(expected) = &state.expect.actions {
                report("action", &action_statuses(&decision), expected);
            }
            if let Some(expected) = &state.expect.outcomes {
                report("outcome", &outcome_statuses(&decision), expected);
            }
        }
        if differences.is_empty() {
            Ok(())
        } else {
            Err(differences)
        }
    }
}

/// Adds a state's entries (`added`, its `add_authority` or `add_decisions`) to those of the states
/// before it (`so_far`), and returns every entry so far written as the one `format` list Canon
/// reads, or `None` when no state so far gave any. An entry that cannot be written as YAML at all
/// is refused, naming the state and its position; Canon reads and refuses the rest.
fn accumulate(
    state: &str,
    entry: &str,
    format: &str,
    so_far: &mut Option<Vec<Value>>,
    added: Option<&[Value]>,
) -> Result<Option<String>, FixtureError> {
    if let Some(added) = added {
        for (index, value) in added.iter().enumerate() {
            if let Err(error) = serde_yaml_ng::to_string(value) {
                return Err(FixtureError::Refused(format!(
                    "state `{state}`: {entry} {} cannot be written as {format}: {error}",
                    index + 1
                )));
            }
        }
        so_far
            .get_or_insert_with(Vec::new)
            .extend(added.iter().cloned());
    }
    match so_far.as_ref().map(serde_yaml_ng::to_string) {
        None => Ok(None),
        Some(Ok(text)) => Ok(Some(text)),
        Some(Err(error)) => Err(FixtureError::Refused(format!(
            "state `{state}`: the {entry}s so far cannot be written as {format}: {error}"
        ))),
    }
}

/// Adds one line to `differences` per id whose value in `found` differs from that in `expected`,
/// in id order: every id the decision holds must be expected with its value, and every expected
/// id must be in the decision.
fn compare(
    state: &str,
    noun: &str,
    found: &BTreeMap<String, String>,
    expected: &BTreeMap<String, String>,
    differences: &mut Vec<String>,
) {
    let ids: BTreeSet<&String> = found.keys().chain(expected.keys()).collect();
    for id in ids {
        let line = match (found.get(id), expected.get(id)) {
            (Some(found), Some(expected)) if found == expected => continue,
            (Some(found), Some(expected)) => format!("is {found}, expected {expected}"),
            (Some(found), None) => format!("is {found}, expected nothing"),
            (None, Some(expected)) => format!("is not declared, expected {expected}"),
            (None, None) => unreachable!("the id comes from one of the two"),
        };
        differences.push(format!("state `{state}`: {noun} `{id}` {line}"));
    }
}

/// The status of each obligation in Canon's decision, keyed by obligation id; empty when the
/// decision has no `obligations` section.
fn obligation_statuses(decision: &Decision) -> BTreeMap<String, String> {
    let entries = decision
        .obligations
        .as_ref()
        .and_then(|section| section.as_array())
        .map(Vec::as_slice)
        .unwrap_or_default();
    entries
        .iter()
        .map(|entry| (text(&entry["id"]), text(&entry["status"])))
        .collect()
}

/// The status of each action in Canon's decision, keyed by action id; empty when the decision
/// has no `actions` section.
fn action_statuses(decision: &Decision) -> BTreeMap<String, String> {
    decision
        .actions
        .as_ref()
        .and_then(|section| section.as_object())
        .map(|actions| {
            actions
                .iter()
                .map(|(id, entry)| (id.clone(), text(&entry["status"])))
                .collect()
        })
        .unwrap_or_default()
}

/// The status of each outcome in Canon's decision, keyed by outcome id; empty when the decision
/// has no `outcomes` section.
fn outcome_statuses(decision: &Decision) -> BTreeMap<String, String> {
    decision
        .outcomes
        .as_ref()
        .and_then(|section| section.as_object())
        .map(|outcomes| {
            outcomes
                .iter()
                .map(|(id, entry)| (id.clone(), text(&entry["status"])))
                .collect()
        })
        .unwrap_or_default()
}

/// A text value of Canon's decision as written, or the JSON it holds when it is not text.
fn text(value: &b10x_canon::model::Json) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_owned)
}

/// Reads `path`, relative to the repository root, and returns the file it resolves to and its
/// text, refusing a path that is absolute, has a `..` component or resolves outside the
/// repository through a symbolic link.
fn read_confined(path: &str, what: &str) -> Result<(PathBuf, String), String> {
    if path.is_empty() || !b10x_canon::conform::is_confined(path) {
        return Err(format!(
            "{what} `{path}` is absolute or leaves the repository"
        ));
    }
    let root = repo_root();
    let resolved = root
        .join(path)
        .canonicalize()
        .map_err(|error| format!("{what} `{path}`: {error}"))?;
    if !resolved.starts_with(&root) {
        return Err(format!("{what} `{path}` resolves outside the repository"));
    }
    let text =
        std::fs::read_to_string(&resolved).map_err(|error| format!("{what} `{path}`: {error}"))?;
    Ok((resolved, text))
}

/// Whether `text` is an instant written `YYYY-MM-DDTHH:MM:SSZ` naming a real day (leap years
/// counted) and time, the one form in which instants compare correctly as text.
fn is_instant(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() != 20 {
        return false;
    }
    let shape = b"dddd-dd-ddTdd:dd:ddZ";
    let shaped = bytes
        .iter()
        .zip(shape)
        .all(|(byte, expected)| match expected {
            b'd' => byte.is_ascii_digit(),
            literal => byte == literal,
        });
    if !shaped {
        return false;
    }
    let field = |from: usize, to: usize| text[from..to].parse::<u32>().unwrap_or(u32::MAX);
    let (year, month) = (field(0, 4), field(5, 7));
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&field(8, 10))
        && field(11, 13) <= 23
        && field(14, 16) <= 59
        && field(17, 19) <= 59
}
