# AGENTS.md — engineering-protocols

This file is for an agent changing the repository. What engineering protocols are, how to take them
and check them is in [README.md](README.md); the public site is
<https://beyond10x.github.io/engineering-protocols/>. The cross-repository architecture is Atlas
ADRs 0066–0075 and 0094, and Atlas `docs/design/governed-autonomy/`.

## Serves

- **O2 — decisions as data, with evidence.** Engineering rules are Canon protocols declared as
  data and evaluated against cited, revision-bound evidence, with `UNKNOWN` kept apart from `FALSE`.
- **O1 — governed reach.** Every write action a protocol declares names the capability that
  authorises it, except `software.change/1`'s `tests.write` and `implementation.edit` (telling
  them apart needs a path scope on capabilities, which Canon does not have), and the six write
  actions of the protocols moved from takt that name none: `story.delivery/1`'s
  `tests.write`, `implementation.edit` and `pr.open`, `self.improvement/1`'s `finding.file`,
  `loop.improvement/1`'s `loop.change.deliver` and `loop.finding.file`
  (`story:takt-protocol-write-capabilities`). Loom's `loom-governor` consumes these protocols;
  § Boundary says how a runtime reads them.

## Boundary

This repository owns engineering-domain vocabulary and the protocols written in it (Atlas ADR
0068): `software.change/1`, `incident.response/1`, `ml.experiment/1` and `eval.gated_change/1`
today, investigation, migration and security response later. Two protocols here are outside
engineering, and their terms stay out of `protocols/vocabulary.yaml`: `support.triage/1`, placed
here by operator decision of 2026-10-05 as the rules half of a support triage example, its actions
tool-agnostic like every other protocol's; and `protocol.adopt/1`, the case that adopts one
revision of a generated protocol (Atlas ADR 0086 § Decision 4), placed here by the decision of
2026-10-29 recorded in `decision-blocker:protocol-adopt-home`, because the registry and the
fixture harness are here. A change to the shape of a `canon diff` or `canon check` report that it
reads as evidence needs a release here as well.

