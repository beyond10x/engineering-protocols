# Changelog

All notable changes to this component are recorded here. Versions are component-scoped and released
under bare-version tags such as `0.1.0`. The workspace is `publish = false`; releases are source
releases at bare-version tags.

## [Unreleased]

### Breaking

- The generated `Gate` has an optional `applies_when` field, so a `Gate { .. }` literal needs
  `applies_when: EssPresence::Absent`; `cli::Input` has four more fields (`Input::default()` fills
  them); `ProviderContext` has a `change` field; `gates run` and `gates evaluate` print a
  `GatesReport` (a superset of Canon's report: `truth` and `assertions` stay).
- `CommandBinding` has a `program` field (a `CommandBinding { .. }` literal needs `program: None`), its
  `executable` defaults to empty, `Project` has `search_path`, `Bundle` has `programs`, and
  `Admission` has `search_path`.
- `evaluate` (the Canon-shaped replay) no longer counts a `not_applicable` gate as a failure: its
  `truth` is the aggregate of the gates' results (`true` only for `pass`, `false` for `fail`,
  otherwise `unknown`, so a run in which nothing applied is not `true`), and its `assertions` are each
  gate's own assertions in gate order with no `applies_when` expression among them, so they pair with
  `document.gates` whether or not a gate has one. `evaluate_gates` still returns every expression
  in `assertions`.
- `git.changed` globs that can never match a path git reports (a leading `./`, a trailing `/`, an
  absolute or empty glob, an empty or `.` segment) are refused at admission; they used to be
  silently false.

### Added

- Gates can see the change and may decline to apply. `git.changed(glob)` is true when a path
  matching `glob` differs between the context's `base` and `subject` revisions (unknown when either
  is absent); `story.id()` and `story.has_tag(tag)` read the story the runtime names. A gate's
  optional `applies_when` expression that is false makes the gate `not_applicable`, never `pass`;
  `evaluate_gates` returns each gate's result and an aggregate, and `gates run` exits 4 when nothing
  applied. The runtime passes the change through `prepare_with` and `Admission`, or the CLI's
  `--base`, `--subject`, `--story-id` and `--story-tag`; the change enters the context identity.
- A command in an assertions project can name a `program` instead of an absolute `executable`.
  Admission resolves the name on the project's `search_path`, else on the default the runtime passes
  in (`Admission::search_path`, or `--default-search-path`); the caller's `PATH` is never read. The
  match must be an executable file outside the tree, and every entry must be absolute and outside
  the tree, or admission refuses and names it. The resolved absolute path and its sha256 are
  recorded in the evidence (`programs`), and a change of the file before or during collection is
  refused like any admitted executable. The process is still started with a cleared environment.

### Changed

- `git.changed` measures the change from the merge base of `base` and `subject` (`base...subject`),
  so edits the base gained after the subject forked are not credited to the change; a subject that
  is an ancestor of the base is compared tree to tree. Paths are relative to the gates root, as
  every other path operation is, so a root in a subdirectory of the repository matches `CHANGELOG.md`
  against its own file. A path that is not UTF-8 no longer aborts the run.
- The context identity carries `base` and `subject` as the commit ids they name at admission, so a
  ref that moved is another context.
- The CLI's `--story-id` without `--story-tag` leaves `story.has_tag` unknown, as its help says.
- The sha256 of a program resolved at admission is read once and is the one the run locks.

### Fixed

- A promisor remote in a repository's configuration could make `git.changed` lazily fetch and start
  `remote.origin.uploadpack` through a shell. Git now runs with `GIT_NO_LAZY_FETCH=1`,
  `GIT_PROTOCOL_FROM_USER=0` and `protocol.allow=never`.
- A `search_path` entry inside the tree that is spelled through `..` and does not exist is refused
  by name instead of skipped.

## [0.10.0] - 2026-10-26

### Breaking

- `vocabulary::Marking` has two new variants, `SelfImprovement` (`self.improvement`) and
  `LoopImprovement` (`loop.improvement`); an exhaustive `match` on it no longer compiles.
- ESS 0.69.2 regenerates the provider contract: `assertion_providers::contract::Status` variants
  are `Known` and `Unavailable` instead of `V0` and `V1`. The wire values (`known`,
  `unavailable`) are unchanged.

### Added

- Three built-in protocols moved from takt byte for byte (Atlas ADR 0094, takt tag 0.12.0):
  `story.delivery/1` (`protocols/story-delivery/1.yaml`), `self.improvement/1` and
  `loop.improvement/1`. A runtime reads them through `registry::get` instead of carrying a copy;
  takt's `loop-improvement/metrics.yaml` stays in takt. Fixtures under `fixtures/story-delivery/`
  (7), `fixtures/self-improvement/` (14) and `fixtures/loop-improvement/` (7) reach every outcome
  and kill every rule mutant two adversary passes tried.
