# VSLICE-001 — Independent Review Protocol

Reviewers should have no authorship responsibility for the core code they review. Capture reviewer identity (GitHub handle is sufficient), review commit SHA, review date, findings and retest evidence. A simple approval without explicit coverage is insufficient.

## Review A — Numerical and interface correctness
- [ ] Mortgage mathematical formula; r=0 special case; repayment frequency and percentage convention.
- [ ] Decimal precision and half-up publication; tricky near-zero rates; upper bounds and no overflow.
- [ ] Input/output JSON Schema constraints and guest domain validation.
- [ ] Cross-check numerical vectors using independent reference computation.
- [ ] Exact port names and schemas consistent across manifest/RDF/WIT/room routing.
- [ ] Document assumptions (no fees/taxes/APR, no financial advice).

**Reviewer / commit / findings / dispositions:** _fill after review_

## Review B — Security, durability and cross-boundary integration
- [ ] Actual registry admission, pin resolution, artifact re-verification and runtime bytes provenance.
- [ ] No guest execution during admission or discovery; no WASI or host imports.
- [ ] No metadata-derived privilege grant or mutable-path TOCTOU escape.
- [ ] Rejected payloads/output batches do not change committed room state.
- [ ] Generation/revision fences, correlation, room isolation and bounded queue behavior.
- [ ] Snapshot restoration requires exact pin; registry partial-install recovery.
- [ ] Check invalid/trapped guest recovery; resource limits; log redaction.

**Reviewer / commit / findings / dispositions:** _fill after review_

## Gate resolution
Each substantive finding: `ID | severity | affected contract | test introduced | fix SHA | reviewer retest | closed?`. Unresolved high/critical or acceptance-contract mismatches block milestone acceptance.
