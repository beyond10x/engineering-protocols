# Engineering assertion providers

This crate collects observations for the engineering catalog. Canon parses, checks, plans and
evaluates expressions separately. Offline replay never calls this crate. The library is
`assertion_providers`; the crate also builds one binary, `assertion-provider-example`.

## Frozen at tag 0.16.0

The crate is frozen at the behaviour of tag 0.16.0. It stays in the tree, builds, passes its tests
and is marked `publish = false`. Frozen means:

- No new assertion function, provider, CLI command or wire-model field.
- An ESS pin still moves when the workspace's does, and that edit lands in these files:
  `ess/providers/ess-inputs.yaml` and the header of `generated/types.rs` (regenerated). The ESS
  version the gate installs is in `.github/workflows/check.yml`. The crate has no Canon
  dependency.
- A security defect is still fixed.
- This README is corrected when what replaces the crate exists.

Why: this repository is becoming a data-only catalog of protocols. The collectors start programs
(`src/process.rs`) and open sockets (`src/network.rs`), and `b10x-canon-engineering-assertions`
calls them; they have no other home yet, so the crate stays until one exists. `publish = false`
stops a crates.io release; it does not affect consumers that depend on the crate by Git tag.

The owner of what replaces it is Loom: observation collection and gates as executors through
Substrate. That design has not started. The decision is recorded in
`decision-blocker:assertion-executor-owner`, in this repository's plan store under
`.engineering/`. Deleting the crate is a later change, after the replacement exists.

## Collection

`collect(operation, &BTreeMap<String, serde_json::Value>, &ProviderContext)` returns
`Known(value)`, `Unavailable(reason)`, or an error. Missing files, command timeout, inaccessible
resources and inconclusive network observations are unavailable. Invalid arguments, malformed
producer reports, schema violations and provider identity mismatches are errors. Neither certifies
a gate. `fs.file.exists` and `fs.dir.exists` return known false for absent paths.

`ProviderContext` explicitly binds the repository root, named argv/environment commands,
external processes, TCP host/IP contexts, a timeout and an output byte bound. Defaults are ten
seconds and four MiB; maximum admitted limits are five minutes and sixteen MiB. Process execution
requires Unix in v1. It uses argv directly with a cleared environment, supervises one process group,
and handles stdin/stdout/stderr using nonblocking pipes. Descendants retaining pipes cannot extend
the collection deadline. Process groups are killed when collection ends. This is resource
supervision, not an operating-system sandbox; registered programs possess the caller's authority.

| Operation | Arguments | Result |
| --- | --- | --- |
| `fs.file.exists`, `fs.dir.exists` | `path` | Boolean |
| `text.contains` | `path`, `needle` | Boolean |
| `json.read`, `yaml.read` | `path` | Native JSON value |
| `document.get` | `path`, `pointer`, optional `format` | JSON Pointer value; missing is null |
| `document.validates` | `path`, `schema` | JSON Schema validation Boolean |
| `generated.matches` | `expected`, `actual` | Exact relative file/directory inventory and content equality |
| `process.run`, `ess.run` | `binding` | `exit_code` (integer or null), `stdout`, `stderr` |
| `process.succeeded` | `binding` | Boolean; unavailable when signal terminated |
| `tests.report` | `path` | Normalized inventory, passed/failed/skipped/total/executed counts |
| `tests.run` | `binding` | Same report, read from bound process stdout; contradictory exit status is refused |
| `codegate.dependencies` | `path` | Normalized verdict/coverage, findings_count, fan_out_max, source_id, configuration_id |
| `ess.report` | `path` | Counts, inventory, conformance_status, spec_digest, implementation |
| `net.host.resolves` | `host`, `resolver` | `resolves`, `ttl_seconds`, `resolver`, `addresses` |
| `net.tcp.reachable` | `host`, `port`, `network` | Boolean |

Filesystem paths stay below the configured root, including symlink targets. Generated comparisons
refuse symlinks and special files, and include extra files and empty directories. JSON numeric
lexemes retain arbitrary precision. YAML floating-point values and custom tags are refused;
use JSON when observing exact decimals. Schema references are restricted to the current document;
HTTP and file resolution are disabled at compile time.

Normalized test input is `engineering-tests/1`, with `inventory: [unique test names]`, `passed`,
`failed`, and `skipped`. The count sum must equal inventory length and at least one test must have
executed. An imported report proves its recorded results; compare the expected inventory and
source identities or run the suite freshly when accepting current work. Codegate consumes the
actual `codegate-dependency-report/0.1` format; it does not invent a score. ESS consumes
`ess-conformance-report/2` and checks count/outcome consistency, selection coverage and qualification.
The consumer still binds the expected specification digest and implementation identity.

DNS uses explicit `IP:port` UDP resolvers and asks for both A and AAAA. It validates the complete
packet and name/record boundaries before a definitive negative. NXDOMAIN is known false, with
zero replay TTL. Address observations retain the minimum answer/CNAME TTL. Timeouts, server errors,
referrals, truncated replies requiring TCP, and unresolved CNAME chains are unavailable. TCP uses
only the selected context's explicit host/IP map; refusal is false, other connection failures
are unavailable. Provider observations do not make network state timeless.

External providers are registered by operation with identity, executable SHA-256, argv and explicit
environment. The executable digest is checked before launch. One request is written to stdin:

```json
{"format":"engineering-provider-request/1","operation":"example.echo","arguments_json":"{\"value\":true}"}
```

One response is read from stdout; diagnostics belong to stderr:

```json
{"format":"engineering-provider-response/1","status":"known","value_json":"true"}
```

Unavailable responses carry `reason` and omit `value_json`. Known responses omit `reason`.
The JSON strings are opaque transport carriers: the catalog validates their decoded argument and
result types. `assertion-provider-example` is a standalone Rust/clap implementation of the new
`example.echo` namespace; the conformance test registers it without changing the collector or
expression engine. The wire structs are generated by ESS 0.77.0 from `ess/providers`, and shared as
`assertion_providers::contract`.
