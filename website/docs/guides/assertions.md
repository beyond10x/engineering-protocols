---
title: Deterministic assertion gates
sidebar_position: 4
description: Author typed engineering checks, collect observations, and replay retained evidence.
lede: A gate passes only when every assertion evaluates true against its bound observations.
source: crates/canon-engineering-assertions; crates/assertion-providers; catalog/engineering.yaml
---

Assertion gates are available in source release 0.2.0. They use the generic `canon-expr/1` language
from [Canon](https://beyond10x.github.io/canon/) ([GitHub](https://github.com/beyond10x/canon)).
Engineering Protocols owns the catalog, process registration and collection. AEP acceptance
objects and automatic task offboarding are **planned** integrations. Kubernetes collection is
also planned; this version does not contact clusters.

## Write and validate a gate

Create `.engineering/gates.yaml`:

```yaml
format: engineering-gates/1
language: canon-expr/1
gates:
  - id: repository-ready
    text: The repository has its manifest and documents its license.
    assertions:
      - 'fs.file.exists("Cargo.toml")'
      - 'text.contains("README.md", "Apache")'
```

```console
canon-engineering gates validate
canon-engineering gates explain
canon-engineering gates run
canon-engineering gates evaluate --evidence .engineering/assertions/evidence.json
```

`validate` parses, checks signatures and resolves recipes before collection. `explain` emits the
checked plan, deduplicated requests and implementation identities. `run` collects and retains
observations, then evaluates. `evaluate` replays that retained snapshot without invoking a
provider or reading its source files. An omitted replay time uses the retained evaluation instant.
To check expiration, supply `--now` in Unix seconds. Supply `--source-identity` and
`--context-identity` when a caller requires a particular snapshot. Replay alone makes no claim
about the present workspace or network.

Exit codes are 0 for true, 1 for false, 2 for invalid input/provider data or IO errors, and 3 for
unknown. Empty gates and empty assertion lists are invalid. Every gate's assertions must be true;
a readable `text` field explains their intent but is not itself executable verification.

## Expressions and observations

The lexer is Logos; Canon maintains a small Pratt parser, separate type checker and request
planner, and a pure evaluator. A new catalog function does not extend the grammar. Operators,
precedence, budgets and exact numeric semantics are versioned by `canon-expr/1`.

```text
file_exists("Cargo.toml")
doc.string("package.json", "/name") == "my-package"
tests.report("reports/tests.json").inventory == ["login", "logout"]
generated.matches("expected", "generated")
proj.tests.run($scope).failed == 0
```

Variables are explicit project bindings (`$scope`). Strings use JSON double quotes. The language
supports booleans, exact integers/decimals, lists and records; fields, indexes, comparisons,
`in`, `not`, `and`, and `or`. Comparisons cannot be chained. `not a == b` means `not (a == b)`.
There is no implicit truthiness, arithmetic beyond unary minus, shell interpolation or user loop.
Integer/decimal comparison never rounds through a floating-point number. JSON document readers
preserve numeric lexemes; YAML document readers refuse fractional numbers in this version.

A missing observation or timeout is unknown. Known absence in an optional value is distinct from
unavailable evidence. Invalid provider data is a hard error, even if a boolean branch could hide
it. Boolean operators do not guard acquisition: all planned observations are collected. An
observation's arguments cannot depend on another observation. Generate inputs before collection;
commands that change relevant source during collection invalidate the run.

Source identity includes dirty and untracked files. Build/cache directories and retained evidence
are excluded from the broad snapshot. File, schema, report and generated-tree arguments are also
hashed explicitly, including ignored files. External providers declare additional inputs through
project `inputs`. Catalog definitions, bindings and executable bytes enter the plan/context identity.
Evidence binds those identities, has an observation time and expires. This records an observation;
it does not make an arbitrary report producer trustworthy.

## Admit commands explicitly

Project files use `engineering-assertions-project/1`. Commands are executable paths, argument
arrays and explicit environment maps. Paths resolve relative to the source root; there is no
implicit PATH lookup or inherited environment. Shell source is not an expression feature.

```yaml
format: engineering-assertions-project/1
variables:
  scope: {kind: string, value: unit-tests}
commands:
  unit-tests:
    executable: .tools/test-report-producer
    args: [unit]
    env: {}
timeout_seconds: 30
freshness_seconds: 300
```

An `executable` that is a symbolic link is resolved once, at admission. A program it reaches is
what is digested and what runs, under the link's path as `argv[0]`, so a multiplexing binary such as
rustup's `cargo` proxy (`/usr/bin/cargo` linked to `rustup`) runs as `cargo`. A script it reaches
(a file starting with `#!`) runs through the link, because its interpreter reads the path it is given
and names it `$0`; the digest and link checks before and after each request are what bind it. A link
retargeted after admission refuses collection, and each command's link and the file it reached enter
the context identity.

Pass this file with `--project .engineering/assertions-project.yaml`. `proj.tests.run($scope)`
expects the admitted Rust producer to execute the suite and emit one JSON document:

```json
{"format":"engineering-tests/1","inventory":["login","logout"],"passed":2,"failed":0,"skipped":0}
```

The process exit and counts must agree; counts must match unique inventory, and execution cannot
be zero. Assert the expected inventory when completeness matters: a smaller passing suite is not
the same suite. `tests.report(path)` and `tests.passed(path)` inspect an existing report. They do
not rerun tests or independently prove the report was produced for the current code.

`process.succeeded(binding)` handles ordinary exit-status checks. Bind it to explicit ESS
validate/compile/synthesize commands when those steps are your gate. `ess.report(path)` reads an
actual `ess-conformance-report/2`; its conformance recipe also requires expected specification and
implementation identities. ESS is [documented here](https://beyond10x.github.io/ess/)
([GitHub](https://github.com/beyond10x/ess)).

Codegate's adapter reads `codegate-dependency-report/0.1`, including coverage and source/configuration
identities. The clean recipe requires those expected identities. `codegate.score` and regression
scores are not implemented: the existing producer does not expose them. A future producer can
declare a typed Decimal result and use ordinary assertions such as `quality.score() > 0.8`.

## Extend the catalog

Project `catalogs` lists YAML files in `canon-catalog/1` format. Entries cannot replace existing
names. Recipes compose existing functions through declared parameters without capturing ambient
variables. For example:

```yaml
format: canon-catalog/1
functions:
  project.manifest.present:
    parameters: []
    returns: {kind: scalar, value: bool}
    implementation:
      kind: recipe
      value: 'fs.file.exists("Cargo.toml")'
```

Typed external operations use the same descriptors with `implementation.kind: observation`.
The descriptor's provider identity, operation and SHA-256 must match explicit project `providers`
registration. The admitted Rust executable exchanges exactly one
`engineering-provider-request/1` / `engineering-provider-response/1` JSON envelope over stdio;
`arguments_json` and `value_json` carry JSON text which is checked against the descriptor's types.
Malformed/extra output is rejected. The included `assertion-provider-example` demonstrates this
contract; the extension integration test builds a separate Rust provider and uses an unchanged
engine to collect and replay a new `acme.ready` namespace.

Processes run in their own process group with bounded output and timeout. This supervision is
implemented on Unix in v1. The default is ten seconds and 4 MiB per output stream; project bounds
are at most 300 seconds and 16 MiB. Collection also has a total elapsed-time budget. This is
explicit local execution admission, not a process sandbox. Remote engineering integrations should
be exposed by an admitted Connectors-based provider.

`canon-engineering assertions list` and `assertions describe <name>` expose released signatures.
The [generated reference](../reference/assertions.md) contains every builtin and recipe. New
signatures and changed recipe semantics change plan identities. Grammar/operator changes require
a new language version; provider/catalog evolution does not require editing the parser.

## Networking and later Kubernetes checks

```text
net.host.resolves("api.example.org", resolver: $resolver)
net.tcp.reachable("api.example.org", port: 443, network: $network)
```

DNS requires an explicit resolver IP and port (for example `192.0.2.53:53`). It checks A/AAAA
answers, follows returned aliases, and retains addresses and the minimum TTL. A definitive negative
answer is false; timeout/server failure is unknown. Negative answers have zero replay lifetime.
Truncated UDP answers are unavailable in v1; automatic TCP retry is not implemented.
TCP requires a named network context whose host-to-IP mapping is explicit. Connection refusal is
false; timeout is unknown. These are observations of a changing environment, so collection itself
is not deterministic; evaluation of retained observations is.

The repository's `fixtures/assertions/network.yaml` and `network-project.yaml` show the complete
configuration with documentation-only addresses. Replace those addresses with your environment
before running them.

Later catalogs can expose `k8s.namespace.exists(...)` and a deployment observation with
`.rollout_complete`. Those providers must bind cluster/context, namespace, resource UID,
resourceVersion and observed generation, and retain freshness. There is no Kubernetes provider
or AEP lifecycle change in this version.