- The vocabulary holds 60 new terms (171 in all): 20 marked `software.change`, 25
  `self.improvement`, 15 `loop.improvement`. The meanings of `baseline`, `candidate` and
  `declined` are widened to cover the new protocols.
- Tests that hold every built-in to Canon's `check` under `STATE_BOUND`, to vocabulary coverage
  for every built-in but `support.triage/1`, and the takt protocols to takt's bytes.

### Changed

- The assertion contracts in `ess/` and `ess/providers/` require ESS 0.69.2 and are regenerated
  by it; CI installs ESS 0.69.2. The generated `Cargo.toml` files require `serde` and
  `serde_json` by caret instead of exact version; `Cargo.lock` is unchanged.

### Limits

- `self.improvement/1` (5,971,968 states), `ml.experiment/1` and `software.change/1` are over
  Canon 0.6.0's state bound of 65,536 and are listed in the check test's `OVER_BOUND`
  (`story:builtin-state-bound`).
- Four takt terms break the vocabulary's spelling rule and are not declared: `promoted`,
  `loop.change.deliver`, `loop.definition.check`, `loop.finding.file`
  (`story:vocabulary-unspellable-terms`).
- Six write actions of the takt protocols name no capability: `story.delivery/1` `tests.write`,
  `implementation.edit`, `pr.open`; `self.improvement/1` `finding.file`; `loop.improvement/1`
  `loop.change.deliver`, `loop.finding.file` (`story:takt-protocol-write-capabilities`).

## [0.9.0] - 2026-10-17

### Added

- `eval.gated_change/1` at `protocols/eval-gated-change/1.yaml`, a fifth built-in protocol: it
  decides one change an automated loop made to an agent's text or configuration against an
  evaluation set. Artifacts `eval_design` and `candidate`, whose revision hashes its content with
  its basis (baseline, question set, data revision, design), so a moved basis is a new candidate
  revision; evidence kinds `design_review`, `guard_check` (`outside`, `leaked`, `clean`),
  `tuning_comparison` (`reproduced`, `not_reproduced`, `not_better`) and `heldout_comparison`
  (`held`, `regressed`); the keep claims `experiment.valid`, `change.within_surface`,
  `content.clean`, `tuning.improved`, `result.reproducible` and `heldout.not_regressed`, each
  demanding a producer independent on `different_agent_run`, and `change.supported`, all of them.
  `change.keep` needs `change.supported` and the capability `loop_change.keep`; `change.reject`
  needs every keep claim decided and one false; `change.propose` a candidate outside the surface;
  `change.apply_outside_surface` the capability `loop_change.apply_outside_surface`, never granted
  to a loop. Outcomes `kept`, `rejected`, `proposed`, and `inconclusive` on
  `explicitly_inconclusive`. Evidence by the run that wrote the candidate, or about an earlier
  candidate revision, leaves its claim unknown, and unknown neither keeps nor rejects. It declares
  no invalidation rule and `canon check` covers it in 55,296 states with no findings. Fifteen
  fixtures under `fixtures/eval-gated-change/` show it.
- The vocabulary holds its 18 new terms, all core; it reuses `candidate`, `design_review`,
  `experiment.valid`, `result.reproducible`, `inconclusive`, `explicitly_inconclusive` and
  `different_agent_run`.

### Limits

- `change.apply_outside_surface` is admissible in any case whose authority grants
  `loop_change.apply_outside_surface`: "never granted to a loop" rests on the case's authority
  input, as every protocol's capability does, not on the protocol.
- `kept` and `inconclusive` can both be legitimate when an `explicitly_inconclusive` decision
  sits beside passing evidence; only `kept`, `rejected` and `proposed` exclude each other.
- An explanation says the evidence is about another candidate revision, not which part of the
  basis moved, and every candidate claim goes unknown when any part of it moves.

## [0.8.0] - 2026-10-16

### Changed

- `b10x-canon` and `b10x-canon-expr` move from Canon 0.4.0 to Canon 0.6.0 (tag `0.6.0`, commit
  efcfd040). `registry::get` returns Canon 0.6.0's validated model, so a consumer that also
  depends on Canon 0.6.0 resolves one `b10x-canon`. Canon 0.5.0 adds optional action argument
  schemas and Canon 0.6.0 a requirements catalogue for its own conformance run; no protocol here
  declares either, and every protocol evaluates as before.
