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
  ([GitHub](https://github.com/beyond10x/loom)) runs agents, and its `loom-governor` crate depends
  on this one.
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

## Install

As a library (crate `b10x-canon-engineering`, imported as `canon_engineering`):

```toml
[dependencies]
b10x-canon-engineering = { git = "https://github.com/beyond10x/engineering-protocols", tag = "0.15.1" }
```

As a command:

```console
cargo install --locked --git https://github.com/beyond10x/engineering-protocols --tag 0.15.1 b10x-canon-engineering
```

Nothing is published to crates.io; releases are Git tags.

## Smallest example

The built-in protocols are embedded in the crate at build time. Listing them from a dependent crate:

```rust
use canon_engineering::registry;

fn main() {
    for (name, major) in registry::list() {
        println!("{name}@{major}");
    }
}
```

Against tag 0.15.1 this prints `eval-gated-change@1`, `incident-response@1`, `loop-improvement@1`,
`ml-experiment@1`, `protocol-adopt@1`, `self-improvement@1`, `software-change@1`, `story-delivery@1`,
`story-delivery@2` and `support-triage@1`.
`registry::get(name, major)` returns one protocol's YAML as released together with Canon's
validated model of it.

From a checkout, the command line does the same:

```console
$ cargo run -q -p b10x-canon-engineering -- protocols list
eval-gated-change@1
incident-response@1
loop-improvement@1
ml-experiment@1
protocol-adopt@1
self-improvement@1
software-change@1
story-delivery@1
story-delivery@2
support-triage@1
$ cargo run -q -p b10x-canon-engineering -- protocols show software-change@1
format: protocol/1

protocol:
  id: software.change
  revision: 1
...
```

A malformed `<name>@<major>` exits 2; a well-formed one that names no built-in exits 1.

## Assertion gates

The CLI also checks `.engineering/gates.yaml` using typed assertions. `canon-engineering
assertions list` discovers the catalog; `gates validate`, `gates run` and `gates evaluate` validate
checks, collect observations and replay retained evidence. See the [assertion guide](website/docs/guides/assertions.md)
for a complete gate and commands. These commands are included in release 0.2.0.

The generic expression core lives in Canon. This repository supplies files/documents, explicit
process bindings, test reports, ESS and Codegate dependency reports, DNS and TCP observations,
recipes and typed external Rust providers. Collection performs IO; retained evaluation is pure.
Kubernetes providers and AEP acceptance integration remain planned.

## Crates

| Crate | Holds |
|---|---|
| `b10x-canon-engineering` | Library `canon_engineering` (protocol registry, engineering vocabulary) and the `canon-engineering` binary |
| `b10x-canon-engineering-assertions` | Typed catalog, gate CLI, collection planning and retained replay |
| `b10x-assertion-providers` | Bounded engineering observation collectors and external-provider transport |
| `canon-engineering-docs` | Generator for the site's protocol pages, protocol graphs, vocabulary page and status record; not released |

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

Latest source release 0.15.0 (2026-10-29), using Canon 0.8.0 and ESS 0.77.0; see [CHANGELOG.md](CHANGELOG.md).

## Licence

Apache-2.0, see [LICENSE](LICENSE).
