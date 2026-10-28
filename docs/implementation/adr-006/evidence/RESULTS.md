# ADR-006 validation ledger — Proposed pending hosted CI

Frozen implementation: `23fd97d90017eeb50a84781faedac82bf43bf9b5`.
Initial OpenAI implementation87ff369; evidence-source hardening a9eb0db; explicit
user-selected OpenRouter profile cd7b29b. Accepted runtime/WIT/manifest/plan/UI/
ontology and existing source bytes remain preserved.

- Locked native:27 conformance tests +2 unit tests pass in default and mock builds.
- HTTPS helper:5 deterministic no-socket DNS/TLS/origin/redirect/header cases pass.
- Provenance:2 positive/negative tests pass, including exact Git-source inventory.
- Genuine Qt:7 cases/14 app-worker pairs pass, using OpenRouter-shaped mocks.
- Two refreshed independent reviews APPROVED for23fd97d with exact37-file source
  maps and executed hashed logs. Final23fd native, transport, metadata and Qt reruns pass.
  Initial unfrozen/concurrent Qt timing failure remains archived; it is not counted
  as passing. Reviewer’s frozen OpenRouter full Qt rerun passed66.927s.
- Original full accepted local wrapper completed successfully outside sandbox;
  every retained ABI/IPC/REG/UI/OP/VS1/VS2/VS3 regression passed. An earlier sandbox
  attempt failed existing special-file fixture creation with EPERM; no assertion
  was weakened. The clean fe43089 full wrapper and downloaded local ZIP verifier also passed.
  Final audit-only source refresh now requires a fresh clean full-wrapper run.
- Hosted Linux CI: fe43089 completed successfully in run40727027481; final23fd97d
  audit/factory correction still requires refreshed hosted execution and downloaded
  verification. LLM-024 is pending for the final source.

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


Final audit correction: each adapter caches its immutable remote identity for
both accepted Session and supplemental telemetry, including cancelled work. Host
factories use immutable profile selection so concurrent invalidation is nonfatal;
profile/identity mismatch fails before credential load or transport. The added
switch/staged-construction regressions pass. Both source reviews were refreshed.
The successful actual LIVE evidence remains explicitly bound to cd7b29b and its
original binary hashes; final23fd binary is not claimed separately live-tested.
Both reviewers assess this narrow cancellation/construction/audit-only delta as
not changing the successfully validated API/normalization/approval/runtime path;
fresh deterministic and hosted validation remain mandatory for final source.
