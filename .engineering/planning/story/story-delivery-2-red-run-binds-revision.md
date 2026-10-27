---
format: aep.planning-md/3
id: story:story-delivery-2-red-run-binds-revision
kind: story
status: draft
title: The red run that admits an edit is bound to its tests_change revision and persists across edits
tags:
- priority:medium
relations:
- decomposes: epic:gates-and-delivery-for-any-repository
- informed_by: story:story-delivery-2-with-settings
revision: 1
---
# Story: story-delivery-2-red-run-binds-revision

## Outcome

In `story.delivery/2` under `tests_first: required`, the failing test run that admits `implementation.edit` is bound to
the revision its `tests_change` produced and stays valid across later edits, so a second edit does not need a new
failing run at each new revision.

## Context

- The review of `story:story-delivery-2-with-settings` showed that Canon 0.6.0's `first` and `before` read the first
  record of a kind at any revision, with no revision-bound or latest-record order form.
- The released protocol therefore reads `tests.shown_red` at the current revision: after an edit moves the revision,
  the next edit waits for a test run there.

## Acceptance

1. A fixture where tests are written at r1, fail at r1, an edit makes r2, and a second edit at r2 is admitted with no
   new test run.
2. A failing run about a revision other than the `tests_change` revision still admits nothing.

## Depends on

A Canon order form bound to a revision, or a latest-record form.
