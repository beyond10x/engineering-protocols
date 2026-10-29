---
format: aep.planning-md/3
id: story:story-delivery-2-red-after-the-latest-write
kind: story
status: active
title: A red gate or CI counts only a red run after the latest write
summary: a late run about an older revision, or the red run of tests written earlier, does not admit implementation.edit; needs a Canon order form over the latest record or over revisions
relations:
- informed_by: story:story-delivery-2-red-gate-needs-red-tests
- decomposes: epic:gates-and-delivery-for-any-repository
- serves: vision:governed-autonomy
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T09:19:30Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-29T09:19:30Z", actor: "human:timo", revision: 3}
---
## Outcome

Under `story.delivery/2` with `tests_first=required`, a red gate or a red continuous integration
admits `implementation.edit` only when a run of the story's tests failed at a revision that holds
them, after the latest time they were written. A late failing run about a revision from before the
write, and a red run of tests written earlier, no longer count.

## Context

The adversary pass on story:story-delivery-2-red-gate-needs-red-tests found two histories its rule
still admits, both admitted before that rule as well:

- `after` orders records by when they were observed, not by revision, so a late failing run about
  the seed revision (which holds none of the story's tests) counts as the tests' red run
  (`a_late_fail_about_the_seed_still_admits_the_edit_on_a_red_gate`).
- `after` anchors on the first `tests_change`, so tests written again later that pass on their
  first run reach the edit on the strength of the earlier tests' red run
  (`tests_written_again_and_never_red_are_still_admitted_by_a_red_gate`).

Both cases are in `crates/canon-engineering/tests/story_delivery_2_red_tests_adv2.rs` and assert
today's verdict. Neither history was observed at runtime. Canon 0.7.0 has no order over revisions
and no form anchored on the latest record of a kind, so this needs a Canon release first.

## Depends on

- A Canon order form anchored on the latest record of a kind, or an order over revisions.

## Acceptance

- Both cases above assert `blocked` and pass.
- Every other `story.delivery/2` fixture and test passes unchanged.