- The assertion contracts in `ess/` and `ess/providers/` require ESS 0.60.0 and are regenerated
  by it (only the generator stamp changes); CI installs ESS 0.60.0.

## [0.7.0] - 2026-10-13

### Added

- `software.change/1` gains an independent security-review rule, on Canon 0.4.0's independence
  requirements. Case input `affects_security_boundary` (`true`, `false`); independence dimensions
  `different_principal` and `different_agent_run`; evidence kind `security_review` about the
  implementation; claim `security.reviewed`, an approving `security_review` about the current
  implementation revision whose `producer` the case's `independence` map lists on
  `different_principal`; obligation `security_review.required`, which applies on a security
  boundary at every risk and is discharged by `security.reviewed`. `implementation.verified` now
  also needs `security.reviewed` when the change affects a security boundary. A review by a
  producer not listed as independent is excluded as `not_independent` and leaves the claim
  unknown, never false. The trusted context that writes the case resolves who is independent;
  the protocol only declares what it demands. Fixtures `security-boundary` and
  `security-boundary-independent` show it.
- `incident.response/1` gains an independent rollback-verification rule. Independence dimension
  `different_principal`; evidence kind `rollback_result` (`complete`, `partial`), which
  `release.rollback` may produce; claim `rollback.verified`, a complete `rollback_result` about the
  release whose `producer` the case's `independence` map lists on `different_principal`. A result
  from a producer not listed as independent, such as the actor who rolled back, is excluded as
  `not_independent` and leaves the claim unknown, never false. No case input and no obligation
  are added, so existing incident cases stay valid; a case with no `independence` map leaves
  `rollback.verified` unknown. Fixtures `rollback-verification` and
  `rollback-verification-independent` show it; `inc-492` expects `rollback.verified` unknown in
  every state.
- Vocabulary term `rollback.verified` (claim, marked `core`).
- Vocabulary category `independence_dimension`, and the terms `affects_security_boundary`,
  `true`, `false`, `security_review`, `security.reviewed`, `security_review.required`,
  `different_principal` and `different_agent_run`.
- The fixture harness documents the case's `independence` map, which Canon reads with the case.
- Fixtures `stale-review`, `stale-observation` (`fixtures/software-change/`) and `stale-health`
  (`fixtures/incident-response/`): a review of a revision the implementation no longer has leaves
  `implementation.reviewed` unknown, and one healthy observation evaluated 29 and then 31 minutes
  after it was made holds and then leaves `deployment.healthy` or `service.healthy` unknown, never
  false; in the incident, `restore_service` reopens and `emergency.leave` is blocked again.
- The fixture harness passes each state's evaluation instant to Canon, and a state may give its
  own `at`, not before the instant of the state before it; a state without one inherits it.
