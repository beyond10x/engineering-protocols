---
format: aep.planning-md/3
id: story:protocol-adopt-authority-and-floor
kind: story
status: draft
title: 'protocol.adopt/1: authority in the outcome, order against the check report, the floor as an artifact'
summary: 'Close what adversary pass 1 on protocol.adopt/1 left open: a denied grant with a listed approver, an approval before the check report, and a moved floor.'
relations:
- serves: vision:governed-autonomy
- depends_on: story:protocol-adopt-protocol
revision: 1
---
## Outcome

`protocol.adopt/1` closes the gaps adversary pass 1 on story:protocol-adopt-protocol left open
(ec0121e closed its findings 1-3):

1. **Authority in the outcome.** Canon outcomes are a predicate or an explicit decision, never both
   (canon `crates/canon/src/model/requirement.rs`), and no predicate reads an authority decision. So
   `adopted` cannot require the `protocol.adopt` grant itself; it rests on an `adoption_approval`
   record from a producer the case lists on `different_principal`. A listed approver's record with
   `protocol.adopt` denied still adopts. Closing it needs a Canon form that reads authority (or an
   outcome that combines a decision with a predicate), then a revision of this protocol.
2. **Order against the check report.** `adoption.approved` needs the first `diff_report` before
   the first approval (`before`); an approval between the diff report and the check report still
   counts. `since` and `after` over both reports put `canon check` over `STATE_BOUND` (2.8M and
   561,600 states against 65,536), so this waits for a cheaper order form or a larger bound.
3. **The floor as an artifact.** The organization floor is not declared, so a `diff_report` stays
   valid after the floor moves. Declaring it as an artifact with an invalidation rule over
   `floor.respected` follows Canon's floor construct (canon https://github.com/beyond10x/canon/issues/5).

## Source

Adversary pass 1, findings 2, 4 and 5 (`crates/canon-engineering/tests/adversary_protocol_adopt.rs`,
e4b8d48); story:protocol-adopt-protocol § Delivered.

## Acceptance

Each closed point has a fixture under `fixtures/protocol-adopt/` that fails against today's
protocol and passes against the revision: a listed approver with `protocol.adopt` denied adopts
nothing; an approval between the diff report and the check report adopts nothing; a diff report
against a floor revision that is no longer current leaves `floor.respected` UNKNOWN.
