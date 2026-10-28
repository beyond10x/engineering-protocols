# AGENTS.md — engineering-protocols

This file is for an agent changing the repository. What engineering protocols are, how to depend on
them and how to run them is in [README.md](README.md); the public site is
<https://beyond10x.github.io/engineering-protocols/>. The cross-repository architecture is Atlas
ADRs 0066–0075 and 0094, and Atlas `docs/design/governed-autonomy/`.

## Serves

- **O2 — decisions as data, with evidence.** Engineering rules are Canon protocols declared as
  data and evaluated against cited, revision-bound evidence, with `UNKNOWN` kept apart from `FALSE`.
- **O1 — governed reach.** Every write action a protocol declares names the capability that
  authorises it, except `software.change/1`'s `tests.write` and `implementation.edit` (telling
  them apart needs a path scope on capabilities, which Canon does not have), and the six write
  actions of the protocols moved from takt byte for byte that name none: `story.delivery/1`'s
  `tests.write`, `implementation.edit` and `pr.open`, `self.improvement/1`'s `finding.file`,
  `loop.improvement/1`'s `loop.change.deliver` and `loop.finding.file`
  (`story:takt-protocol-write-capabilities`). Loom's `loom-governor` consumes these protocols.

## Boundary

This repository owns engineering-domain vocabulary and the protocols written in it (Atlas ADR
0068): `software.change/1`, `incident.response/1`, `ml.experiment/1` and `eval.gated_change/1`
today, investigation, migration and security response later. `support.triage/1` is the one
protocol outside engineering, placed here by operator decision of 2026-10-05 as the rules half of
a support triage example; its terms stay out of `protocols/vocabulary.yaml`, and its actions are tool-agnostic like every other
protocol's.

