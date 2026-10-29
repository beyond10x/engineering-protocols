//! Acceptance 3 of `story:takt-protocols-builtin` (Atlas ADR 0094 rule 5): Canon's `check` runs
//! on every built-in, finds nothing, and evaluates at most `STATE_BOUND` states.
//!
//! The built-ins are read from `registry::list` at run time, so a protocol file added later is
//! checked without an edit here. A built-in that does not hold today is named in [`OVER_BOUND`]
//! with its current measurement, and the test asserts that measurement exactly, so it goes red
//! when the protocol or Canon changes it: the entry is then corrected or removed, never left
//! stale. Each built-in is checked on its own thread; a check of tens of thousands of states takes
//! tens of seconds in a debug build.

use b10x_canon::check::STATE_BOUND;
use canon_engineering::registry;

/// The built-ins whose state space is over `STATE_BOUND`, each with the state count Canon 0.7.0's
/// `state-space-bound` refusal names. `self.improvement/1` is at the count Atlas ADR 0094
/// § Consequences records; splitting it, or a Canon change, is its own story. `ml.experiment/1`
/// and `software.change/1` were over the bound before ADR 0094 and are measured here as they are.
/// `story.delivery/2` is over it by its two case inputs, the order form over `tests_change`,
/// `paths_changed` and the `after` form over a failed `test_result`
/// (story:story-delivery-2-red-gate-needs-red-tests); `story.delivery/1` stays under it
/// (`story:story-delivery-2-with-settings`).
const OVER_BOUND: &[(&str, u32, u128)] = &[
    ("ml-experiment", 1, 4_398_046_511_104),
    ("self-improvement", 1, 5_971_968),
    ("software-change", 1, 10_192_158_720),
    ("story-delivery", 2, 3_067_084_800),
];

/// Checks the built-in `name@major` and returns what differs from the rule, or from its
/// [`OVER_BOUND`] entry, and the measurement as one line.
fn check(name: &str, major: u32) -> (Vec<String>, String) {
    let builtin = registry::get(name, major)
        .unwrap_or_else(|error| panic!("registry::get({name}, {major}): {error}"));
    let ir = b10x_canon::ir::compile(&builtin.model)
        .unwrap_or_else(|problems| panic!("{name}@{major} compiles: {problems:?}"));
    let recorded = OVER_BOUND
        .iter()
        .find(|(over, over_major, _)| *over == name && *over_major == major)
        .map(|(_, _, states)| *states);
    let mut differences = Vec::new();
    let measurement = match (b10x_canon::check::check(&ir, None), recorded) {
        (Ok(report), None) => {
            if !report.is_clean() {
                differences.push(format!("{name}@{major}: canon check found:\n{report}"));
            }
            if report.states > STATE_BOUND {
                differences.push(format!(
                    "{name}@{major}: {} states, more than the bound of {STATE_BOUND}",
                    report.states
                ));
            }
            format!(
                "{name}@{major}: {} states, {} findings",
                report.states,
                report.findings.len()
            )
        }
        (Ok(report), Some(states)) => {
            differences.push(format!(
                "{name}@{major} is listed over the bound at {states} states, but checks at {} \
                 states with {} findings: correct OVER_BOUND",
                report.states,
                report.findings.len()
            ));
            format!("{name}@{major}: {} states", report.states)
        }
        (Err(refusal), recorded) => {
            let expected = recorded
                .map(|states| format!("has {states} states, more than the bound of {STATE_BOUND}"));
            let holds = refusal.code() == "state-space-bound"
                && expected
                    .as_ref()
                    .is_some_and(|expected| refusal.to_string().contains(expected.as_str()));
            if !holds {
                differences.push(format!(
                    "{name}@{major}: canon check refused ({}), recorded over the bound at \
                     {recorded:?} states: {refusal}",
                    refusal.code()
                ));
            }
            format!("{name}@{major}: refused, {}: {refusal}", refusal.code())
        }
    };
    (differences, measurement)
}

#[test]
fn every_builtin_checks_clean_under_the_bound() {
    let listed = registry::list();
    assert!(!listed.is_empty(), "the registry lists no built-in");
    for (name, major, _) in OVER_BOUND {
        assert!(
            listed.contains(&(*name, *major)),
            "OVER_BOUND names {name}@{major}, which the registry does not list: {listed:?}"
        );
    }
    let results: Vec<(Vec<String>, String)> = std::thread::scope(|scope| {
        let checks: Vec<_> = listed
            .iter()
            .map(|&(name, major)| scope.spawn(move || check(name, major)))
            .collect();
        checks
            .into_iter()
            .map(|check| check.join().expect("a check thread panicked"))
            .collect()
    });
    let mut differences = Vec::new();
    for (found, measurement) in results {
        println!("{measurement}");
        differences.extend(found);
    }
    assert!(
        differences.is_empty(),
        "canon check over the built-ins:\n{}",
        differences.join("\n")
    );
}
