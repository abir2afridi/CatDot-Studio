# ADR 0003 — tracing without tracing-subscriber in Phase 0

Context: Part C/U3 want structured logs with tracing spans per job/task.
Decision: Depend only on `tracing` (macros/attributes) in Phase 0;
`diagnostics::init_tracing` is idempotent no-op wiring until Phase 1 adds
`tracing-subscriber` (fmt + rotation + in-app viewer) with its own dep
justification (what it replaces: ad-hoc println; license MIT; small; maintained).
Alternatives: add tracing-subscriber now — rejected to keep Phase 0 dep set
minimal while no log viewer consumes it.
Consequences: No log output regression risk; Phase 1 must add the subscriber
and rotation before any UI ships logs.
