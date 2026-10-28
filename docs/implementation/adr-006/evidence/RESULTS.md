# ADR-006 validation ledger — Proposed

Implementation source: `a9eb0db306d193309cac51faf75ec431efbf96fc` (initial
implementation87ff369; independent evidence-source hardening a9eb0db).
Accepted runtime/WIT/manifest/plan/UI/ontology and existing source bytes preserved.

- New locked native suite:24 conformance tests +2 unit tests passed (default and
  mock-feature builds). Actual local subprocess cancellation/reaping is tested.
- Trusted HTTPS helper:4 no-socket DNS/TLS/origin/redirect/header/failure tests pass.
- Provenance tests:2 positive/negative cases pass (run/source/workflow identity and
  exact source-map missing/changed/extra rejection).
- New Qt suite:7 cases,14 app/worker pairs pass. Actual admitted mortgage WASM,
  approval, reactive edits and distinct cold-process offline display are exercised.
- Two independent reviews APPROVED, exact36-source maps, executed hashed logs:
  `review-security.json` and `review-integration.json`. The integration review
  retains an initial unfrozen/heavily concurrent Qt timing failure; its frozen
  full7-case rerun passed86.048s. The failed attempt is not counted as passing.
- Full accepted local ABI/IPC/REG/UI/OP/VS1/VS2/VS3 wrapper: running. Initial
  sandbox run failed an existing registry special-file creation with EPERM;
  rerun outside sandbox. No rejection assertion was weakened.
- Hosted Linux CI: pending; mandatory before LLM-024 can pass.
- LIVE-001/002: incomplete. Explicitly consented real request1 returned HTTP429
  (RATE_LIMITED) in3051ms, no suggestion/preview/room changes or guest execution.
  No retry/fallback/approval occurred. PIDs, binary hashes, sanitized status and
  logs are archived in `live-attempt-1/`; actual credential scan passed.
- LIVE-003: partial failed-request observation only; no successful usage/cost or
  approved live-derived runtime journey can be claimed. Provider account/quota
  availability needs resolution. At most2 requests remain within recorded consent.

Do not mark Accepted from mocked/hosted success alone. Actual successful
structured suggestion, exact preview/human approval/runtime and no-call reactive/
cold demonstration remain required. No monetary hard cap or server-side
cancellation guarantee is claimed. Accepted topology and initialization retain
separate, non-atomic commits; provider revocation is ordered against approval by
host transaction guard, not retroactive rollback.

API/model documentation and schema translation are recorded in
`../API_VERIFICATION.md`; reproducible commands/gate map are in `../CONFORMANCE.md`
and `../TASK.md`. Open artifact/archive design remains Proposed separately as
ADR-015; it was not silently resolved by this provider implementation.
