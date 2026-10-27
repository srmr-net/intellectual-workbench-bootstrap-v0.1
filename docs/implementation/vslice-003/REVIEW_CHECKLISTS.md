# VSLICE-003 — Independent Review Checklists

Each review must be completed by an independently scoped reviewer or agent that did not author the relevant implementation. Archive findings, commit reviewed, dispositions and retest evidence. A green CI run alone is not a review.

## Review A — Operator and security boundary

- [ ] Provider has no privileged direct tools, install, kernel write, or implicit model authority.
- [ ] Only admitted local candidates are offered; catalog is bounded and sanitised; metadata injection is treated as data.
- [ ] Strict parsing and limits; invalid keys, fields, unexpected permission requests and unit mismatches fail closed.
- [ ] Exact identity/version/digest is re-resolved during confirmation; drift produces no mutation.
- [ ] Approval is human originated, bound to proposal/room/revision/expiry, single-use; rejection and double-click safe.
- [ ] Prompt privacy, per-room isolation, no secret persistence, redacted logs and proper provider timeout/cancellation.
- [ ] Scope remains one component, local-only, no guest privilege change, no new WIT/ABI.

## Review B — Desktop, kernel, persistence and numerical integration

- [ ] Desktop uses actual accepted Operator pathway, not a fake fixture-only screen.
- [ ] Real Qt thread ownership, focus and accessible statuses/controls/diagnostics verified.
- [ ] Preview does not mutate state; apply uses accepted canonical plan and kernel revision fences.
- [ ] Typed ingress and real WASM produce the reference vector; QML does not calculate it.
- [ ] Two-stage partial failure is described truthfully; no invalid half-commit of computed result.
- [ ] Post-assembly edits use zero provider invocations; stale projections cannot corrupt switched rooms.
- [ ] Offline cold restart restores pinned identity and committed state without model/guest for display.
- [ ] Test artifacts/logs/hashes prove real installed package; retained regression suites pass.

## Sign-off

For each reviewer: identity/role, date, reviewed commit, checklist result, severity of each finding, fix commit, validation command and final approval. Unresolved critical/high findings block acceptance.
