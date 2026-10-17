//! Adversarial review of `story:eval-gated-change-protocol` at `49dea9a`: `eval.gated_change/1`,
//! its fixtures under `fixtures/eval-gated-change/` and `eval_gated_change_protocol.rs`.
//!
//! The behavioural cases build fixtures in memory and evaluate them through the harness, as a
//! fixture file's states are. The mutant cases write a mutated copy of the protocol under the
//! tree's `target/` (never the protocol itself), compile it, run every eval-gated-change fixture
//! against it and run `canon check` over it: a mutant every fixture still passes and `canon check`
//! still finds clean is a rule the unit's suite does not exercise, since every assertion of
//! `eval_gated_change_outcomes_are_earned` past item 2 reads a fixture state.
//!
//! Red on purpose (each names the gap it shows):
//! - `egc_reject_guards_are_each_killed_by_a_fixture`
//! - `egc_rejected_outcome_guards_are_each_killed_by_a_fixture`
//! - `egc_tuning_improved_reading_is_killed_by_a_fixture`

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::collections::BTreeMap;

use b10x_canon::model::{Decision, ExclusionReason, Json};
use support::{Compiled, Fixture, compile_protocol, repo_root};

const PROTOCOL: &str = "protocols/eval-gated-change/1.yaml";
const AT: &str = "2026-10-17T12:00:00Z";
const RUN_B: &str = "{run-b: [different_agent_run]}";

/// One evidence entry of a fixture state.
fn entry(
    observed_at: &str,
    id: &str,
    kind: &str,
    result: &str,
    subject: &str,
    revision: &str,
    producer: &str,
) -> String {
    format!(
        "      - observed_at: \"{observed_at}\"\n        record: {{format: canon-evidence/1, \
         id: {id}, kind: {kind}, result: {result}, subject: {subject}, \
         subject_revision: {revision}, producer: {producer}}}\n"
    )
}

fn design(result: &str, producer: &str) -> String {
    entry(
        "2026-10-17T09:00:00Z",
        "dr",
        "design_review",
        result,
        "eval_design",
        "h1",
        producer,
    )
}

fn guard(result: &str, producer: &str) -> String {
    entry(
        "2026-10-17T10:00:00Z",
        "gc",
        "guard_check",
        result,
        "candidate",
        "h1",
        producer,
    )
}

fn tuning(result: &str, producer: &str) -> String {
    entry(
        "2026-10-17T10:10:00Z",
        "tc",
        "tuning_comparison",
        result,
        "candidate",
        "h1",
        producer,
    )
}

fn heldout(result: &str, producer: &str) -> String {
    entry(
        "2026-10-17T10:20:00Z",
        "hc",
        "heldout_comparison",
        result,
        "candidate",
        "h1",
        producer,
    )
}

/// One state: its id, the extra keys it gives (already indented), and its evidence entries.
fn state(id: &str, extra: &str, evidence: &[String]) -> String {
    let evidence = if evidence.is_empty() {
        "    add_evidence: []\n".to_owned()
    } else {
        format!("    add_evidence:\n{}", evidence.concat())
    };
    format!("  - id: {id}\n{extra}{evidence}    expect: {{claims: {{}}}}\n")
}

const GRANT_KEEP: &str =
    "    add_authority:\n      - {capability: loop_change.keep, decision: granted}\n";

