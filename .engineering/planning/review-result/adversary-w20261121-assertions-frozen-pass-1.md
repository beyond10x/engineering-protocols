---
format: aep.planning-md/3
id: review-result:adversary-w20261121-assertions-frozen-pass-1
kind: review-result
status: active
title: Wave 2026-11-21-w3 adversary, story:assertions-frozen-at-0-16-0, pass 1
relations:
- reviews: story:assertions-frozen-at-0-16-0
revision: 1
---
unit: story:assertions-frozen-at-0-16-0, commit ba94af2 on impl/assertions-frozen-at-0-16-0 (base 794db23; worktree clean, no change by this pass)
verdict: green (no red case; strongest finding CONFIRMED warning, pre-existing)
cases: n/a (suite not run)
origin: introduced 3 / pre-existing 2 / undecided 0
wrote-outside-worktree: 2 paths, both in the adversary scratch under the coordinator's session scratch
needs-coordinator: yes (where F1 and F2 go; the nightly's rustc version is in no scratch file)

No defect a test can catch, so no case was added. Five findings, none a blocker: two places on the
website still say Kubernetes and AEP acceptance are planned, one reason given in both READMEs is
wrong, and the changelog has no entry.

**Run.** One cargo command, `cargo metadata --locked --format-version 1 --no-deps`: exit 0;
`publish` is `[]` for `b10x-assertion-providers`, `b10x-canon-engineering-assertions` and
`canon-engineering-docs`, and absent for `b10x-canon-engineering`. The implementor's 97 tests
before and after were not re-run.

**Findings**

| # | Finding | file:line | Measured | What reaches it | Verdict | Origin |
|---|---|---|---|---|---|---|
| F1 | The status page still lists "AEP acceptance and Kubernetes providers" as planned | `crates/canon-engineering-docs/src/status.rs:108` (generated into `website/data/status.json` and the status page) | `git grep -i "kubernetes\|AEP acceptance"`; the freeze says no new provider, so the line became false when the unit landed; the acceptance covers only README.md § Assertion gates | every reader of the public status page; neither unit of the wave fixes it | CONFIRMED | pre-existing |
| F2 | The assertion guide still lists AEP acceptance and Kubernetes collection as planned, says later catalogs can expose `k8s.namespace.exists(...)`, and says nothing about the freeze | `website/docs/guides/assertions.md:11` (also `:170`, `:189`) | same grep | README.md's changed paragraph sends readers to this guide | CONFIRMED | pre-existing |
| F3 | The reason "because the crates share its lockfile" is wrong for both pins | `crates/canon-engineering-assertions/README.md:16` | the ESS pin is in `ess/ess-inputs.yaml:2`, `generated/types.rs:1` and `.github/workflows/check.yml:51`; the Canon pin is `tag = "0.9.0"` in the crate's own `Cargo.toml:15`; the sentence is copied from the story's "What frozen means" | a reader who moves a pin and expects the lockfile to carry it | CONFIRMED | introduced |
| F4 | The same sentence | `crates/assertion-providers/README.md:13` | as F3; its ESS pin is `crates/assertion-providers/generated/types.rs:1` | as F3 | CONFIRMED | introduced |
| F5 | `[Unreleased]` has no entry for `publish = false` or the freeze | `CHANGELOG.md:7` | `[Unreleased]` lists three stories, not this one; the file is the coordinator's | the next release's notes are taken from this entry | CONFIRMED | introduced |

**Checked and holding.** Every other README sentence: the names, the binary, the `cli` module
(`canon-engineering/src/main.rs:30,33,70,73`), `prepare`, `collect`, `evaluate`
(`lib.rs:332,623,754`), "offline replay never calls this crate" (`evaluate` reaches only `sources`
and `canon_expr::evaluate`), the process and socket claims (`process.rs:171,184`,
`network.rs:28,59,64`), release 0.2.0 (`CHANGELOG.md:563`), the three relative links.

"Frozen at the behaviour of tag 0.16.0" is true as written, and the listings do not show it:
between the tag and ba94af2 the only source change in either crate is the `include_str!` path, the
catalog file is the same blob at both paths, and `tests/`, `generated/` and `ess/` are unchanged.
The listings show signatures only, not the value of `BUILTIN_CATALOG`, attributes or bodies; the
behaviour claim rests on that source diff.

The listings are what they claim: the build logs show two distinct trees, each finished; both
status files read exit 0; nine spot-checked items are present. Nothing in the tree assumes the
crates are published. The acceptance holds clause by clause; open is only the evidence record,
which is the coordinator's.

```findings
- file: crates/canon-engineering-docs/src/status.rs
  line: 108
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: the generated status page still lists AEP acceptance and Kubernetes providers as planned, which the freeze on new providers makes false, and no story in wave w3 covers it
- file: website/docs/guides/assertions.md
  line: 11
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: the assertion guide that README.md links to still promises AEP acceptance, Kubernetes collection and later k8s catalog functions and never mentions the freeze
- file: crates/canon-engineering-assertions/README.md
  line: 16
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the reason given for pins moving, a shared lockfile, is wrong because the ESS pin is in ess/ and generated/ and the Canon pin is the tag in the crate's own Cargo.toml
- file: crates/assertion-providers/README.md
  line: 13
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the same shared-lockfile reason is wrong here too, since this crate's ESS pin is generated/types.rs and not Cargo.lock
- file: CHANGELOG.md
  line: 7
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: Unreleased has no entry for publish = false or the freeze of the two assertion crates, so the next release notes omit them
```
