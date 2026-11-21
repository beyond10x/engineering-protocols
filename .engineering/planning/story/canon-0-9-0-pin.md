---
format: aep.planning-md/3
id: story:canon-0-9-0-pin
kind: story
status: implemented
title: Move from Canon 0.8.0 to Canon 0.9.0 and release 0.16.0
summary: b10x-canon and b10x-canon-expr pin Canon tag 0.9.0, the workspace compiles and gates on it, and the move is released as 0.16.0.
relations:
- decomposes: epic:els-first-domain
- serves: vision:O2
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T23:42:08Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-20T23:42:15Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-20T23:42:15Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`b10x-canon` and `b10x-canon-expr` move from Canon tag 0.8.0 (cf171757) to Canon tag 0.9.0
(8f1f4527439b813f4c0c0159c8b4ea7cd4131994, Canon `main` on 2026-11-21), and the move is released
as Engineering Protocols 0.16.0.

## Context

Loom passes Canon values across `b10x-canon-engineering` (`registry::get` returns Canon's model
type), and three Loom tests refuse a second Canon copy in its dependency graph. Loom therefore
cannot move to Canon 0.9.0 before a release of this repository is built on it. The request reached
this repository on 2026-11-21 from the `sdc` session on Timo's instruction, with the facts read by
the `loom` session.

Canon 0.9.0 adds child cases (Atlas ADR 0081): an action's `case_open`, the predicate form
`{children: {in: [...]}}` and the evidence kind `case_outcome`. For Rust consumers `Predicate`
gains `Children(ChildOutcomes)`, `Action` gains `case_open` and `EvidenceRecord` gains
`child_case`. Its ESS specification requires ESS 0.77.0, which this repository already pins.

## Scope

- `crates/canon-engineering/Cargo.toml`, `crates/canon-engineering-assertions/Cargo.toml`,
  `crates/canon-engineering-docs/Cargo.toml`: `tag = "0.9.0"`; `Cargo.lock` records 8f1f4527.
- `crates/canon-engineering-docs/src/graph.rs`, `src/protocol.rs`: the four exhaustive matches
  over `Predicate` read `Children`.
- `crates/canon-engineering-docs/src/generate.rs`: generation refuses a protocol with `case_open`
  or a `children` form, naming the file and the construct. `b10x-protocol-graph/1` in the pinned
  docs-system 0.14.0, and in docs-system `main` at 081e05f, has no key for `case_open` and no
  `children` leaf, so a graph or page written for such a protocol would leave the construct out.
- `crates/canon-engineering/tests/`: the reach walks of `software_change_protocol.rs`,
  `incident_response_protocol.rs`, `adversary_software_change_protocol.rs` and
  `support_triage_protocol.rs` count a `children` form as reading `case_outcome`;
  `crates/canon-engineering-docs/tests/adversary_graph_case_inputs.rs` reads the new variant.
- Release 0.16.0: workspace version, `Cargo.lock`, README install lines and latest-release line,
  CHANGELOG.

## Not in scope

- No built-in protocol uses `case_open`, a `children` form or `case_outcome`. A protocol that
  does needs a docs-system release whose graph schema draws them, and the generator change that
  follows it; neither is planned here.
- Moving Loom, takt or any other consumer to this release.

## Protocol first

Exempt: no protocol YAML and no fixture expectation changes, and every fixture verdict is the one
0.15.2 gives. The generator's new refusal is covered by
`generate::tests::a_protocol_with_a_child_case_construct_is_refused_with_its_path`.

## Acceptance

1. `Cargo.lock` resolves exactly one `b10x-canon` and one `b10x-canon-expr`, both from
   `git+https://github.com/beyond10x/canon?tag=0.9.0#8f1f4527439b813f4c0c0159c8b4ea7cd4131994`.
2. `task check` is green on the tree: fmt, clippy with `-D warnings`, the workspace tests, and
   `canon-engineering-docs generate --check` with no generated file changed.

## Release

After this story: the pull request's `check`, `Documentation validation` and `Shared source gates`
runs green, the merge to `main`, tag `0.16.0` by `b10x-bot[bot]` on the release commit and its
GitHub Release (AGENTS.md § Releases).

## Verification

`ESS_TOOLCHAIN=0.77.0 slot task check` on 2026-11-20T23:41:37Z, rustc 1.99.1: exit 0, 99 test
result lines, 421 passed, 0 failed, 0 ignored, 35 generated files fresh (10 protocols). The evidence
record names `green.txt`, the summary of that run; it is not committed, because `aep plan artifact
validate` refuses a file under `.engineering/evidence/` that is not a record. The workstation's
`ess` is 0.96.0 and delegates to the pinned 0.77.0 only with `ESS_TOOLCHAIN` set; without it the
two drift tests of `ess_contract.rs` fail on the refusal `requires ess 0.77.0 and this is ess 0.96.0`,
which names `ess/ess-inputs.yaml` and `ess/providers/ess-inputs.yaml`; this story changes neither.