/// An `eval.gated_change/1` fixture at case revision `case_revision`, `eval_design` and
/// `candidate` at h1, with the independence map `independence` and `states`.
fn egc(case_revision: &str, independence: &str, states: &[String]) -> Fixture {
    let text = format!(
        "format: els-fixture/1\nid: adversary-egc\nprotocol: {PROTOCOL}\nat: \"{AT}\"\n\
         case:\n  format: canon-case/1\n  id: EGC-9001\n  protocol: eval.gated_change\n  \
         revision: {case_revision}\n  independence: {independence}\n  artifacts:\n    \
         eval_design: {{revision: h1}}\n    candidate: {{revision: h1}}\nstates:\n{}",
        states.concat()
    );
    Fixture::from_yaml(&text).unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn compiled() -> Compiled {
    compile_protocol(PROTOCOL).unwrap_or_else(|error| panic!("{PROTOCOL}: {error}"))
}

fn decide(fixture: &Fixture, compiled: &Compiled, state: &str) -> Decision {
    fixture
        .evaluate(compiled, state)
        .unwrap_or_else(|refusal| panic!("state `{state}` refused: {}: {refusal}", refusal.code()))
}

fn claim(decision: &Decision, id: &str) -> String {
    decision
        .claims
        .iter()
        .find(|(claim, _)| claim.as_str() == id)
        .map_or("undeclared".to_owned(), |(_, entry)| {
            entry.value.to_string()
        })
}

fn excluded(decision: &Decision, id: &str) -> Vec<ExclusionReason> {
    decision
        .claims
        .iter()
        .find(|(claim, _)| claim.as_str() == id)
        .map(|(_, entry)| entry.excluded_evidence.iter().map(|e| e.reason).collect())
        .unwrap_or_default()
}

fn statuses(section: &Option<Json>) -> BTreeMap<String, String> {
    match section {
        Some(Json::Array(entries)) => entries
            .iter()
            .map(|entry| {
                (
                    entry["id"].as_str().unwrap_or_default().to_owned(),
                    entry["status"].as_str().unwrap_or_default().to_owned(),
                )
            })
            .collect(),
        Some(Json::Object(entries)) => entries
            .iter()
            .map(|(id, entry)| {
                (
                    id.clone(),
                    entry["status"].as_str().unwrap_or_default().to_owned(),
                )
            })
            .collect(),
        _ => BTreeMap::new(),
    }
}

fn action(decision: &Decision, id: &str) -> String {
    statuses(&decision.actions)
        .get(id)
        .cloned()
        .unwrap_or_else(|| "undeclared".to_owned())
}

fn legitimate(decision: &Decision) -> Vec<String> {
    statuses(&decision.outcomes)
        .into_iter()
        .filter(|(_, status)| status == "legitimate")
        .map(|(id, _)| id)
        .collect()
}

const KEEP_CLAIMS: [&str; 6] = [
    "experiment.valid",
    "change.within_surface",
    "content.clean",
    "tuning.improved",
    "result.reproducible",
    "heldout.not_regressed",
];

// ---------------------------------------------------------------------------------------------
// The loop's own evidence, and producers the case does not list
// ---------------------------------------------------------------------------------------------

/// Passing evidence by a producer the case does not list at all (run-c), or lists with no
/// dimension, or lists on no dimension the protocol demands, is excluded as not independent: every
/// claim UNKNOWN, neither keep nor reject, no outcome legitimate, even with `loop_change.keep`.
#[test]
fn egc_unlisted_or_empty_producer_neither_keeps_nor_rejects() {
    let compiled = compiled();
    for (independence, producer) in [
        (RUN_B, "run-c"),
        ("{run-b: [different_agent_run], run-a: []}", "run-a"),
    ] {
        let passing = [
            design("passed", producer),
            guard("clean", producer),
            tuning("reproduced", producer),
            heldout("held", producer),
        ];
        let failing = [
            design("failed", producer),
            guard("leaked", producer),
            tuning("not_better", producer),
            heldout("regressed", producer),
        ];
        for (name, evidence) in [("passing", &passing), ("failing", &failing)] {
            let fixture = egc("c1", independence, &[state(name, GRANT_KEEP, evidence)]);
            let decision = decide(&fixture, &compiled, name);
            for id in KEEP_CLAIMS {
                assert_eq!(claim(&decision, id), "unknown", "{producer} {name}: `{id}`");
                let reasons = excluded(&decision, id);
                assert!(
                    !reasons.is_empty()
                        && reasons
                            .iter()
                            .all(|r| *r == ExclusionReason::NotIndependent),
                    "{producer} {name}: `{id}` excludes as not independent: {reasons:?}"
                );
            }
            assert_eq!(claim(&decision, "change.supported"), "unknown");
            assert_eq!(
                action(&decision, "change.keep"),
                "blocked",
                "{producer} {name}"
            );
            assert_eq!(
                action(&decision, "change.reject"),
                "blocked",
                "{producer} {name}"
            );
            assert_eq!(
                action(&decision, "change.propose"),
                "blocked",
                "{producer} {name}"
            );
            assert!(legitimate(&decision).is_empty(), "{producer} {name}");
        }
    }
}

/// The loop's own failing record beside run-b's passing ones is excluded, so it neither rejects
/// nor clouds the keep; and run-a's `outside` beside run-b's clean neither proposes nor blocks.
#[test]
fn egc_own_records_beside_independent_ones_change_nothing() {
    let compiled = compiled();
    let fixture = egc(
        "c1",
        RUN_B,
        &[state(
            "mixed",
            GRANT_KEEP,
            &[
                design("passed", "run-b"),
                guard("clean", "run-b"),
                tuning("reproduced", "run-b"),
                heldout("held", "run-b"),
                entry(
                    "2026-10-17T10:30:00Z",
                    "gc-a",
                    "guard_check",
                    "outside",
                    "candidate",
                    "h1",
                    "run-a",
                ),
                entry(
                    "2026-10-17T10:31:00Z",
                    "hc-a",
                    "heldout_comparison",
                    "regressed",
                    "candidate",
                    "h1",
                    "run-a",
                ),
            ],
        )],
    );
    let decision = decide(&fixture, &compiled, "mixed");
    assert_eq!(claim(&decision, "change.supported"), "true");
    assert_eq!(action(&decision, "change.keep"), "admissible");
    assert_eq!(action(&decision, "change.propose"), "blocked");
    assert_eq!(legitimate(&decision), ["kept"]);
}

// ---------------------------------------------------------------------------------------------
// Subject confusion: evidence about eval_design versus the candidate
// ---------------------------------------------------------------------------------------------

/// Both artifacts are at h1. Candidate evidence written against `eval_design`, and a design review
/// written against `candidate`, decide nothing: no keep claim is TRUE and nothing is admissible.
#[test]
fn egc_evidence_on_the_wrong_subject_decides_nothing() {
    let compiled = compiled();
    let swapped = [
        entry(
            "2026-10-17T09:00:00Z",
            "dr",
            "design_review",
            "passed",
            "candidate",
            "h1",
            "run-b",
        ),
        entry(
            "2026-10-17T10:00:00Z",
            "gc",
            "guard_check",
            "clean",
            "eval_design",
            "h1",
            "run-b",
        ),
        entry(
            "2026-10-17T10:10:00Z",
            "tc",
            "tuning_comparison",
            "reproduced",
            "eval_design",
            "h1",
            "run-b",
        ),
        entry(
            "2026-10-17T10:20:00Z",
            "hc",
            "heldout_comparison",
            "held",
            "eval_design",
            "h1",
            "run-b",
        ),
    ];
    let fixture = egc("c1", RUN_B, &[state("swapped", GRANT_KEEP, &swapped)]);
    let decision = decide(&fixture, &compiled, "swapped");
    for id in KEEP_CLAIMS {
        assert_eq!(claim(&decision, id), "unknown", "`{id}`");
    }
    assert_eq!(action(&decision, "change.keep"), "blocked");
    assert_eq!(action(&decision, "change.reject"), "blocked");
    assert!(legitimate(&decision).is_empty());
}

// ---------------------------------------------------------------------------------------------
// UNKNOWN never becomes a decision
// ---------------------------------------------------------------------------------------------

/// A leaked guard check alone, by run-b: `content.clean` FALSE and `change.within_surface` TRUE,
/// but the design, tuning and held-out claims are UNKNOWN, so neither `change.reject` nor
/// `rejected`. No fixture has a state with the candidate within the surface and another keep
/// claim UNKNOWN (fixture `rejected-heldout` state `heldout-regressed` leaves the surface UNKNOWN,
/// which blocks `rejected` on its own); this state is the one that tells the guards apart.
#[test]
fn egc_leaked_alone_neither_rejects_nor_closes_rejected() {
    let compiled = compiled();
    let fixture = egc(
        "c1",
        RUN_B,
        &[state(
            "leaked-alone",
            GRANT_KEEP,
            &[guard("leaked", "run-b")],
        )],
    );
    let decision = decide(&fixture, &compiled, "leaked-alone");
    assert_eq!(claim(&decision, "change.within_surface"), "true");
    assert_eq!(claim(&decision, "content.clean"), "false");
    assert_eq!(claim(&decision, "change.supported"), "false");
    assert_eq!(action(&decision, "change.reject"), "blocked");
    assert!(
        legitimate(&decision).is_empty(),
        "{:?}",
        legitimate(&decision)
    );
}

/// Every keep claim decided TRUE except one left UNKNOWN: `rejected` is blocked and so is
/// `change.reject`, for each claim in turn (a leaked guard check gives the FALSE one).
#[test]
fn egc_each_unknown_keep_claim_blocks_rejection() {
    let compiled = compiled();
    let cases: [(&str, Vec<String>); 3] = [
        (
            "experiment.valid",
            vec![
                guard("leaked", "run-b"),
                tuning("reproduced", "run-b"),
                heldout("held", "run-b"),
            ],
        ),
        (
            "heldout.not_regressed",
            vec![
                design("passed", "run-b"),
                guard("leaked", "run-b"),
                tuning("reproduced", "run-b"),
            ],
        ),
        (
            "tuning.improved",
            vec![
                design("passed", "run-b"),
                guard("leaked", "run-b"),
                heldout("held", "run-b"),
            ],
        ),
    ];
    for (unknown, evidence) in cases {
        let fixture = egc("c1", RUN_B, &[state("one-unknown", GRANT_KEEP, &evidence)]);
        let decision = decide(&fixture, &compiled, "one-unknown");
        assert_eq!(claim(&decision, unknown), "unknown", "`{unknown}`");
        assert_eq!(claim(&decision, "content.clean"), "false", "`{unknown}`");
        assert_eq!(
            action(&decision, "change.reject"),
            "blocked",
            "`{unknown}` unknown"
        );
        assert!(
            legitimate(&decision).is_empty(),
            "`{unknown}` unknown: {:?}",
            legitimate(&decision)
        );
    }
}

/// A first tuning run better and the second not (`not_reproduced`): `tuning.improved` TRUE,
/// `result.reproducible` FALSE, reject admissible, `rejected` legitimate. No fixture records
/// `not_reproduced`.
#[test]
fn egc_not_reproduced_is_improved_but_not_reproducible() {
    let compiled = compiled();
    let fixture = egc(
        "c1",
        RUN_B,
        &[state(
            "not-reproduced",
            GRANT_KEEP,
            &[
                design("passed", "run-b"),
                guard("clean", "run-b"),
                tuning("not_reproduced", "run-b"),
                heldout("held", "run-b"),
            ],
        )],
    );
    let decision = decide(&fixture, &compiled, "not-reproduced");
    assert_eq!(claim(&decision, "tuning.improved"), "true");
    assert_eq!(claim(&decision, "result.reproducible"), "false");
    assert_eq!(action(&decision, "change.reject"), "admissible");
    assert_eq!(action(&decision, "change.keep"), "blocked");
    assert_eq!(legitimate(&decision), ["rejected"]);
}

// ---------------------------------------------------------------------------------------------
// Basis moves and stale decisions
// ---------------------------------------------------------------------------------------------

/// A rejection about h1 does not survive the candidate moving to h2, and partial evidence about h2
/// beside complete evidence about h1 keeps nothing.
#[test]
fn egc_basis_move_unsettles_reject_and_keep() {
    let compiled = compiled();
    let to_h2 = "    set_revisions:\n      candidate: h2\n";
    let fixture = egc(
        "c1",
        RUN_B,
        &[
            state(
                "rejected-h1",
                GRANT_KEEP,
                &[
                    design("passed", "run-b"),
                    guard("leaked", "run-b"),
                    tuning("reproduced", "run-b"),
                    heldout("held", "run-b"),
                ],
            ),
            state("moved", to_h2, &[]),
            state(
                "h2-partly",
                "",
                &[
                    entry(
                        "2026-10-17T11:00:00Z",
                        "gc2",
                        "guard_check",
                        "clean",
                        "candidate",
                        "h2",
                        "run-b",
                    ),
                    entry(
                        "2026-10-17T11:10:00Z",
                        "tc2",
                        "tuning_comparison",
                        "reproduced",
                        "candidate",
                        "h2",
                        "run-b",
                    ),
                ],
            ),
        ],
    );
    let rejected = decide(&fixture, &compiled, "rejected-h1");
    assert_eq!(legitimate(&rejected), ["rejected"]);
    let moved = decide(&fixture, &compiled, "moved");
    assert_eq!(action(&moved, "change.reject"), "blocked");
    assert_eq!(action(&moved, "change.keep"), "blocked");
    assert!(legitimate(&moved).is_empty(), "{:?}", legitimate(&moved));
    let partly = decide(&fixture, &compiled, "h2-partly");
    assert_eq!(claim(&partly, "heldout.not_regressed"), "unknown");
    assert_ne!(claim(&partly, "change.supported"), "true");
    assert_eq!(action(&partly, "change.keep"), "blocked");
    assert!(legitimate(&partly).is_empty(), "{:?}", legitimate(&partly));
}

/// The evaluation design moving to h2 while the candidate stays at h1 takes `experiment.valid`
/// away, and with it the keep.
#[test]
fn egc_design_move_takes_the_keep_away() {
    let compiled = compiled();
    let fixture = egc(
        "c1",
        RUN_B,
        &[
            state(
                "kept",
                GRANT_KEEP,
                &[
                    design("passed", "run-b"),
                    guard("clean", "run-b"),
                    tuning("reproduced", "run-b"),
                    heldout("held", "run-b"),
                ],
            ),
            state(
                "design-moved",
                "    set_revisions:\n      eval_design: h2\n",
                &[],
            ),
        ],
    );
    assert_eq!(legitimate(&decide(&fixture, &compiled, "kept")), ["kept"]);
    let moved = decide(&fixture, &compiled, "design-moved");
    assert_eq!(claim(&moved, "experiment.valid"), "unknown");
    assert_eq!(action(&moved, "change.keep"), "blocked");
    assert!(legitimate(&moved).is_empty(), "{:?}", legitimate(&moved));
}

/// A decision taken at case revision c1 is stale at c2: `inconclusive` stays blocked.
#[test]
fn egc_stale_decision_is_not_inconclusive() {
    let compiled = compiled();
    let decision = "    add_decisions:\n      - {decision: explicitly_inconclusive, outcome: inconclusive, \
                    principal: loop-owner, case_revision: c1}\n";
    let fixture = egc("c2", RUN_B, &[state("stale", decision, &[])]);
    assert!(legitimate(&decide(&fixture, &compiled, "stale")).is_empty());
}

// ---------------------------------------------------------------------------------------------
// Observations, not defects: what the protocol does when a case grants more
// ---------------------------------------------------------------------------------------------

/// `change.apply_outside_surface` is admissible as soon as a case grants
/// `loop_change.apply_outside_surface`: nothing in /1 ties that grant to a person rather than the
/// loop. Green; it records the behaviour the judgement finding is about.
#[test]
fn egc_observed_apply_outside_surface_follows_the_grant() {
    let compiled = compiled();
    let grant = "    add_authority:\n      - {capability: loop_change.apply_outside_surface, \
                 decision: granted}\n";
    let fixture = egc(
        "c1",
        RUN_B,
        &[state("outside", grant, &[guard("outside", "run-b")])],
    );
    let decision = decide(&fixture, &compiled, "outside");
    assert_eq!(
        action(&decision, "change.apply_outside_surface"),
        "admissible"
    );
}

/// An `explicitly_inconclusive` decision beside complete passing evidence: `kept` and
/// `inconclusive` are legitimate together. Green; acceptance 13 leaves `inconclusive` out of the
/// exclusive set, and this records what that allows.
#[test]
fn egc_observed_kept_and_inconclusive_together() {
    let compiled = compiled();
    let decision = "    add_decisions:\n      - {decision: explicitly_inconclusive, outcome: inconclusive, \
                    principal: loop-owner, case_revision: c1}\n";
    let fixture = egc(
        "c1",
        RUN_B,
        &[state(
            "both",
            &format!("{GRANT_KEEP}{decision}"),
            &[
                design("passed", "run-b"),
                guard("clean", "run-b"),
                tuning("reproduced", "run-b"),
                heldout("held", "run-b"),
            ],
        )],
    );
    assert_eq!(
        legitimate(&decide(&fixture, &compiled, "both")),
        ["inconclusive", "kept"]
    );
}

// ---------------------------------------------------------------------------------------------
// Mutants
// ---------------------------------------------------------------------------------------------

/// Whether the unit's suite tells `PROTOCOL` with `from` replaced by `to` once apart from the
/// protocol: some eval-gated-change fixture fails against it, or `canon check` over it finds
/// something or exceeds the bound. Panics when `from` is not in the protocol exactly once.
fn suite_kills(name: &str, from: &str, to: &str) -> bool {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(PROTOCOL)).expect("protocol reads");
    assert_eq!(text.matches(from).count(), 1, "mutant `{name}`: {from:?}");
    let mutant = text.replacen(from, to, 1);
    std::fs::create_dir_all(root.join("target/adversary-egc-mutants")).expect("mutant directory");
    let path = format!("target/adversary-egc-mutants/{name}.yaml");
    std::fs::write(root.join(&path), mutant).expect("mutant writes");
    let compiled = compile_protocol(&path).unwrap_or_else(|error| panic!("{name}: {error}"));
    let mut files: Vec<String> = std::fs::read_dir(root.join("fixtures/eval-gated-change"))
        .expect("fixtures")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .into_string()
                .expect("utf-8")
        })
        .filter(|file| file.ends_with(".fixture.yaml"))
        .collect();
    files.sort();
    let fixture_kills = files.iter().any(|file| {
        Fixture::load(&format!("fixtures/eval-gated-change/{file}"))
            .unwrap_or_else(|error| panic!("{file}: {error}"))
            .check(&compiled)
            .is_err()
    });
    if fixture_kills {
        return true;
    }
    match b10x_canon::check::check(compiled.ir(), None) {
        Ok(report) => !report.is_clean() || report.states > b10x_canon::check::STATE_BOUND,
        Err(_) => true,
    }
}

