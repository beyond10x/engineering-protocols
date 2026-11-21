# Engineering protocols

Engineering protocols write down, as data, what makes engineering work legitimate: which claims
must hold, which evidence counts and for which revision, which actions need authority, and which
outcomes a case can end in. Each protocol is a [Canon](https://beyond10x.github.io/canon/)
([GitHub](https://github.com/beyond10x/canon)) `protocol/1` document; Canon validates, compiles
and evaluates it, and this repository ships the protocols with the engineering vocabulary they use.

**Documentation: <https://beyond10x.github.io/engineering-protocols/>**

The rule that runs through every protocol is that `UNKNOWN` is not `FALSE`. Tests that passed on
revision R1 say nothing about R2, so a change now at R2 has `tests.pass = UNKNOWN` and merge stays
blocked until evidence for R2 exists.

## What it is not

- Not the meaning of a claim, an evidence item or an obligation, nor the evaluator. That is Canon.
- Not the live engineering record. That is [AEP](https://beyond10x.github.io/ecosystem/aep/)
  ([GitHub](https://github.com/beyond10x/aep)).
- Not an agent runtime. [Loom](https://beyond10x.github.io/loom/)
  ([GitHub](https://github.com/beyond10x/loom)) runs agents. It takes protocols from this
  repository; some of its crates, `loom-governor` among them, still compile against the crate
  `b10x-canon-engineering`, and Loom plans to read the protocol files instead.
- Not a source of time, network or model input: a protocol adds no clock, network or model call
  to Canon's evaluation.

## Protocols

| Protocol | Source | Page | Released in |
|---|---|---|---|
| `software.change/1` | [`protocols/software-change/1.yaml`](protocols/software-change/1.yaml) | [software change](https://beyond10x.github.io/engineering-protocols/docs/protocols/software-change/1/) | 0.1.0 |
| `incident.response/1` | [`protocols/incident-response/1.yaml`](protocols/incident-response/1.yaml) | [incident response](https://beyond10x.github.io/engineering-protocols/docs/protocols/incident-response/1/) | 0.1.0 |
| `ml.experiment/1` | [`protocols/ml-experiment/1.yaml`](protocols/ml-experiment/1.yaml) | [ML experiment](https://beyond10x.github.io/engineering-protocols/docs/protocols/ml-experiment/1/) | 0.4.0 |
| `eval.gated_change/1` | [`protocols/eval-gated-change/1.yaml`](protocols/eval-gated-change/1.yaml) | [eval-gated change](https://beyond10x.github.io/engineering-protocols/docs/protocols/eval-gated-change/1/) | 0.9.0 |
| `support.triage/1` | [`protocols/support-triage/1.yaml`](protocols/support-triage/1.yaml) | [support triage](https://beyond10x.github.io/engineering-protocols/docs/protocols/support-triage/1/) | 0.2.0 |
| `story.delivery/1` | [`protocols/story-delivery/1.yaml`](protocols/story-delivery/1.yaml) | [story delivery](https://beyond10x.github.io/engineering-protocols/docs/protocols/story-delivery/1/) | 0.10.0 |
| `story.delivery/2` | [`protocols/story-delivery/2.yaml`](protocols/story-delivery/2.yaml) | [story delivery 2](https://beyond10x.github.io/engineering-protocols/docs/protocols/story-delivery/2/) | 0.11.0 |
| `self.improvement/1` | [`protocols/self-improvement/1.yaml`](protocols/self-improvement/1.yaml) | [self-improvement](https://beyond10x.github.io/engineering-protocols/docs/protocols/self-improvement/1/) | 0.10.0 |
| `loop.improvement/1` | [`protocols/loop-improvement/1.yaml`](protocols/loop-improvement/1.yaml) | [loop improvement](https://beyond10x.github.io/engineering-protocols/docs/protocols/loop-improvement/1/) | 0.10.0 |
| `protocol.adopt/1` | [`protocols/protocol-adopt/1.yaml`](protocols/protocol-adopt/1.yaml) | [protocol adoption](https://beyond10x.github.io/engineering-protocols/docs/protocols/protocol-adopt/1/) | 0.15.0 |

The terms they are written in are [`protocols/vocabulary.yaml`](protocols/vocabulary.yaml),
rendered as the [vocabulary page](https://beyond10x.github.io/engineering-protocols/docs/vocabulary/).

## Take the protocols and check them

The protocols are files. A release carries them in two forms, a Git tag and an archive, and
[`protocols/manifest.yaml`](protocols/manifest.yaml) lists the vocabulary and every protocol file
with the sha256 of its bytes. For each protocol it also gives the id, the revision and `canon`, the
lowest Canon release its author declared the file needs; it is written by hand and the repository
only refuses a value above the Canon it pins, so it is no guarantee to choose your Canon by. The
commands below need `git`, `curl`, `tar`, `sha256sum` and `awk`; nothing is compiled.

Tag 0.16.0 has no manifest and no release archive. The first release that carries both is the one
after 0.16.0; `<version>` below is the tag of a release that does.

From the release tag, a shallow clone:

```sh
git clone --depth 1 --branch <version> https://github.com/beyond10x/engineering-protocols
cd engineering-protocols
```

From the release archive, the file and its digest, checked and unpacked:

```sh
curl -fLO https://github.com/beyond10x/engineering-protocols/releases/download/<version>/engineering-protocols-catalog-<version>.tar.gz
curl -fLO https://github.com/beyond10x/engineering-protocols/releases/download/<version>/engineering-protocols-catalog-<version>.tar.gz.sha256
```

```sh
sha256sum -c engineering-protocols-catalog-<version>.tar.gz.sha256
mkdir catalog
tar -xzf engineering-protocols-catalog-<version>.tar.gz -C catalog
cd catalog
```

Either way, in the directory that holds `protocols/`, check each file against the manifest:

```sh
awk '$1 == "path:" {p = $2} $1 == "-" && $2 == "path:" {p = $3} $1 == "sha256:" {print $2 "  " p}' protocols/manifest.yaml | sha256sum -c -
```

It prints `<path>: OK` for the vocabulary and for each protocol and exits 0. If a listed file is
missing or differs it names the file with `FAILED` and exits non-zero. It does not notice a file
under `protocols/` that the manifest does not list, and it does not check the schema the manifest
names, `schema/b10x.protocol-catalog.v1.schema.json`, which carries no digest.

The crate `b10x-canon-engineering`, which embeds the protocols at build time and exposes them as a
registry, is still in the tree. It is not the way to take the catalog, and it is planned to be
deleted. The `canon-engineering` binary is planned to move to `b10x-canon-engineering-assertions`
under the same name, with its `assertions` and `gates` commands unchanged and its `protocols list`
and `protocols show` commands ended; § Assertion gates below runs it from a checkout today.

## Assertion gates

The CLI also checks `.engineering/gates.yaml` using typed assertions. `canon-engineering
assertions list` discovers the catalog; `gates validate`, `gates run` and `gates evaluate` validate
checks, collect observations and replay retained evidence. From a checkout, run it as
`cargo run --bin canon-engineering -- assertions list`. See the [assertion guide](website/docs/guides/assertions.md)
for a complete gate and commands. These commands are included in release 0.2.0.

The generic expression core lives in Canon. This repository supplies files/documents, explicit
process bindings, test reports, ESS and Codegate dependency reports, DNS and TCP observations,
recipes and typed external Rust providers. Collection performs IO; retained evaluation is pure.

The two crates behind these commands, `b10x-canon-engineering-assertions` and
`b10x-assertion-providers`, are frozen at the behaviour of tag 0.16.0 and marked `publish = false`.
Frozen means no new assertion function, provider, CLI command or wire-model field; the Canon and
ESS pins still move with the workspace's, and a security defect is still fixed. The reason is that
this repository is becoming a data-only catalog, while offline replay and the collectors have no
other home yet. The owner of what replaces them is Loom (observation collection and gates as
executors through Substrate, design not started), recorded in
`decision-blocker:assertion-executor-owner`. See the READMEs of
[`canon-engineering-assertions`](crates/canon-engineering-assertions/README.md) and
[`assertion-providers`](crates/assertion-providers/README.md).

## Crates

| Crate | Holds |
|---|---|
| `b10x-canon-engineering` | Library `canon_engineering` (protocol registry, engineering vocabulary) and the `canon-engineering` binary |
| `b10x-engineering-vocabulary` | Library `engineering_vocabulary`: the reader of `protocols/vocabulary.yaml`. It embeds no data; `canon_engineering::vocabulary` re-exports it. Not released |
| `b10x-canon-engineering-assertions` | Typed catalog, gate CLI, collection planning and retained replay; frozen at tag 0.16.0 |
| `b10x-assertion-providers` | Bounded engineering observation collectors and external-provider transport; frozen at tag 0.16.0 |
| `canon-engineering-docs` | Generator for the site's protocol pages, protocol graphs, vocabulary page and status record, and the catalog archive; reads `protocols/` as files and does not depend on `b10x-canon-engineering`; not released |

## Documentation

The site at <https://beyond10x.github.io/engineering-protocols/> has the
[overview](https://beyond10x.github.io/engineering-protocols/docs/), one page per
[protocol](https://beyond10x.github.io/engineering-protocols/docs/protocols/), concept pages
(protocols and compositions, step order, capabilities and bindings, profiles) and the
[status table](https://beyond10x.github.io/engineering-protocols/docs/status/). In this tree,
[`docs/examples/`](docs/examples/) holds two worked cases, and
[`docs/design/`](docs/design/engineering-lifecycle-specification-design.md) holds the original
design, written under the project's earlier name.

## Build

Needs Rust stable and [Task](https://taskfile.dev/).

```console
task check        # fmt, clippy, tests, generated-page drift check
task site-build   # also needs Node 22; builds website/build
```

Changing the repository: read [AGENTS.md](AGENTS.md).

## Status

Latest source release 0.16.0 (2026-11-21), using Canon 0.9.0 and ESS 0.77.0; see [CHANGELOG.md](CHANGELOG.md).

## Licence

Apache-2.0, see [LICENSE](LICENSE).
