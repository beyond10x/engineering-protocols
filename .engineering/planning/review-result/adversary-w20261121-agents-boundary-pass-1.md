---
format: aep.planning-md/3
id: review-result:adversary-w20261121-agents-boundary-pass-1
kind: review-result
status: active
title: Wave 2026-11-21-w4 adversary, story:agents-boundary-data-only, pass 1
relations:
- reviews: story:agents-boundary-data-only
revision: 1
---
unit: story:agents-boundary-data-only, commit 361cdcb on impl/agents-boundary-data-only (base b0d38a0)
verdict: CONFIRMED (strongest; no NEEDS-CHANGE)
cases: n/a (suite not run)
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 7 files in the adversary scratch under the coordinator's session scratch
needs-coordinator: upstream-blocker:consumers-compile-against-canon-engineering says Loom pins tag 0.15.2; Loom's four Cargo.toml files pin 0.16.0 today. AGENTS.md cites that record and names no tag.

Five problems, all introduced by this unit, none NEEDS-CHANGE. Every acceptance clause is met on
the commit. No test was added and no suite ran. One cargo command:
`cargo run --locked --bin canon-engineering -- assertions list`, exit 0, 932 lines; the `--bin`
form resolves in this two-binary workspace.

**Findings**

| Finding | file:line | Measured | What reaches it | Verdict | Origin |
|---|---|---|---|---|---|
| The manifest check passes a changed file when the manifest is written in another YAML form of the same document | `README.md:80` | each edit leaves the parsed YAML unchanged: an entry starting `- sha256:` plus one byte appended to eval-gated-change gives 10 OK, exit 0; the vocabulary as `{path, sha256}` on one line plus a changed vocabulary gives 10 OK, exit 0; a quoted digest plus a changed software-change gives "1 line is improperly formatted", exit 0 (exit 1 with `sha256sum --strict`). The gate parses the manifest with serde and checks no line shape | no current or released manifest; a later hand edit would, since the manifest is hand-written | INFEASIBLE | introduced |
| The tag route fails with Git for Windows' default `core.autocrlf=true`, and "names the file with FAILED" does not hold there | `README.md:58-60`, `80-84` | a clone with `-c core.autocrlf=true` checks out the manifest and the vocabulary with CRLF; the check prints only "no properly formatted checksum lines found", names no file, exit 1; the repository has no `.gitattributes` | a Windows reader whose Git keeps the installer's default | CONFIRMED | introduced |
| "the `canon-engineering` binary [is] still in the tree until a CI check replaces them": story:protocol-check-tool moves the binary to `b10x-canon-engineering-assertions` under the same name, and only the crate goes; AGENTS.md:53-54 implies the same | `README.md:88-89` | the story's "Decided here" | any reader; `README.md:96`'s `cargo run --bin canon-engineering` stays valid after the move | CONFIRMED | introduced |
| `canon` is called "the lowest Canon release that reads the file" as a fact; it is hand-declared, checked only against the pinned Canon, and an understated value is not detected | `README.md:49-50` | `catalog_manifest.rs:8-10`; `AGENTS.md:91` | a consumer who chooses its Canon from `canon` | CONFIRMED | introduced |
| Only ADR 0094 decision 2 is called replaced; decision 4 ("a protocol change in a runtime is a pin bump, so the pin files … are on the runtime's judge surface") also assumes the crate pin | `AGENTS.md:49-51` | Atlas `architecture/adr/0094…md:33-34`; no Atlas ADR mentions data-only, the manifest or the archive, so "not yet recorded in Atlas" is true; the approval-record names only decision 2 | an agent reading ADR 0094 for the runtime rule | CONFIRMED | introduced |

**Acceptance, clause by clause: all met.** The grep prints nothing on 361cdcb and 9 lines on the
base. The section is `README.md:45-89`. Archive route on the two files from `target/catalog`:
`sha256sum -c` 0, `mkdir` 0, `tar` 0, check 0. Tag route on a shallow clone of the branch: 11 OK,
exit 0. The curl lines get 404 on 0.16.0; the URL shape matches real release assets. § Boundary
(`AGENTS.md:45-51`), the Loom row (no version; its crate names match Loom's four manifests) and
§ ESS (`AGENTS.md:172-176`) say what the acceptance asks.

**Held.** awk's 11 pairs equal an independent YAML reading and recomputed digests. A changed or
missing file gives FAILED and exit 1. An empty manifest and a tag without one exit 1 on GNU
coreutils 9.12 and `shasum -a 256`; uutils and busybox not measured. `gawk --posix` and
`shasum -a 256 -c -` give the same 11 OK. A BOM or trailing comments: 11 OK. A quoted path, or
`sha256:` before `path:` inside an entry, fails with exit 1. Tag 0.16.0 has no manifest and no
assets. The docs skill path exists at the new location. Nothing outside `docs/plan` points at the
removed headings. No other file tells a runtime to depend on the crate.

```findings
- file: README.md
  line: 80
  category: boundary
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: the awk check skips an entry that starts with a dash before sha256, a flow-style vocabulary or a quoted digest, all accepted by the gate's serde parse, and then exits 0 with a changed file unchecked
- file: README.md
  line: 80
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: on a tag clone with core.autocrlf=true, the Git for Windows default, the check exits 1 saying no properly formatted checksum lines found and names no file, and the repository has no .gitattributes
- file: README.md
  line: 88
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the README says the canon-engineering binary leaves the tree when a CI check replaces the crate, but story protocol-check-tool moves it under the same name into the assertions crate, and AGENTS.md line 53 implies the same
- file: README.md
  line: 49
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: canon is stated as the lowest Canon release that reads the file, while the manifest test checks it only against the pinned Canon and an understated value goes undetected
- file: AGENTS.md
  line: 49
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: only ADR 0094 decision 2 is named as replaced, yet decision 4 (a protocol change is a Cargo pin bump) also assumes the crate pin the data-only decision removes
```
