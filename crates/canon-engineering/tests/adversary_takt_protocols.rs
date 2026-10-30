//! Adversary pass 1 on `story:takt-protocols-builtin`.
//!
//! The protocol tests compare the built-in with the file on disk and with takt's blob id, so any
//! edit of a committed protocol is red for that reason alone. These cases ask the question the
//! byte check cannot: do the fixtures under `fixtures/<name>/` pin each rule of the protocol? Each
//! mutant below is one rule changed (a precondition dropped, an `independent_on` dropped, a result
//! value swapped, a `requires: capability` dropped); it is compiled from its text through the
//! harness, with no file written, and every fixture of the protocol is checked
//! against it. A mutant every fixture still agrees with is a rule no fixture exercises.
//!
//! The last case holds the O1 line of `AGENTS.md` § Serves for every built-in.

#[allow(dead_code)] // uses part of the harness; fixture_harness.rs uses all of it
mod support;

use support::Fixture;

/// One mutant: what it changes, the text it replaces (exactly once) and the replacement.
type Mutant = (&'static str, &'static str, &'static str);

/// The labels of the mutants of `protocols/<name>/1.yaml` that no fixture under
/// `fixtures/<name>/` tells apart from the protocol.
fn survivors(name: &str, mutants: &[Mutant]) -> Vec<String> {
    let root = support::repo_root();
    let original = std::fs::read_to_string(root.join(support::protocol_path(name, 1)))
        .unwrap_or_else(|error| panic!("{name}: {error}"));
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
    for (label, from, to) in mutants {
        assert_eq!(
            original.matches(from).count(),
            1,
            "{name} mutant `{label}`: the text it replaces occurs exactly once"
        );
        let mutated = original.replacen(from, to, 1);
        let compiled = support::compile_protocol_text(&mutated)
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
        "repository.merge requires no capability",
        "    requires:\n      - capability: repository.merge\n",
        "",
    ),
    (
        "pr.open without change.made",
        "        - claim: change.handed_over\n        - claim: change.made\n",
        "        - claim: change.handed_over\n",
    ),
    (
        "pr.open without change.handed_over",
        "        - claim: gate.green\n        - claim: change.handed_over\n",
        "        - claim: gate.green\n",
    ),
    (
        "pr.open without tests.pass",
        "        - claim: tests.pass\n        - claim: gate.green\n",
        "        - claim: gate.green\n",
    ),
    (
        "pr.open without gate.green",
        "        - claim: gate.green\n        - claim: change.handed_over\n",
        "        - claim: change.handed_over\n",
    ),
    (
        "repository.merge without pr.opened",
        "        - claim: ci.green\n        - claim: pr.opened\n",
        "        - claim: ci.green\n",
    ),
    (
        "repository.merge without ci.green",
        "        - claim: ci.green\n        - claim: pr.opened\n",
        "        - claim: pr.opened\n",
    ),
    (
        "change.handover may repeat after a handover",
        "    precondition:\n      claim: change.handed_over\n      is: unknown\n",
        "",
    ),
    (
        "change.read before a handover",
        "    precondition:\n      claim: change.handed_over\n    may_produce:\n      - evidence: change_record\n",
        "    may_produce:\n      - evidence: change_record\n",
    ),
    (
        "ci.read without an open pull request",
        "    precondition:\n      claim: pr.opened\n    may_produce:\n      - evidence: ci_result\n",
        "    may_produce:\n      - evidence: ci_result\n",
    ),
    (
        "accepted on ci.green instead of pr.merged",
        "    requires:\n      claim: pr.merged\n",
        "    requires:\n      claim: ci.green\n",
    ),
    (
        "abandoned on another decision",
        "      decision: explicitly_abandoned\n",
        "      decision: explicitly_declined\n",
    ),
    (
        "change.made true on `none`",
        "        result: ahead\n",
        "        result: none\n",
    ),
    (
        "tests.pass true on `fail`",
        "        kind: test_result\n        result: pass\n",
        "        kind: test_result\n        result: fail\n",
    ),
    (
        "gate.green true on `fail`",
        "        kind: gate_result\n        result: pass\n",
        "        kind: gate_result\n        result: fail\n",
    ),
    (
        "ci.green true on `fail`",
        "        kind: ci_result\n        result: pass\n",
        "        kind: ci_result\n        result: fail\n",
    ),
    (
        "pr.opened true on `closed`",
        "        result: open\n",
        "        result: closed\n",
    ),
    (
        "pr.merged true on `open`",
        "        result: merged\n",
        "        result: open\n",
    ),
    (
        "merge.required discharged by ci.green",
        "    discharged_when:\n      claim: pr.merged\n",
        "    discharged_when:\n      claim: ci.green\n",
    ),
    (
        "tests.required discharged by gate.green",
        "    discharged_when:\n      claim: tests.pass\n",
        "    discharged_when:\n      claim: gate.green\n",
    ),
];

const SELF_IMPROVEMENT: &[Mutant] = &[
    (
        "scenario.passes_after without independent_on",
        "        independent_on: [different_agent_run]\n",
        "",
    ),
    (
        "candidate.merge on a judge surface too",
        "          in: [non_judge]\n",
        "          in: [judge, non_judge]\n",
    ),
    (
        "candidate.merge_judge_surface on a non-judge surface too",
        "          in: [judge]\n",
        "          in: [judge, non_judge]\n",
    ),
    (
        "candidate.merge requires no capability",
        "    requires:\n      - capability: repository.merge\n",
        "",
    ),
    (
        "candidate.merge_judge_surface needs only repository.merge",
        "      - capability: repository.merge.judge_surface\n",
        "      - capability: repository.merge\n",
    ),
    (
        "binary.promote requires no capability",
        "    requires:\n      - capability: takt.promote\n",
        "",
    ),
    (
        "binary.promote without scorecard.no_regression",
        "        - claim: fix.merged\n        - claim: scorecard.no_regression\n",
        "        - claim: fix.merged\n",
    ),
    (
        "binary.promote without fix.merged",
        "        - claim: fix.merged\n        - claim: scorecard.no_regression\n",
        "        - claim: scorecard.no_regression\n",
    ),
    (
        "scorecard.compare without fix.merged",
        "    precondition:\n      claim: fix.merged\n",
        "",
    ),
    (
        "scenario.verify without scenario.reproduced_before",
        "    precondition:\n      claim: scenario.reproduced_before\n",
        "",
    ),
    (
        "scenario.reproduce without finding.cited",
        "    precondition:\n      claim: finding.cited\n",
        "",
    ),
    (
        "candidate.merge without scenario.passes_after",
        "        - claim: scenario.passes_after\n        - input: surface\n          in: [non_judge]\n",
        "        - input: surface\n          in: [non_judge]\n",
    ),
    (
        "candidate.merge_judge_surface without scenario.passes_after",
        "        - claim: scenario.passes_after\n        - input: surface\n          in: [judge]\n",
        "        - input: surface\n          in: [judge]\n",
    ),
    (
        "accepted without promoted",
        "        - claim: scorecard.no_regression\n        - claim: promoted\n",
        "        - claim: scorecard.no_regression\n",
    ),
    (
        "accepted without scorecard.no_regression",
        "        - claim: scorecard.no_regression\n        - claim: promoted\n",
        "        - claim: promoted\n",
    ),
    (
        "accepted without scenario.passes_after",
        "        - claim: scenario.passes_after\n        - claim: scorecard.no_regression\n",
        "        - claim: scorecard.no_regression\n",
    ),
    (
        "declined on the baseline instead of the candidate",
        "        result: reproduced\n        subject: candidate\n",
        "        result: reproduced\n        subject: baseline\n",
    ),
    (
        "unreproducible on the candidate instead of the baseline",
        "        result: not_reproduced\n        subject: baseline\n",
        "        result: not_reproduced\n        subject: candidate\n",
    ),
    (
        "finding.cited true on `malformed`",
        "        result: filed\n",
        "        result: malformed\n",
    ),
    (
        "scorecard.no_regression true on `regression`",
        "        result: no_regression\n",
        "        result: regression\n",
    ),
    (
        "promoted true on `mismatch`",
        "        result: promoted\n",
        "        result: mismatch\n",
    ),
    (
        "scenario.passes_after true on `reproduced`",
        "        result: not_reproduced\n        subject: candidate\n",
        "        result: reproduced\n        subject: candidate\n",
    ),
    (
        "fix.merged true on `open`",
        "        result: merged\n",
        "        result: open\n",
    ),
    (
        "verification.required discharged by scenario.reproduced_before",
        "    discharged_when:\n      claim: scenario.passes_after\n",
        "    discharged_when:\n      claim: scenario.reproduced_before\n",
    ),
    (
        "regression.required discharged by fix.merged",
        "    discharged_when:\n      claim: scorecard.no_regression\n",
        "    discharged_when:\n      claim: fix.merged\n",
    ),
    (
        "abandoned on another decision",
        "      decision: explicitly_abandoned\n",
        "      decision: explicitly_declined\n",
    ),
];

const LOOP_IMPROVEMENT: &[Mutant] = &[
    (
        "loop.metric_improved without independent_on",
        "        independent_on: [different_agent_run]\n",
        "",
    ),
    (
        "loop.merge requires no capability",
        "    requires:\n      - capability: repository.merge.judge_surface\n",
        "",
    ),
    (
        "loop.merge needs only repository.merge",
        "      - capability: repository.merge.judge_surface\n",
        "      - capability: repository.merge\n",
    ),
    (
        "loop.merge without loop.definition_honest",
        "    precondition:\n      claim: loop.definition_honest\n",
        "",
    ),
    (
        "loop.compare without loop.change_merged",
        "    precondition:\n      claim: loop.change_merged\n",
        "",
    ),
    (
        "loop.finding.file without loop.metric_below_target",
        "file the lowest metric as a finding against the loop\n    effect: write\n    precondition:\n      claim: loop.metric_below_target\n",
        "file the lowest metric as a finding against the loop\n    effect: write\n",
    ),
    (
        "loop.change.deliver without loop.metric_below_target",
        "a protocol or an agent profile\n    effect: write\n    precondition:\n      claim: loop.metric_below_target\n",
        "a protocol or an agent profile\n    effect: write\n",
    ),
    (
        "loop.definition_honest not on `dual_reported`",
        "        - evidence:\n            kind: definition_check\n            result: dual_reported\n            subject: loop_candidate\n",
        "",
    ),
    (
        "loop.definition_honest not on `unchanged`",
        "        - evidence:\n            kind: definition_check\n            result: unchanged\n            subject: loop_candidate\n",
        "",
    ),
    (
        "loop.definition_honest true on `redefined`",
        "            result: dual_reported\n",
        "            result: redefined\n",
    ),
    (
        "accepted without loop.definition_honest",
        "        - claim: loop.change_merged\n        - claim: loop.definition_honest\n        - claim: loop.metric_improved\n",
        "        - claim: loop.change_merged\n        - claim: loop.metric_improved\n",
    ),
    (
        "accepted without loop.change_merged",
        "        - claim: loop.change_merged\n        - claim: loop.definition_honest\n        - claim: loop.metric_improved\n",
        "        - claim: loop.definition_honest\n        - claim: loop.metric_improved\n",
    ),
    (
        "accepted without loop.metric_improved",
        "        - claim: loop.change_merged\n        - claim: loop.definition_honest\n        - claim: loop.metric_improved\n",
        "        - claim: loop.change_merged\n        - claim: loop.definition_honest\n",
    ),
    (
        "declined on `improved`",
        "        result: not_improved\n",
        "        result: improved\n",
    ),
    (
        "loop.metric_improved true on `not_improved`",
        "        result: improved\n",
        "        result: not_improved\n",
    ),
    (
        "loop.metric_below_target true on `at_target`",
        "        result: below_target\n",
        "        result: at_target\n",
    ),
    (
        "loop.change_merged true on `open`",
        "        result: merged\n",
        "        result: open\n",
    ),
    (
        "honesty.required discharged by loop.change_merged",
        "    discharged_when:\n      claim: loop.definition_honest\n",
        "    discharged_when:\n      claim: loop.change_merged\n",
    ),
    (
        "improvement.required discharged by loop.change_merged",
        "    discharged_when:\n      claim: loop.metric_improved\n",
        "    discharged_when:\n      claim: loop.change_merged\n",
    ),
    (
        "abandoned on another decision",
        "      decision: explicitly_abandoned\n",
        "      decision: explicitly_declined\n",
    ),
];

#[test]
fn adversary_story_delivery_fixtures_kill_every_rule_mutant() {
    assert_killed("story-delivery", STORY_DELIVERY);
}

#[test]
fn adversary_self_improvement_fixtures_kill_every_rule_mutant() {
    assert_killed("self-improvement", SELF_IMPROVEMENT);
}

#[test]
fn adversary_loop_improvement_fixtures_kill_every_rule_mutant() {
    assert_killed("loop-improvement", LOOP_IMPROVEMENT);
}

/// `AGENTS.md` § Serves, O1: every write action a protocol declares names the capability that
/// authorises it, except `software.change/1`'s `tests.write` and `implementation.edit` and the
/// six write actions of the protocols moved from takt that AGENTS.md lists. Walked over every
/// built-in the registry lists; `EXCEPTED` must match that AGENTS.md list.
///
/// The three protocols moved from takt are takt 0.12.0's files but for wording agreed with takt, so their six write
/// actions without a capability are an open defect, asserted here in its current state in both
/// directions: a seventh fails, and so does one of the six gaining a capability.
#[test]
fn adversary_every_builtin_write_action_names_its_capability() {
    const EXCEPTED: [(&str, &str); 8] = [
        ("software-change", "tests.write"),
        ("software-change", "implementation.edit"),
        ("story-delivery", "implementation.edit"),
        ("story-delivery", "pr.open"),
        ("story-delivery", "tests.write"),
        ("self-improvement", "finding.file"),
        ("loop-improvement", "loop.change.deliver"),
        ("loop-improvement", "loop.finding.file"),
    ];
    let mut uncapable = Vec::new();
    for (name, major) in canon_engineering::registry::list() {
        let builtin = canon_engineering::registry::get(name, major)
            .unwrap_or_else(|error| panic!("registry::get({name}, {major}): {error}"));
        let ir = b10x_canon::ir::compile(&builtin.model)
            .unwrap_or_else(|problems| panic!("{name}@{major} compiles: {problems:?}"));
        for (id, action) in &ir.actions {
            let write = action
                .effect
                .as_ref()
                .is_some_and(|effect| effect.as_str() == "write");
            if write && action.requires.is_empty() {
                uncapable.push((name, id.as_str().to_owned()));
            }
        }
    }
    let unauthorised: Vec<String> = uncapable
        .iter()
        .filter(|(name, id)| !EXCEPTED.contains(&(*name, id.as_str())))
        .map(|(name, id)| format!("{name} `{id}`"))
        .collect();
    assert!(
        unauthorised.is_empty(),
        "write actions that name no capability, outside AGENTS.md's two exceptions and the six \
         open under story:takt-protocol-write-capabilities:\n- {}",
        unauthorised.join("\n- ")
    );
    let resolved: Vec<String> = EXCEPTED
        .iter()
        .filter(|(name, id)| {
            !uncapable
                .iter()
                .any(|(found, action)| found == name && action == id)
        })
        .map(|(name, id)| format!("{name} `{id}`"))
        .collect();
    assert!(
        resolved.is_empty(),
        "excepted write actions that now name a capability or no longer exist; \
         story:takt-protocol-write-capabilities: remove the entry from EXCEPTED when the \
         protocol changes:\n- {}",
        resolved.join("\n- ")
    );
}
