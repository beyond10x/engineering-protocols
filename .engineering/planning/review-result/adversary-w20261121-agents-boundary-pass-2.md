---
format: aep.planning-md/3
id: review-result:adversary-w20261121-agents-boundary-pass-2
kind: review-result
status: active
title: Wave 2026-11-21-w4 adversary, story:agents-boundary-data-only, pass 2
relations:
- reviews: story:agents-boundary-data-only
revision: 1
---
unit: story:agents-boundary-data-only, commit 2cafe04 on impl/agents-boundary-data-only (pass 2, the correction only)
verdict: CONFIRMED (two notes; no NEEDS-CHANGE)
cases: catalog_manifest executed 34→34, red 0 (run on a scratch copy of 2cafe04, 34 passed)
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 22 files in the adversary scratch under the coordinator's session scratch
needs-coordinator: none

ledger: carried 0 / new 2 / resolved 5

All five pass-1 findings are resolved. Two new findings, both notes about how strong the new
tests are; the code is correct in both cases. No case was added.

**Pass-1 findings**

| Pass 1 | Status | Measurement |
|---|---|---|
| The README check passes a changed file on other YAML forms of the manifest (`README.md:80`) | resolved | those forms are refused by the gate as `LineShape`, with a case each; no accepted form was found that gets past the check |
| Windows default `autocrlf` breaks the tag route | resolved | a clone of 2cafe04 with `-c core.autocrlf=true`: README.md comes out CRLF, the manifest, vocabulary and schema LF; the check prints 11 OK, exit 0 |
| The binary was said to be replaced by a CI check | resolved | `README.md:89-93` and `AGENTS.md:60-63` match story:protocol-check-tool's "Decided here" |
| `canon` was stated as a fact | resolved | `README.md:50-52` says hand-declared, only a value above the pinned Canon is refused, no guarantee |
| ADR 0094 decision 4 was not mentioned | resolved | `AGENTS.md:51-54` quotes decision 4 accurately and says it is not decided here |

**`line_pairs` against the real awk.** 16 manifest variants, each run through the gate and then
through the README's check with one byte appended to a protocol and to the vocabulary. Accepted
by the gate and still caught by the check (exit 1, 2 FAILED): an id-first entry, tabs after the
colon, extra and trailing spaces, `%YAML`/`---`/`...` markers, comment lines containing
`sha256:` and `- path:`, sha256 as the last line with no final newline, a sequence at column 0.
Refused by the gate: as `LineShape`, the vocabulary after the protocols, CR-only line endings,
U+2028 and NEL as line breaks; as `Manifest`, a duplicate key, NBSP after the colon, a second
YAML document, a tab after `-`; as `Digest`, an empty `sha256:`. No accepted text makes
`line_pairs` and gawk read different pairs. Measured with gawk only.

**Mutants** against the 34 cases: dropping the `- path:` rule, comparing counts only, trimming
`\r` and ignoring the vocabulary are each killed; comparing as sets survives.

**`.gitattributes`.** `git check-attr text` gives `unset` for the manifest, the vocabulary, a
nested protocol file and the schema, and `unspecified` for README.md. `git ls-files --eol` shows
`i/lf w/lf attr/-text` for all 13 files. `git archive` from the CRLF clone exports the manifest
with LF.

**Findings**

| Finding | file:line | Measured | What reaches it | Verdict | Origin |
|---|---|---|---|---|---|
| The "in order" part of `LineShape` is tested by no case: comparing as sets passes all 34 | `crates/canon-engineering/tests/catalog_manifest.rs:235` | mutant m2 survives; the real code refuses a manifest with the vocabulary after the protocols, which the README check handles correctly | no manifest needs it | CONFIRMED | introduced |
| `catalog_files_are_checked_out_byte_for_byte` only checks that two lines are present in `.gitattributes`, not how files check out | `crates/canon-engineering/tests/catalog_manifest.rs:1150` | with `*.yaml text eol=crlf` appended in a clone both lines are still present and the test's rule holds, but the catalog files check out CRLF and the README check exits 1 | a later attribute line that overrides `-text`; nobody has written one | CONFIRMED | introduced |

```findings
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 235
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: comparing the line pairs as sets instead of in order passes all 34 cases, so the in-order part of LineShape is tested by no case, though order does not affect the README check
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 1150
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: catalog_files_are_checked_out_byte_for_byte only checks that two lines exist in .gitattributes and stays green when a later line sets text eol=crlf, which makes catalog files check out CRLF and the README check exit 1
```
