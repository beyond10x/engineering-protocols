# Engineering assertions

This crate (`b10x-canon-engineering-assertions`, library `canon_engineering_assertions`) is the
edge around Canon's pure assertion evaluator. It holds the typed assertion catalog, the
`engineering-gates/1` document model, collection planning (`prepare`), observation collection
(`collect`, through `b10x-assertion-providers`) and offline replay of retained evidence
(`evaluate`). The `canon-engineering assertions` and `canon-engineering gates` commands are its
`cli` module.

## Frozen at tag 0.16.0

The crate is frozen at the behaviour of tag 0.16.0. It stays in the tree, builds, passes its tests
and is marked `publish = false`. Frozen means:

- No new assertion function, provider, CLI command or wire-model field.
- A Canon or ESS pin still moves when the workspace's does, because the crates share its lockfile.
- A security defect is still fixed.
- This README is corrected when what replaces the crate exists.

Why: this repository is becoming a data-only catalog of protocols. Offline replay and the
collectors that start programs and open sockets have no other home yet, so the crate stays until
one exists. `publish = false` stops a crates.io release; it does not affect consumers that depend
on the crate by Git tag.

The owner of what replaces it is Loom: observation collection and gates as executors through
Substrate. That design has not started. The decision is recorded in
`decision-blocker:assertion-executor-owner`, in this repository's plan store under
`.engineering/`. Deleting the crate is a later change, after the replacement exists.

The same applies to its dependency `b10x-assertion-providers`, which has its own
[README](../assertion-providers/README.md).
