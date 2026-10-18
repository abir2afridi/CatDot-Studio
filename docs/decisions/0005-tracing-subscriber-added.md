# ADR 0005 — add tracing-subscriber now (supersedes ADR 0003)

Context: ADR 0003 deferred tracing-subscriber to Phase 1. Reconsidered while
closing Phase 0: Part K lists "logging" in Phase 0, and without a subscriber
`tracing` spans are silently dropped — no real logging exists.
Decision: add `tracing-subscriber 0.3.23` (env-filter + fmt to stderr) and
wire `diagnostics::init_tracing` to `try_init()`. ADR 0003 is superseded.
Alternatives: keep no-op — rejected, Phase 0 would ship fake logging.
Consequences: in-app log viewer + rotation + a redacting subscriber Layer
remain Phase 1 work (secrets are not logged today; diagnostic bundle output
is redacted and tested).