fn survivors(mutants: &[(&str, &str, &str)]) -> Vec<String> {
    mutants
        .iter()
        .filter(|(name, from, to)| !suite_kills(name, from, to))
        .map(|(name, _, _)| (*name).to_owned())
        .collect()
}

/// Control: mutants the suite does kill, so a survivor below is a gap and not a broken harness.
#[test]
fn egc_control_mutants_are_killed() {
    let mutants: [(&str, &str, &str); 4] = [
        (
            "drop-experiment-independence",
            "        subject: eval_design\n        independent_on: [different_agent_run]\n",
            "        subject: eval_design\n",
        ),
        (
            "drop-keep-capability",
            "    requires:\n      - capability: loop_change.keep\n",
            "",
        ),
        (
            "rejected-drop-within-surface",
            "        - claim: change.within_surface\n        - claim: change.supported\n",
            "        - claim: change.supported\n",
        ),
        (
            "propose-flip",
            "    precondition:\n      claim: change.within_surface\n      is: false\n    requires:",
            "    precondition:\n      claim: change.within_surface\n      is: true\n    requires:",
        ),
    ];
    assert_eq!(survivors(&mutants), Vec::<String>::new());
}

/// RED ON PURPOSE. Each of `change.reject`'s six "not unknown" guards dropped alone: the suite
/// stays green, because the one partial state (`rejected-heldout`/`heldout-regressed`) leaves five
/// keep claims UNKNOWN and any remaining guard blocks. Acceptance 10's "UNKNOWN blocks reject" is
/// tested for one claim combination only.
#[test]
fn egc_reject_guards_are_each_killed_by_a_fixture() {
    let mutants: Vec<(String, String)> = KEEP_CLAIMS
        .iter()
        .map(|claim| {
            (
                format!("reject-drop-unknown-{claim}"),
                format!("        - not: {{claim: {claim}, is: unknown}}\n"),
            )
        })
        .collect();
    // Each guard line occurs in both change.reject and `rejected`; mutate the action's (first).
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(PROTOCOL)).expect("protocol reads");
    let action_start = text.find("  change.reject:\n").expect("change.reject");
    let action_end = text.find("  change.propose:\n").expect("change.propose");
    let action = &text[action_start..action_end];
    let mut surviving = Vec::new();
    for (name, line) in &mutants {
        assert_eq!(action.matches(line.as_str()).count(), 1, "{name}");
        let mutated_action = action.replacen(line.as_str(), "", 1);
        if !suite_kills(name, action, &mutated_action) {
            surviving.push(name.clone());
        }
    }
    assert!(
        surviving.is_empty(),
        "change.reject guards no fixture or check exercises: {surviving:?}"
    );
}