- Protocol pages list a protocol's independence dimensions, say "from a producer independent on
  `<dimension>`" for an evidence match that demands one, and show an evidence kind's `max_age` in
  an "Applies for" column; a protocol graph's `establishes` edge appends `, independent on
  <dimensions>` to its qualifier (`b10x-protocol-graph/1` is unchanged).

### Changed

- `operational_observation` expires after 30 minutes (`max_age: 30m`) in both
  `software.change/1` and `incident.response/1`: at an evaluation instant more than 30 minutes
  after a record's `observed_at`, Canon excludes the record as `expired`, and the claim it would
  have decided (`deployment.healthy`, `service.healthy`) is unknown, not false. A caller that
  evaluates without an instant (no `--at`) gets no expiry: Canon reads no clock, so without an
  instant, or for a record without `observed_at`, nothing expires. Fixtures `chg-1842`, `inc-492`,
  `rollback-verification` and `rollback-verification-independent` moved their observation times
  into the 30 minutes before their instant and keep every expectation.
- Every `software.change/1` case must now give `affects_security_boundary`: Canon refuses a case
  that leaves out a declared input (`missing-input`). Every fixture under
  `fixtures/software-change/` gives `affects_security_boundary: false` and keeps its
  expectations.
- Canon moves from 0.3.0 to 0.4.0 (`b10x-canon`, `b10x-canon-expr`); protocols that demand no
  independence evaluate as before.
- Fixtures `security-boundary-rejected`, `security-boundary-other-subject`,
  `rollback-verification-partial` and `rollback-verification-other-subject` hold a record that
  contradicts each condition of `security.reviewed` and `rollback.verified` (result, subject,
  independence), so dropping any one of them fails a fixture.

### Known limits

- `rollback.verified` is a claim only: no obligation, action or outcome of `incident.response/1`
  reads it yet. `software.change/1`'s `rolled_back` outcome still accepts a complete
  `rollback_result` from any producer.
- Who is independent of whom is the case writer's word: the protocols declare the demand, and
  the trusted context that writes the case's `independence` map resolves identities. A case
  writer can also change `affects_security_boundary` between snapshots, as it can `risk`.
- The rules are held by this repository's fixtures, not by an entry in Canon's requirements
  catalogue, which Canon has not yet released.

## [0.6.0] - 2026-10-13

### Added

- `software.change/1` reads a bugfix's red-first rule from evidence order, on Canon 0.3.0's
  `first` and `before`. Evidence kinds `regression_test_result` (`tests.run` may produce it) and
  `implementation_change` (`implementation.edit` may produce it); `regression.reproduced` also
  holds when the first `regression_test_result` about the implementation failed, on any revision;
  claim `regression.shown_first` holds when that first result failed and came before the first
  `implementation_change` about the implementation; obligation `red_first.required` applies to
  every bugfix, at every risk, and is discharged by it, so a regression test whose first result
  passed never discharges it. A bugfix that runs the suite green and its regression test red on R1
  may edit, and its red run keeps counting once the edit moves the implementation to R2: the
  limitation 0.5.0 stated is lifted. Fixtures `red-first`, `red-first-edit-first`,
  `red-first-green-first` and `red-first-other-subject` show it. Known limits: from Canon, a red
  record about any earlier revision counts, whether or not the current revision descends from it;
  and the rule sees only recorded `implementation_change` evidence, which `implementation.edit`
  may but need not produce, so a change made without that record is not seen.
- A protocol page writes the order forms in words.
- Protocol graphs draw the order forms as docs-system 0.10.0's `b10x-protocol-graph/1` predicate
  leaves `{"first": {"kind", "result", "subject"?}}` and `{"before": {"first", "then",
  "subject"?}}`, with `subject` only when the form names one. In a claim's `true_when` each draws
  an `establishes` edge from every kind it reads, qualified `first <result>`, `before <then>` or
  `after <first>`, then `about <subject>` when bound and prefixed `not` under negation.

### Changed

- Canon is pinned at tag 0.3.0.
- `website/package.json` pins docs-system 0.10.0 (`48afe6a`).
- The fixture harness passes each record to Canon observed at its entry's `observed_at`, which the
  order forms read; it still passes no evaluation instant, so nothing expires.
- `regression.reproduced` is unknown, not false, for a change whose tests pass and whose
  regression test never ran: the fixtures `chg-1842`, `superseded` and `profiles-discharged` say
  so.

## [0.5.0] - 2026-10-13

### Added

- `software.change/1` case inputs `risk` (`trivial`, `standard`, `elevated`, `critical`) and
  `change_kind` (`feature`, `bugfix`, `refactor`), which every case now gives. Obligations
  `tests.required`, `review.required`, `provenance.required` and `health.required` apply by risk,
  each level adding one, and are reported `not_applicable` otherwise; they do not gate
  `accepted`. The claim `regression.reproduced` is TRUE on a failing `test_result` about the
  current implementation revision, and a bugfix may `implementation.edit` only once it holds; red
  on R1 no longer counts at R2, and a pass and a fail on one revision leave it unknown. Fixtures
  `profiles`, `profiles-discharged` and `change-kinds` show them.
- Vocabulary categories `case_input` and `input_value`, and 16 terms for the profiles. An
  obligation id may be dotted (`tests.required`) as well as one token (`restore_service`).
- The fixture harness's `set_inputs:` state key sets a case input the case gives.
- Protocol graphs carry docs-system 0.9.0's `b10x-protocol-graph/1` additions: a top-level
  `case.inputs`, the `input` predicate leaf and an obligation's `applicable_when`.

### Changed

- `software.change/1`'s `repository.edit` splits into `tests.write`, with no precondition, and
  `implementation.edit`; the vocabulary drops `repository.edit`. Every `software.change`
  fixture gives the case inputs.
- The protocol graph generator no longer refuses a protocol with case inputs or a conditional
  obligation; `website/package.json` pins docs-system 0.9.0.
- The assertion contracts in `ess/` and `ess/providers/` require ESS 0.59.0, CI installs it, and
  the generated gate and provider contracts are regenerated by it. Number handling is unchanged:
  both host crates keep `serde_json/arbitrary_precision`.

### Removed

- The site's preview protocol `software-change-bugfix/1` and its example graph: the built-in
  carries the `bugfix` profile, and the step-order page draws `software.change/1` instead.

## [0.4.0] - 2026-10-12

Source release using Canon 0.2.0 and ESS 0.55.0. `software.change/1` gains outcomes and an action,
and `ml.experiment/1` is a fourth built-in protocol; the gate and provider contracts are unchanged.

### Added

- `ml.experiment/1` at `protocols/ml-experiment/1.yaml`: artifacts `experiment_design`,
  `dataset_snapshot`, `baseline`, `candidate`, `run_configuration` and `results`; claims
  `experiment.valid`, `result.reproducible`, `candidate.beats_baseline`, `effect.robust` and
  `hypothesis.supported`; outcomes `supported`, `refuted`, `invalid_experiment`, `inconclusive` and
  `superseded`. A new dataset snapshot or baseline revision invalidates the claims that rest on
  it. `refuted` needs a comparison that is `FALSE`: one that is missing or about another candidate
  revision leaves it blocked. Seven fixtures under `fixtures/ml-experiment/` show it.
- `software.change/1` outcomes `declined`, `superseded` and `abandoned`, each earned only by an
  explicit decision on the current case revision, and `rolled_back`, earned by a `complete`
  `rollback_result` about the current release revision. Action `release.rollback` needs the
  `release.rollback` capability and may produce a `rollback_result`. Fixtures `declined`,
  `superseded`, `abandoned`, `rolled-back` and `rolled-back-complete` show them.
- Vocabulary category `decision_name` (titled Decisions) and 29 core terms for the two protocol
  changes; the generated protocol, index, graph, vocabulary and status pages follow.
- Protocol pages list the case inputs a protocol declares, with their values, and an "Applies
  when" column for obligations that carry `applicable_when`.

### Changed

- `b10x-canon` and `b10x-canon-expr` move from Canon 0.1.0 to 0.2.0.
- The protocol graph generator refuses a protocol that tests a case input or makes an obligation
  conditional, naming each place, since `b10x-protocol-graph/1` has no shape for either yet. No
  built-in protocol does.

## [0.3.0] - 2026-10-08

Source release using Canon 0.1.0 and ESS 0.55.0. `incident.response/1` gains an obligation, so
its decisions list one more entry; the gate and provider contracts are unchanged.

### Added

- `incident.response/1` declares obligation `investigate_cause`, discharged when
  `cause.identified` is `TRUE`. It stays open after `emergency.leave` becomes admissible, until a
  cause analysis of a current revision of the service or the release identifies the cause.
  `emergency.leave` still rests on restoration evidence only.
- Fixture `inc-492` expects `investigate_cause` in every state, open again after the rollback
  makes the earlier analysis stale, and adds state `cause-identified-after-restore`.
- Vocabulary term `investigate_cause` (`obligation_id`); the generated protocol, index, graph,
  vocabulary and status pages follow.
- `AGENTS.md` names the Atlas objectives the repository serves (O2, O1).

## [0.2.1] - 2026-10-07

Source release using Canon 0.1.0 and ESS 0.55.0. Protocol formats and the generated gate and
provider contracts are unchanged.

### Changed

- The ESS pin moves from 0.53.0 to 0.55.0 in `ess/` and `ess/providers/`, and CI installs 0.55.0.
  The regenerated contracts differ only in their generator stamp; source, contract and projection
  digests are unchanged.
- Builds go into each checkout's own `target/`: the Taskfile no longer sets `CARGO_TARGET_DIR`.

## [0.2.0] - 2026-10-05

Source release using Canon 0.1.0. Existing protocol formats remain unchanged.

### Added

- Typed assertion catalog and `gates validate/run/evaluate/explain`, backed by Canon's pure
  expression core. Collection retains source/context/implementation-bound observations for
  offline replay. Recipes and explicitly admitted Rust providers extend the catalog.
- File/document/schema, process/test, generated-tree, ESS/Codegate report, DNS and TCP providers;
  generated catalog reference and an assertion guide. AEP acceptance and Kubernetes remain planned.

- `support.triage/1`: triage one support ticket with tool-agnostic actions, step order by
  precondition (classify, review, then route) and outcomes `triaged`, `escalated` and `needs_human`;
  three fixtures, a `canon check` test and a concept page.

## [0.1.0] - 2026-10-05

The first release.

### Added

- `software.change/1` and `incident.response/1` on Canon, embedded in the crate and listed by the
  protocol registry (`canon_engineering::registry`).
- The engineering vocabulary, read from `protocols/vocabulary.yaml`.
- The fixture harness that compiles and evaluates each protocol's fixtures.
- The `canon-engineering` command line: list and show the built-in protocols.
- `canon-engineering-docs`: generated protocol pages and the documentation site manifest.

### Changed

- The crate is `b10x-canon-engineering` (library `canon_engineering`, binary `canon-engineering`),
  formerly `b10x-els`; the docs crate is `canon-engineering-docs`, formerly `els-docs`.
