# Wave plan: the data-only catalog

Plan for `epic:data-only-catalog`, derived on 2026-11-21 from `aep plan artifact waves --kind story`
(aep 0.80.0) over the epic's seven stories. Decision: `approval-record:data-only-catalog`. The
`sdc` session decides for the operator in this repository's session
(`approval-record:sdc-orchestrates-engineering-protocols-2026-11-21`); its standing decision of
2026-11-21 is that the epic's stories are accepted once the critic panel and scoping are done and
that each next wave opens in this order without a further question.

| Wave | Stories | Serves | State |
|---|---|---|---|
| 1 | `story:catalog-manifest`, `story:assertion-catalog-path` | O2 | [2026-11-21-w1](2026-11-21-w1-catalog-manifest.md), delivered 2026-11-21 |
| 2 | `story:catalog-release-archive` | O2 | [2026-11-21-w2](2026-11-21-w2-catalog-release-archive.md), delivered 2026-11-21 |
| 3 | `story:assertions-frozen-at-0-16-0`, `story:docs-generator-reads-files` | O2 | [2026-11-21-w3](2026-11-21-w3-frozen-assertions-docs-generator.md), delivered 2026-11-21 |
| 4 | `story:agents-boundary-data-only` | O2 | planned |
| 5 | `story:protocol-check-tool` | O2 | planned; waits on `upstream-blocker:consumers-compile-against-canon-engineering` |

A wave closes at its merge into `main`. A release is not part of a wave: releases, tags and
version numbers stay with the operator.

## Backlog

Drafts filed from findings while this plan runs are listed here. None is taken into a wave of
this plan.