Every spec-level protocol a runtime uses lives here (Atlas ADR 0094): `story.delivery/1`,
`self.improvement/1` and `loop.improvement/1` came from takt byte for byte (takt tag 0.12.0) and
stay identical to it but for changes agreed with takt, each listed in `AGREED` in
`tests/takt_protocols_builtin.rs`. The one so far is D257 (2026-10-30, takt agreeing):
`story.delivery/1`'s `pr.open` opens its pull request through the run's forge, since a public
protocol names no organisation's delivery route. Any other change needs takt's agreement first,
or goes into a new revision. Runtime data that is not a
protocol (takt's `loop-improvement/metrics.yaml`) stays in the runtime. Their terms are in the
vocabulary, marked `software.change`, `self.improvement` or `loop.improvement` when they mean
nothing outside that domain or protocol.

A runtime reads a protocol as a file. It takes `protocols/<name>/<major>.yaml` and
`protocols/vocabulary.yaml` from a release tag or from the release archive (§ Releases), checks
each file's sha256 against `protocols/manifest.yaml`, and links Canon itself, not
`b10x-canon-engineering` (`approval-record:data-only-catalog`, operator decision of 2026-11-21;
README.md § Take the protocols and check them has the commands). That replaces Atlas ADR 0094
decision 2, which had a runtime pin the registry crate and read protocols through it; the
replacement is not yet recorded in Atlas (no ADR under `architecture/adr/` says it). Decision 4 of
that ADR (a protocol change in a runtime is a pin bump, so the pin files are on the runtime's judge
surface) still describes a Cargo pin and has not been restated for a catalog read as files; that is
not decided here and not yet recorded in Atlas.

Until `story:protocol-check-tool` lands the registry crate `b10x-canon-engineering` still exists,
with its `registry` module and the `canon-engineering` binary, and tests under
`crates/canon-engineering/tests/` still go through it; the invariants below that name them hold
today. It is not the way to consume the catalog. What still compiles against it is in
`upstream-blocker:consumers-compile-against-canon-engineering`. That story (draft) deletes the
crate: the `canon-engineering` binary moves to `b10x-canon-engineering-assertions` under the same
name with its `assertions` and `gates` commands unchanged, and `protocols list` and `protocols
show` end.

It does not own:

| Not here | Owner |
|---|---|
| Claim, evidence and obligation semantics; validation, compilation, evaluation | Canon. Do not re-implement them here. |
| The live engineering record | AEP (Atlas ADR 0069) |
| Running agents | Loom. Its `loom-protocols`, `loom-governor`, `loom-intake-slice` and `loom-intake-router` crates still compile against `b10x-canon-engineering` until Loom reads the catalog as files (`upstream-blocker:consumers-compile-against-canon-engineering`); takt is frozen on its tag |

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
| A generated protocol is adopted only with a `diff_report` within the floor, a passing `check_report` and an approval under `protocol.adopt`, each bound to the revision under adoption; the approval counts only from a producer the case lists on `different_principal` and only after the first diff report, since Canon outcomes read claims, not authority decisions; a new revision leaves every claim `UNKNOWN`, not `FALSE`, and a report outside the floor makes `floor.respected` `FALSE` | `tests/protocol_adopt_protocol.rs::adoption_needs_floor_check_and_authority`, `every_protocol_adopt_fixture_holds` |
| Every built-in is model-checked: Canon's `check` finds nothing under `STATE_BOUND`, except the built-ins listed in `OVER_BOUND` at their measured state count (`story:builtin-state-bound`) | `tests/builtin_canon_check.rs::every_builtin_checks_clean_under_the_bound` |
| The protocols moved from takt are byte-identical to takt tag 0.12.0 but for the changes agreed with takt (`AGREED`) | `tests/takt_protocols_builtin.rs::takt_protocols_are_builtins_byte_for_byte` |
| `protocols/vocabulary.yaml` is the vocabulary's only source | `tests/vocabulary_yaml.rs::vocabulary_yaml_is_the_source` |
| `protocols/manifest.yaml` (`b10x-protocol-catalog/1`, schema `schema/b10x.protocol-catalog.v1.schema.json`) lists every other file under `protocols/` with the sha256 of its bytes, and each protocol with the id and revision Canon parses from it and a hand-declared lowest Canon release; a Canon pinned in `Cargo.lock` below a listed release fails, an understated release is not detected; its text keeps the line shape of README.md's `awk` check, which the test's `line_pairs` repeats, so a change to that line edits both (`LineShape`) | `tests/catalog_manifest.rs::the_manifest_matches_the_catalog`, `the_readme_holds_the_awk_program_the_line_reader_is_written_from`, `catalog_files_are_checked_out_byte_for_byte` |
| Every term a built-in other than `support.triage/1` and `protocol.adopt/1` names is declared in the vocabulary in the category it is used in, except the four in `NOT_SPELLABLE` (`story:vocabulary-unspellable-terms`) | `tests/builtin_vocabulary_coverage.rs::every_builtin_term_is_declared_in_the_category_it_is_used_in` |
| Every protocol rule lands with a fixture under `fixtures/` that exercises it, stale-revision cases included | the fixture harness, `tests/fixture_harness.rs`; adding the fixture is review's job |
| Evaluation adds no hidden clock, network or model call; a fixture's instant is an input | `tests/support/mod.rs` reads the instant from the fixture; review |
| The package, library and binary names are `b10x-canon-engineering`, `canon_engineering`, `canon-engineering`; no retired name ships | `tests/crate_names.rs`, `tests/adversary_rename.rs` |
| The catalog archive (`canon-engineering-docs catalog-archive`) holds the files `protocols/manifest.yaml` lists and the manifest, each with its listed sha256, as plain entries in byte order with no owner and no time, and two runs on one tree are byte-identical; a manifest that breaks a path rule of the manifest check, a missing or changed listed file, or an unlisted file under `protocols/` is refused before anything is written | `crates/canon-engineering-docs/tests/catalog_archive.rs::two_runs_write_identical_archives_and_digests`, `archive_holds_exactly_what_the_manifest_lists_and_each_file_has_its_digest`, and `tests/adversary_catalog_archive.rs` beside it |
| `canon-engineering-docs` reads `protocols/` and `protocols/vocabulary.yaml` as files and has no dependency on `b10x-canon-engineering`; the vocabulary reader is defined once, in `crates/engineering-vocabulary` (`b10x-engineering-vocabulary`, `publish = false`, no embedded data), and `canon_engineering::vocabulary` re-exports it beside the embedded released vocabulary. The one file the generator shares with the registry crate is `crates/canon-engineering/src/builtin_name.rs`, included by `#[path]` | `crates/canon-engineering-docs/tests/no_registry_dependency.rs`, `tests/vocabulary_reexport.rs` |
| `b10x-canon-engineering-assertions` and `b10x-assertion-providers` are frozen at the behaviour of tag 0.16.0 and `publish = false`: no new assertion function, provider, CLI command or wire-model field | review; the two crates' READMEs say what a pin move still edits |
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
(`~/beyond10x/atlas/.agents/skills/docs/SKILL.md`), and verify the live page before reporting it
published.

## ESS

This repository opts out of ESS for its protocol semantics: protocols are defined in Canon and
tested by Canon conformance (Atlas ADR 0067). ESS conformance reports are an evidence *kind*
engineering protocols may admit (`story:ess-conformance-evidence`, draft); that is a use of ESS
output, not a specification of this repository.

The two assertion crates, `b10x-canon-engineering-assertions` and `b10x-assertion-providers`, are
frozen at the behaviour of tag 0.16.0 and `publish = false` (§ Invariants). The owner of what
replaces them is Loom; `decision-blocker:assertion-executor-owner` (cleared) is the record. The
rest of this section still binds a security fix or a Canon or ESS pin move there; the crates'
READMEs say what a pin move edits.

Assertion collection and its CLI are not part of that opt-out. Their wire model lives in `ess/`;
provider envelopes live in `ess/providers/`, both pinned to ESS 0.77.0. The gate document is a
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
- A change to any file under `protocols/` updates its entry in `protocols/manifest.yaml` in the
  same commit: the sha256 (`sha256sum <file>`), and for a protocol the id, the revision and, when
  it uses a newer Canon form, `canon`. The manifest is hand-written, not generated, and carries
  no comments. Files under `protocols/` and `schema/` are checked out byte for byte
  (`.gitattributes`, `-text`), because the manifest lists the sha256 of their bytes.
- Build into the checkout's own `target/`. The Taskfile sets no `CARGO_TARGET_DIR`; do not set
  one. `crates/canon-engineering/build.rs` embeds `protocols/` by absolute path, so a target
  directory shared between checkouts can embed another checkout's protocol files, and cargo can
  judge another checkout's test binary fresh and run it. Before a gate run counts as evidence,
  check with `cargo test --workspace --locked -- --list` that the tests the run printed exist in
  the gated tree.
- Every commit and push is `b10x-bot[bot]`'s, through `b10x-gates bot`; check both author and
  committer before pushing. Every other GitHub write (pull request, comment, release, workflow
  dispatch, re-run) goes through `b10x-gates api`, which sends JSON bodies; a release asset is a
  file and goes through `b10x-gates gh -- release upload`. Bare `gh` is for reading only.

## Releases

Source releases at bare-version tags (`0.1.0`). Nothing is published to crates.io and there is no
release workflow. A release is:

1. A release commit on `main` (through a wave or a bot pull request): the workspace `version` in
   `Cargo.toml`, `Cargo.lock`, and a `CHANGELOG.md` entry for that version, moved out of
   `[Unreleased]`; and README.md § Status (the latest release, its date, the Canon and ESS
   versions) and, for a protocol the release adds, its `Released in` cell in § Protocols.
2. `task check` green on that commit, and its CI checks green.
3. An annotated tag by `b10x-bot[bot]` on that commit: `b10x-gates bot … -- tag -a <version> -m
   "Engineering protocols <version>" <commit>`, then `-- push origin <version>`.
4. The GitHub Release for the tag, created by the bot (`b10x-gates api --method POST --path
   /repos/beyond10x/engineering-protocols/releases`), its notes taken from the CHANGELOG entry.
5. The catalog archive, built from a checkout of the tag with its own `Cargo.lock`:
   `task catalog-archive` writes `target/catalog/engineering-protocols-catalog-<version>.tar.gz`
   and the same name with `.sha256` appended, `<version>` being the workspace version of that
   checkout, which must equal the tag. The bot attaches both files to the release:
   `b10x-gates gh -- release upload <version> <the two files>`.

The archive holds what `protocols/manifest.yaml` lists (the protocol files, `vocabulary.yaml`,
the schema it names) and the manifest, at repository-relative paths with no top-level directory.
It is the same bytes for one tree and one `Cargo.lock`: the compressed bytes depend on the
locked `flate2`, so the archive of a release is the one attached to it, and a rebuild is compared
by unpacking, not by digest, once `Cargo.lock` has moved.

A consumer takes the files of a tag or the archive and checks them against the manifest
(README.md § Take the protocols and check them). Those commands take `<version>`, so README.md
names no version in a command and the release commit edits none.

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

The required artifacts are the two release assets of § Releases step 5,
`engineering-protocols-catalog-<version>.tar.gz` and the same name with `.sha256` appended. A
release without both is queued, not complete. They are verified by downloading both from the
published release and running `sha256sum -c` on the digest file beside the archive. Releases up
to 0.16.0 have no assets.

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