Every spec-level protocol a runtime uses lives here (Atlas ADR 0094): `story.delivery/1`,
`self.improvement/1` and `loop.improvement/1` came from takt byte for byte (takt tag 0.12.0) and
must stay identical to it until a new major is agreed with takt. A runtime pins a release of
`b10x-canon-engineering` and reads them through `registry::get`; runtime data that is not a
protocol (takt's `loop-improvement/metrics.yaml`) stays in the runtime. Their terms are in the
vocabulary, marked `software.change`, `self.improvement` or `loop.improvement` when they mean
nothing outside that domain or protocol.

It does not own:

| Not here | Owner |
|---|---|
| Claim, evidence and obligation semantics; validation, compilation, evaluation | Canon. Do not re-implement them here. |
| The live engineering record | AEP (Atlas ADR 0069) |
| Running agents | Loom; its `loom-governor`, `loom-intake-router` and `loom-intake-slice` crates depend on this crate at tag 0.1.0 |

Pressure-test every abstraction against both software delivery and incident response. A rule that
only makes sense for Git, pull requests or code is not necessarily a core rule of engineering
protocols.

## Invariants

| A change must keep | Held by |
|---|---|
| Every `protocols/<name>/<major>.yaml` is embedded by `crates/canon-engineering/build.rs` with no Rust edit, and each one parses and validates in Canon | `tests/protocol_registry.rs::registry_lists_fetches_and_validates_every_builtin` |
| A malformed protocol path under `protocols/` fails the build rather than being skipped | `build.rs`, rule in `src/builtin_name.rs` |
| `UNKNOWN` is not `FALSE`: tests on a stale revision leave `tests.pass` `UNKNOWN` and merge blocked | `tests/software_change_protocol.rs::chg_1842_merge_waits_for_current_revision_tests_and_authority` |
| A profile is a case input, not a protocol: one compiled `software.change/1` serves every risk, and each risk's open obligations strictly contain those of the risk below | `tests/software_change_profiles.rs::profiles_nest_open_obligations` |
| A change on a security boundary is verified only on an approving security review by a producer the case lists as independent on `different_principal`; the implementer's own approval is excluded as `not_independent` and leaves `security.reviewed` `UNKNOWN`, not `FALSE` | `tests/security_independence_rules.rs::independent_security_review_gates_implementation_verified` |
| An incident rollback is verified only on a complete `rollback_result` from a producer the case lists as independent on `different_principal`; the result of the actor who rolled back is excluded as `not_independent` and leaves `rollback.verified` `UNKNOWN`, not `FALSE` | `tests/rollback_verification_rule.rs::rollback_verified_needs_an_independent_verifier` |
| Stale evidence is `UNKNOWN`, not `FALSE`: a review of a revision the implementation no longer has leaves `implementation.reviewed` `UNKNOWN`, and an operational observation more than 30 minutes old at the evaluation instant, a fixture input the harness passes to Canon for each state, leaves `deployment.healthy` or `service.healthy` `UNKNOWN` | `tests/stale_evidence.rs::stale_evidence_is_unknown_and_fresh_evidence_holds` |
| An incident leaves emergency mode while its cause is still `UNKNOWN` | `tests/incident_response_protocol.rs::inc_492_leaves_emergency_while_cause_unknown` |
| A hypothesis is refuted only by a comparison that is `FALSE`: a missing comparison, or one about another candidate revision, leaves `refuted` blocked | `tests/ml_experiment_protocol.rs::ml_experiment_outcomes_are_earned` |
| An eval-gated change is kept only on evidence about the current candidate revision from a producer the case lists as independent on `different_agent_run`; evidence by the run that wrote the candidate is excluded as `not_independent`, and evidence about an earlier candidate revision (a moved basis) as `revision_mismatch`, each leaving its claim `UNKNOWN`, not `FALSE`, so neither `change.keep` nor `change.reject` is admissible | `tests/eval_gated_change_protocol.rs::eval_gated_change_outcomes_are_earned` |
| `eval.gated_change/1` checks under Canon's `STATE_BOUND` with no findings | `tests/eval_gated_change_protocol.rs::eval_gated_change_check_stays_under_the_bound` |
| `support.triage/1` has no unreachable outcome and no authority bypass | `tests/support_triage_protocol.rs::canon_check_finds_no_unreachable_outcome_and_no_authority_bypass` |
| Every built-in is model-checked: Canon's `check` finds nothing under `STATE_BOUND`, except the built-ins listed in `OVER_BOUND` at their measured state count (`story:builtin-state-bound`) | `tests/builtin_canon_check.rs::every_builtin_checks_clean_under_the_bound` |
| The protocols moved from takt are byte-identical to takt tag 0.12.0 | `tests/takt_protocols_builtin.rs::takt_protocols_are_builtins_byte_for_byte` |
| `protocols/vocabulary.yaml` is the vocabulary's only source | `tests/vocabulary_yaml.rs::vocabulary_yaml_is_the_source` |
| Every term a built-in other than `support.triage/1` names is declared in the vocabulary in the category it is used in, except the four in `NOT_SPELLABLE` (`story:vocabulary-unspellable-terms`) | `tests/builtin_vocabulary_coverage.rs::every_builtin_term_is_declared_in_the_category_it_is_used_in` |
| Every protocol rule lands with a fixture under `fixtures/` that exercises it, stale-revision cases included | the fixture harness, `tests/fixture_harness.rs`; adding the fixture is review's job |
| Evaluation adds no hidden clock, network or model call; a fixture's instant is an input | `tests/support/mod.rs` reads the instant from the fixture; review |
| The package, library and binary names are `b10x-canon-engineering`, `canon_engineering`, `canon-engineering`; no retired name ships | `tests/crate_names.rs`, `tests/adversary_rename.rs` |
| Generated site files match a fresh render | `canon-engineering-docs generate --check` in `task check` |

Test paths above are under `crates/canon-engineering/`. Anything that runs is Rust, and command
lines use clap derive.

## Gate

`task check` is the gate, and CI's `check` workflow runs exactly it. Its steps, each runnable alone:

```console
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run --locked -p canon-engineering-docs -- generate --check
```

There is no `rust-toolchain.toml`; CI installs the current `stable` through
`dtolnay/rust-toolchain`, so gate on an up-to-date stable. `task plan` runs
`aep plan artifact validate`. Two more workflows run on every pull request and on `main`:
`Documentation validation` (`pages.yml`: the docs crate's tests, `generate --check`, the site
build) and `Shared source gates` (`shared-gates.yml`: the common Gates checks).

`b10x-canon` and `b10x-canon-expr` are Git dependencies on the same Canon release tag in the
crate manifests; `Cargo.lock` records its exact commit. A Canon change reaches this repository
through an explicit tag and lockfile update, which is a change to gate like any other.

## Generated files

Never edit these by hand; change the protocol or the generator, then run `task docs`
(`canon-engineering-docs generate`):

| Output | From |
|---|---|
| `website/docs/protocols/<name>/<major>.mdx`, `website/docs/protocols/index.md` | `protocols/<name>/<major>.yaml`, compiled by Canon |
| `website/data/protocol-graphs/*.json` (`b10x-protocol-graph/1`) | the same protocols |
| `website/data/example-graphs/*.json` | preview protocols in `website/examples/protocols/` |
| `website/docs/vocabulary.md` | `protocols/vocabulary.yaml` |
| `website/data/status.json` (`b10x-status/1`), `website/docs/status.mdx` | one shipped row per protocol plus the list in `crates/canon-engineering-docs/src/status.rs` |

Each generated page starts with `generated by canon-engineering-docs, do not edit`. Preview
protocols that concept pages draw belong in `website/examples/protocols/`, never in `protocols/`:
anything in `protocols/` becomes a built-in. Generation also refuses the old admonition form
(`:::kind Title`); write `:::kind[Title]`.

## Public documentation

`website/` is Docusaurus on the Docs System product-site template, with `@beyond10x/docs-system`
pinned to a commit in `website/package.json`; only the site build is Node (22 in CI). The landing
page is data, `website/product.json` (`b10x-product-landing/1`); its claims are checked like any
page's. Hand-written pages are `website/docs/index.md`, `website/docs/concepts/` and
`website/docs/showcase.mdx`; each concept page labels what is shipped, decided and planned, checked
against Canon and this repository. `task site-build` builds the site into `website/build`.

The site is independent. A bot push to `main` that passes `Documentation validation` uploads
`website/build` (bound to its commit by `canon-engineering-docs site-manifest`) and triggers
`Documentation site` (`b10x-docs-site.yml`), which deploys through Website's `project-site.yml`.
`site-manifest` also writes `.well-known/b10x-routes.json` (`b10x-project-routes/v1`, same
commit): every built page route with its element IDs; redirects, trailing-slash copies and
`404.html` are no route (`crates/canon-engineering-docs/src/routes.rs`).

In `website/docusaurus.config.ts`, `product: 'els'` stays as it is: it is a fixed Docs System
palette key, not the repository name, and any other value fails the build. For any change to the
site, its workflows or README.md, follow the workspace `docs` skill
(`~/beyond10x/.agents/skills/docs/SKILL.md`), and verify the live page before reporting it
published.

## ESS

This repository opts out of ESS for its protocol semantics: protocols are defined in Canon and
tested by Canon conformance (Atlas ADR 0067). ESS conformance reports are an evidence *kind*
engineering protocols may admit (`story:ess-conformance-evidence`, draft); that is a use of ESS
output, not a specification of this repository.

Assertion collection and its CLI are not part of that opt-out. Their wire model lives in `ess/`;
provider envelopes live in `ess/providers/`, both pinned to ESS 0.72.0. The gate document is a
generated Rust data library. The Canon expression schema is vendored by `canon-engineering-docs`
from the pinned core's `MODEL_SPEC`, with only the namespace remapped. Do not edit generated
contracts or `.ess-output` ownership state. `crates/canon-engineering-assertions/tests/ess_contract.rs`
validates/compiles/synthesizes ESS, regenerates gate types and compares native envelope fields.

`b10x-canon-engineering-assertions` owns the edge and `b10x-assertion-providers` collects observations;
all generic parsing/checking/planning/evaluation is supplied by `b10x-canon-expr` from Canon.
Keep evaluate free of provider calls (`crates/canon-engineering-assertions/tests/extensions.rs::registered_rust_provider_extends_namespace_and_replay_does_not_execute_it`).
Typed provider responses may report known values or unavailability; malformed data must fail.
Retained output stays under `.engineering/assertions/` and uses no-follow directory handles.
Explicit inputs include ignored files and generated output; a changed command executable or
source snapshot invalidates collection. Adversarial cases live in the runner's `tests/adversary.rs`.

Protocol first (Atlas ADR 0080): a unit that changes behaviour lands its protocol YAML and fixture
expectations in its first commit, a named test (a Canon fixture test under
`crates/canon-engineering/tests/`) fails on that commit and the failing run is recorded, and only
later commits implement against it. A change with no behaviour change is exempt and says so in its
story's `## Protocol first`.

## Work

- The plan is the AEP store under `.engineering/`, written only through `aep plan artifact`
  (`aep plan artifact list` to read it). Body drafts go in `.engineering/drafts/`, which Git
  ignores. Evidence for a story lives under `.engineering/evidence/story/<id>/`.
- Waves run as `aep:implementing` describes: one `impl/<story-id>` branch per unit, merged into
  `wave/<date>-w<N>`, closed by a `plan: close wave …` commit. Smaller changes go through a bot
  pull request.
- Use a managed worktree: `worktree create --repo ~/beyond10x/engineering-protocols --purpose …`,
  and end it with `worktree finish --discard-cache --archive <tree>`.
- Build into the checkout's own `target/`. The Taskfile sets no `CARGO_TARGET_DIR`; do not set
  one. `crates/canon-engineering/build.rs` embeds `protocols/` by absolute path, so a target
  directory shared between checkouts can embed another checkout's protocol files, and cargo can
  judge another checkout's test binary fresh and run it. Before a gate run counts as evidence,
  check with `cargo test --workspace --locked -- --list` that the tests the run printed exist in
  the gated tree.
- Every commit and push is `b10x-bot[bot]`'s, through `b10x-gates bot`; check both author and
  committer before pushing. Every other GitHub write (pull request, comment, release, workflow
  dispatch, re-run) goes through `b10x-gates api`. `gh` is for reading only.

## Releases

Source releases at bare-version tags (`0.1.0`). Nothing is published to crates.io and there is no
release workflow. A release is:

1. A release commit on `main` (through a wave or a bot pull request): the workspace `version` in
   `Cargo.toml`, `Cargo.lock`, and a `CHANGELOG.md` entry for that version, moved out of
   `[Unreleased]`.
2. `task check` green on that commit, and its CI checks green.
3. An annotated tag by `b10x-bot[bot]` on that commit: `b10x-gates bot … -- tag -a <version> -m
   "Engineering protocols <version>" <commit>`, then `-- push origin <version>`.
4. The GitHub Release for the tag, created by the bot (`b10x-gates api --method POST --path
   /repos/beyond10x/engineering-protocols/releases`), its notes taken from the CHANGELOG entry.

Consumers pin `b10x-canon-engineering` by `tag = "<version>"`; README.md's install lines name the
latest tag and change in the release commit.

## Never

- Re-implement Canon semantics, or add a clock, network or model call to evaluation.
- Put a preview or example protocol in `protocols/`.
- Hand-edit a generated file, or the generated block below.
- Commit App credentials or private policy to this repository.
- Write to GitHub as anyone but the bot, or report the site published without checking it live.

<!-- b10x-release-operations:start -->
## Release completion

An ordinary release completes after this repository's exact tag, required source checks,
published release and required artifacts are verified. A pushed tag with unfinished checks or
uploads is queued; report it as released only after those requirements succeed.

Atlas reconciliation and public documentation publication run asynchronously. Do not wait for
Atlas or Website, update Website source locks or bootstrap snapshots, promote consumer pins,
release shared docs tooling, or redeploy documentation façades as part of an ordinary source
release. Report documentation as pending unless its publication was actually verified. A background
documentation failure does not invalidate a successful source release.

Keep this repository's provenance, correctness, security, compatibility and artifact verification
requirements. Shared rendering, routing or delivery-control changes still require their relevant
integration gates. A release request does not authorize deployment or downstream releases.
Repositories without a release unit retain their existing publication policy. This completion
boundary supersedes older instructions that attach synchronous documentation ceremony to each
source release.
<!-- b10x-release-operations:end -->
