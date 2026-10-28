# ADR-006 validation ledger — Proposed pending hosted CI

Frozen implementation: `cd7b29b31cc5f2ec5f9b4fb2ba9699125dc864d0`.
Initial OpenAI implementation87ff369; evidence-source hardening a9eb0db; explicit
user-selected OpenRouter profile cd7b29b. Accepted runtime/WIT/manifest/plan/UI/
ontology and existing source bytes remain preserved.

- Locked native:27 conformance tests +2 unit tests pass in default and mock builds.
- HTTPS helper:5 deterministic no-socket DNS/TLS/origin/redirect/header cases pass.
- Provenance:2 positive/negative tests pass, including exact Git-source inventory.
- Genuine Qt:7 cases/14 app-worker pairs pass, using OpenRouter-shaped mocks.
- Two refreshed independent reviews APPROVED for cd7b29b with exact37-file source
  maps and executed hashed logs. Native, transport, metadata and Qt reruns pass.
  Initial unfrozen/concurrent Qt timing failure remains archived; it is not counted
  as passing. Reviewer’s frozen OpenRouter full Qt rerun passed58.867s.
- Original full accepted local wrapper completed successfully outside sandbox;
  every retained ABI/IPC/REG/UI/OP/VS1/VS2/VS3 regression passed. An earlier sandbox
  attempt failed existing special-file fixture creation with EPERM; no assertion
  was weakened. Final clean OpenRouter full-wrapper run is pending.
- Hosted Linux CI: pending for final OpenRouter revision. Mandatory LLM-024 remains
  pending until completed success and independent downloaded artifact verification.

## Separate actual provider validation

LIVE-001–003 PASS for the user-selected **OpenRouter/NVIDIA free profile on Linux**:

- Exactly `nvidia/nemotron-3-ultra-550b-a55b:free`, fixed NVIDIA-only routing,
  fallbacks disabled, zero-price routing filters. No provider JSON Schema facility
  is claimed; accepted strict canonical validation remains authority.
- One explicitly consented request,5469ms,565 input/159 output tokens. Only
  synthetic explicit mortgage request and bounded admitted projection sent.
- Human explicitly approved digest
  `38b0c7bbae3810cb3cf1c52ac6e96397e38e2e5e525d99e6bea14d2737c9468b`
  in chat; a fresh matching decision file then exercised the existing Qt
  confirmation checkbox/button. Native host independently checked revision/expiry.
- Real admitted WASM initialized payment1461.48; principal edit300000 produced
  committed1753.77 with remote/provider call count still1.
- Distinct cold desktop/worker PIDs restored pinned1753.77 with network denied,
  zero provider calls and zero guest invocations merely for display.
- PIDs868645/868648, cold869276/869279; exact hashes, host previews, approval,
  sanitized observations, process reports and screenshot: `live-openrouter/`.
  Actual credential scan passed; raw HTTP bodies and credentials not archived.
- No verified price table configured or billed cost retained in telemetry. Exact
  free model/zero-price routing filters are recorded; no exact-spend claim.
- No refusal/timeout observed in successful live attempt; deterministic negatives
  cover those cases. Two unused consented calls remain; no further calls needed.

The original OpenAI adapter remains implemented and deterministically tested,
but its only actual attempt returned429 in3051ms, no preview/mutation. It is **not**
claimed live validated. That failed attempt and earlier scoped consent remain
archived separately in `live-attempt-1/` and `live-consent.json`.

No monetary hard cap, remote confidentiality/ZDR, or server inference/billing
cancellation guarantee is claimed. Topology and initialization remain separate,
non-atomic commits; provider revocation is ordered against approval by host guard,
not retroactive rollback. See API_VERIFICATION, CONFORMANCE and TASK for primary
capability sources, limits and reproducible commands. ADR-015 artifact/archive
architecture remains independently Proposed; this task does not resolve it.
