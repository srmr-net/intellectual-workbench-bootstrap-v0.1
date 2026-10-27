# ADR-005 independent reviews

## Review A — Agent/data/security

- [ ] Provider sees only the minimum user request and admitted bounded candidate metadata; no credentials, paths or unrelated room history.
- [ ] All candidate text is treated as data; prompt injection cannot manufacture executable tool calls, permissions or UI code.
- [ ] Strict JSON parsing rejects duplicate keys, unknown fields, excessive output and forged candidate IDs.
- [ ] Provider transport is opt-in, bounded, cancellable and has no active tools; deterministic offline mode exists.
- [ ] Approval binds to immutable proposal digest, exact room/revision, permission effects and expiry; no replay.
- [ ] Audit log is sufficient for reconstruction without unnecessary prompt retention; secrets are redacted.
- [ ] Tests actually exercise malicious metadata, forged room IDs, invalid proposal, approval replay and provider timeout.

## Review B — Registry/kernel/state integration

- [ ] Selection resolves only against offered, currently installed/admitted exact component pins.
- [ ] Real manifest/RDF/schema/UI-binding cross-check, no hardcoded mortgage port or digest.
- [ ] Host builds canonical accepted plan and reuses current validator and atomic apply API.
- [ ] Stale revision and catalog/grant changes reject without state mutation or implicit retry.
- [ ] Initialisation uses accepted ingress and differentiates topology commit from input commit.
- [ ] Concurrent room requests and cross-room permission/projection isolation verified.
- [ ] Ordinary UI edits and cold restore invoke no Operator/provider.
- [ ] Existing accepted tests pass unchanged.

For each reviewer record role, commit hash, exact command/tests, findings, remediation commits and sign-off. Reviews must be independent of the implementing agent and must not be inferred from automated test counts.
