//! Adversary pass 2 on `story:takt-protocols-builtin`: mutant classes pass 1 did not try (an
//! evidence match's subject or kind swapped, a capability swapped for another the fixtures also
//! grant, `all` and `any` swapped, a precondition swapped for another claim, an `independent_on`
//! added where the protocol has none, an outcome's result swapped for `error`). Same mechanism as
//! `adversary_takt_protocols.rs`: each mutant is written to cargo's per-target scratch directory,
//! compiled through the harness, and every fixture of the protocol is checked against it.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use std::path::Path;

use support::Fixture;

/// One mutant: what it changes, the text it replaces (exactly once) and the replacement.
type Mutant = (&'static str, &'static str, &'static str);

/// The labels of the mutants of `protocols/<name>/1.yaml` that no fixture under
/// `fixtures/<name>/` tells apart from the protocol.
fn survivors(name: &str, mutants: &[Mutant]) -> Vec<String> {
    let root = support::repo_root();
    let original = std::fs::read_to_string(root.join(support::protocol_path(name, 1)))
        .unwrap_or_else(|error| panic!("{name}: {error}"));
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary2-takt-mutants")
        .join(name);
    std::fs::create_dir_all(&scratch).unwrap_or_else(|error| panic!("{scratch:?}: {error}"));
    let scratch = scratch
        .canonicalize()
        .unwrap_or_else(|error| panic!("{scratch:?}: {error}"));
    let fixtures_dir = root.join("fixtures").join(name);
    let mut fixture_paths: Vec<String> = std::fs::read_dir(&fixtures_dir)
        .unwrap_or_else(|error| panic!("{fixtures_dir:?}: {error}"))
        .map(|entry| entry.expect("a directory entry").file_name())
        .map(|file| file.to_string_lossy().into_owned())
        .filter(|file| file.ends_with(".fixture.yaml"))
        .map(|file| format!("fixtures/{name}/{file}"))
        .collect();
    fixture_paths.sort();
    assert!(!fixture_paths.is_empty(), "{name} has fixtures");
    let fixtures: Vec<Fixture> = fixture_paths
        .iter()
        .map(|path| Fixture::load(path).unwrap_or_else(|error| panic!("{path}: {error}")))
        .collect();
    let original_compiled = support::compile_protocol(&support::protocol_path(name, 1))
        .unwrap_or_else(|error| panic!("{name}: {error}"));
    for fixture in &fixtures {
        assert_eq!(
            fixture.check(&original_compiled),
            Ok(()),
            "{} agrees with the protocol as committed",
            fixture.id()
        );
    }
    let mut alive = Vec::new();
    for (index, (label, from, to)) in mutants.iter().enumerate() {
        assert_eq!(
            original.matches(from).count(),
            1,
            "{name} mutant `{label}`: the text it replaces occurs exactly once"
        );
        let mutated = original.replacen(from, to, 1);
        let file = scratch.join(format!("{index:02}.yaml"));
        std::fs::write(&file, &mutated).unwrap_or_else(|error| panic!("{file:?}: {error}"));
        let relative = file
            .strip_prefix(&root)
            .unwrap_or_else(|_| panic!("{file:?} is inside the repository {root:?}"))
            .to_string_lossy()
            .into_owned();
        let compiled = support::compile_protocol(&relative)
            .unwrap_or_else(|error| panic!("{name} mutant `{label}` compiles: {error}"));
        if fixtures
            .iter()
            .all(|fixture| fixture.check(&compiled).is_ok())
        {
            alive.push((*label).to_owned());
        }
    }
    alive
}

fn assert_killed(name: &str, mutants: &[Mutant]) {
    let alive = survivors(name, mutants);
    assert!(
        alive.is_empty(),
        "{} of {} mutants of {name}@1 pass every fixture under fixtures/{name}/:\n- {}",
        alive.len(),
        mutants.len(),
        alive.join("\n- ")
    );
}

const STORY_DELIVERY: &[Mutant] = &[
    (
        "tests.pass about the plan instead of the implementation",
        "        kind: test_result\n        result: pass\n        subject: implementation\n",
        "        kind: test_result\n        result: pass\n        subject: plan\n",
    ),
    (
        "gate.green read from test_result",
        "        kind: gate_result\n        result: pass\n",
        "        kind: test_result\n        result: pass\n",
    ),
    (
        "ci.green read from gate_result",
        "        kind: ci_result\n        result: pass\n",
        "        kind: gate_result\n        result: pass\n",
    ),
    (
        "change.handover precondition `is: false`",
        "      claim: change.handed_over\n      is: unknown\n",
        "      claim: change.handed_over\n      is: false\n",
    ),
    (
        "repository.merge needs repository.merge.judge_surface",
        "      - capability: repository.merge\n",
        "      - capability: repository.merge.judge_surface\n",
    ),
    (
        "pr.open precondition `any` instead of `all`",
        "    precondition:\n      all:\n        - claim: tests.pass\n",
        "    precondition:\n      any:\n        - claim: tests.pass\n",
    ),
    (
        "repository.merge precondition `any` instead of `all`",
        "    precondition:\n      all:\n        - claim: ci.green\n",
        "    precondition:\n      any:\n        - claim: ci.green\n",
    ),
    (
        "ci.read waits for ci.green instead of pr.opened",
        "    precondition:\n      claim: pr.opened\n    may_produce:\n      - evidence: ci_result\n",
        "    precondition:\n      claim: ci.green\n    may_produce:\n      - evidence: ci_result\n",
    ),
    (
        "tests.run waits for a handover",
        "    description: run the tests against an implementation revision\n    effect: read\n",
        "    description: run the tests against an implementation revision\n    effect: read\n    precondition:\n      claim: change.handed_over\n",
    ),
    (
        "pr.open waits for ci.green instead of change.made",
        "        - claim: change.handed_over\n        - claim: change.made\n",
        "        - claim: change.handed_over\n        - claim: ci.green\n",
    ),
];

const SELF_IMPROVEMENT: &[Mutant] = &[
    (
        "scenario.reproduced_before about the candidate",
        "        kind: scenario_result\n        result: reproduced\n        subject: baseline\n",
        "        kind: scenario_result\n        result: reproduced\n        subject: candidate\n",
    ),
    (
        "promoted about the candidate instead of the promotion",
        "        result: promoted\n        subject: promotion\n",
        "        result: promoted\n        subject: candidate\n",
    ),
    (
        "fix.merged about the baseline",
        "        kind: merge_observation\n        result: merged\n        subject: candidate\n",
        "        kind: merge_observation\n        result: merged\n        subject: baseline\n",
    ),
    (
        "candidate.merge needs takt.promote instead of repository.merge",
        "      - capability: repository.merge\n",
        "      - capability: takt.promote\n",
    ),
    (
        "binary.promote needs repository.merge instead of takt.promote",
        "      - capability: takt.promote\n",
        "      - capability: repository.merge\n",
    ),
    (
        "candidate.merge only on a judge surface",
        "          in: [non_judge]\n",
        "          in: [judge]\n",
    ),
    (
        "accepted on `any` of its three claims",
        "    requires:\n      all:\n        - claim: scenario.passes_after\n",
        "    requires:\n      any:\n        - claim: scenario.passes_after\n",
    ),
    (
        "declined on `error` instead of `reproduced`",
        "        result: reproduced\n        subject: candidate\n",
        "        result: error\n        subject: candidate\n",
    ),
    (
        "unreproducible on `error` instead of `not_reproduced`",
        "        result: not_reproduced\n        subject: baseline\n",
        "        result: error\n        subject: baseline\n",
    ),
    (
        "binary.promote waits for scenario.passes_after instead of fix.merged",
        "        - claim: fix.merged\n        - claim: scorecard.no_regression\n",
        "        - claim: scenario.passes_after\n        - claim: scorecard.no_regression\n",
    ),
    (
        "scorecard.compare waits for scenario.passes_after instead of fix.merged",
        "    precondition:\n      claim: fix.merged\n",
        "    precondition:\n      claim: scenario.passes_after\n",
    ),
    (
        "scenario.reproduced_before needs an independent producer",
        "        kind: scenario_result\n        result: reproduced\n        subject: baseline\n",
        "        kind: scenario_result\n        result: reproduced\n        subject: baseline\n        independent_on: [different_agent_run]\n",
    ),
    (
        "scorecard.no_regression needs an independent producer",
        "        result: no_regression\n        subject: candidate\n",
        "        result: no_regression\n        subject: candidate\n        independent_on: [different_agent_run]\n",
    ),
    (
        "fix.merged read from scenario_result",
        "        kind: merge_observation\n        result: merged\n",
        "        kind: scenario_result\n        result: merged\n",
    ),
];

const LOOP_IMPROVEMENT: &[Mutant] = &[
    (
        "loop.metric_below_target about the candidate",
        "        result: below_target\n        subject: loop_baseline\n",
        "        result: below_target\n        subject: loop_candidate\n",
    ),
    (
        "loop.change_merged about the baseline",
        "        result: merged\n        subject: loop_candidate\n",
        "        result: merged\n        subject: loop_baseline\n",
    ),
    (
        "loop.metric_improved about the baseline",
        "        result: improved\n        subject: loop_candidate\n",
        "        result: improved\n        subject: loop_baseline\n",
    ),
    (
        "declined about the baseline",
        "        result: not_improved\n        subject: loop_candidate\n",
        "        result: not_improved\n        subject: loop_baseline\n",
    ),
    (
        "loop.definition_honest on `all` of its two results",
        "    true_when:\n      any:\n",
        "    true_when:\n      all:\n",
    ),
    (
        "accepted on `any` of its three claims",
        "    requires:\n      all:\n        - claim: loop.change_merged\n",
        "    requires:\n      any:\n        - claim: loop.change_merged\n",
    ),
    (
        "loop.compare waits for loop.definition_honest instead of loop.change_merged",
        "    precondition:\n      claim: loop.change_merged\n",
        "    precondition:\n      claim: loop.definition_honest\n",
    ),
    (
        "loop.merge waits for loop.metric_below_target instead of loop.definition_honest",
        "    precondition:\n      claim: loop.definition_honest\n",
        "    precondition:\n      claim: loop.metric_below_target\n",
    ),
    (
        "loop.metric_improved read from loop_measurement",
        "        kind: loop_comparison\n        result: improved\n",
        "        kind: loop_measurement\n        result: improved\n",
    ),
    (
        "loop.change_merged needs an independent producer",
        "        result: merged\n        subject: loop_candidate\n",
        "        result: merged\n        subject: loop_candidate\n        independent_on: [different_agent_run]\n",
    ),
];

#[test]
fn adversary2_story_delivery_fixtures_kill_every_new_mutant_class() {
    assert_killed("story-delivery", STORY_DELIVERY);
}

#[test]
fn adversary2_self_improvement_fixtures_kill_every_new_mutant_class() {
    assert_killed("self-improvement", SELF_IMPROVEMENT);
}

#[test]
fn adversary2_loop_improvement_fixtures_kill_every_new_mutant_class() {
    assert_killed("loop-improvement", LOOP_IMPROVEMENT);
}
