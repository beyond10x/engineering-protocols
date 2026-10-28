---
format: aep.planning-md/3
id: approval-record:gate-link-identity-and-script-name
kind: approval-record
status: approved
title: An executable link's name and target both bind the context identity; a script runs under the link's name
summary: decided on the second adversary pass of the link fix
relations:
- decides: story:gate-executable-identity-bound-to-spawn
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T14:48:55Z", actor: "human:timo", revision: 3}
---
# Approval: an executable link's name and target both bind the context identity; a script runs under the link's name

## Decision

Decided 2026-10-28 by the orchestrator, under the owner's delegation, on the adversary's question after the second pass on story:gate-executable-identity-bound-to-spawn:

- The context identity binds both the name a command is invoked under (`argv[0]`, the admitted link) and the digest of the file it reached, per binding: two commands that swap their links get a different identity.
- A script reached through a link runs under the link's name, because multiplexing tools dispatch on it. This is documented in the story and in the assertions guide.

## Consequence

A program reached through a link is spawned at the file admission resolved, with the link as `argv[0]`. A script is spawned through the link (its interpreter reads the path it is given), so for a script the digest and link checks before and after each request are the binding, not the spawn path. The adversary's cases in `crates/canon-engineering-assertions/tests/executable_link_adv2.rs` hold both.
