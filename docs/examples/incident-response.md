# Example — Incident Response

```text
Case INC-492

Claims:
  impact.bounded = TRUE
  service.healthy = FALSE
  cause.identified = UNKNOWN
  rollback.verified = UNKNOWN

Urgent obligation:
  restore_service

Actions:
  metrics.inspect
  logs.search
  release.inspect
  traffic.shift [approval]
  release.rollback [approval]
```

After successful rollback and fresh health evidence:

```text
service.healthy = TRUE
impact.bounded = TRUE
cause.identified = UNKNOWN
rollback.verified = UNKNOWN
```

The operational incident can leave emergency mode while the investigation remains open:
`restore_service` is discharged, and `investigate_cause` stays open until a cause analysis of a
current revision of the service or the release identifies the cause. `rollback.verified` stays
`UNKNOWN` throughout: no verifier other than the one who rolled back has reported the rollback
complete.

This demonstrates why a protocol is richer than one scalar workflow state.