/// RED ON PURPOSE. `rejected`'s five "not unknown" guards dropped, alone or all together: the
/// suite stays green, because every fixture state with `change.supported` FALSE and a keep claim
/// UNKNOWN also has `change.within_surface` UNKNOWN or FALSE, which blocks `rejected` by itself.
/// With all five dropped a leaked guard check alone makes `rejected` legitimate
/// (`egc_leaked_alone_neither_rejects_nor_closes_rejected` tells it apart).
#[test]
fn egc_rejected_outcome_guards_are_each_killed_by_a_fixture() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(PROTOCOL)).expect("protocol reads");
    let outcome_start = text.find("  rejected:\n").expect("rejected");
    let outcome_end = text.find("  proposed:\n").expect("proposed");
    let outcome = &text[outcome_start..outcome_end];
    let guards: Vec<&str> = outcome
        .lines()
        .filter(|line| line.contains("is: unknown}"))
        .collect();
    assert_eq!(guards.len(), 5, "rejected's guards: {guards:?}");
    let mut surviving = Vec::new();
    for guard in &guards {
        let name = format!(
            "rejected-drop-{}",
            guard
                .trim()
                .trim_start_matches("- not: {claim: ")
                .trim_end_matches(", is: unknown}")
        );
        let mutated = outcome.replacen(&format!("{guard}\n"), "", 1);
        if !suite_kills(&name, outcome, &mutated) {
            surviving.push(name);
        }
    }
    let all = guards.iter().fold(outcome.to_owned(), |acc, guard| {
        acc.replacen(&format!("{guard}\n"), "", 1)
    });
    if !suite_kills("rejected-drop-all-unknown-guards", outcome, &all) {
        surviving.push("rejected-drop-all-unknown-guards".to_owned());
    }
    assert!(
        surviving.is_empty(),
        "`rejected` guards no fixture or check exercises: {surviving:?}"
    );
}

/// RED ON PURPOSE. `tuning.improved` read as "the tuning comparison reproduced" instead of "not
/// not_better": the suite stays green, because no fixture records `not_reproduced`
/// (`egc_not_reproduced_is_improved_but_not_reproducible` tells it apart).
#[test]
fn egc_tuning_improved_reading_is_killed_by_a_fixture() {
    let from = "      not:\n        evidence:\n          kind: tuning_comparison\n          \
                result: not_better\n          subject: candidate\n          \
                independent_on: [different_agent_run]\n";
    let to = "      evidence:\n        kind: tuning_comparison\n        result: reproduced\n        \
              subject: candidate\n        independent_on: [different_agent_run]\n";
    assert_eq!(
        survivors(&[("tuning-improved-as-reproduced", from, to)]),
        Vec::<String>::new(),
        "a mutant no fixture or check kills"
    );
}
