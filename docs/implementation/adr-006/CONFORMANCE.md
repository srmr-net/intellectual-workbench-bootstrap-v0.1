# Executable conformance map (Proposed)

`bash apps/desktop-qt/validate-live-provider.sh` runs the unchanged accepted VS3
wrapper and all its nested OP/VS2/UI/VS1/REG/IPC/ABI gates, then the new entry.
Mock tests have no external credentials and the UI worker lacks `live-http`.

| Gates | Executable evidence |
|---|---|
| LLM-001–003 | native `llm_001`–`003`; Qt offline/bad-config and missing-consent |
| LLM-004 | native closed configuration; Python public/mixed DNS, TLS/SNI/pinning, redirect/auth unread bodies |
| LLM-005 | private file permissions/symlink/hardlink test; secret echo and telemetry exclusion |
| LLM-006–014 | individually numbered native tests: closed API schema, canonical values, strict JSON, invented authority, refusal, incomplete, auth, bounded retry and budgets |
| LLM-015 | native pending/drop cancellation, actual subprocess kill/reap, revoke inside start/ready; Qt pending provider switch |
| LLM-016–017 | remote Qt preview/rejection/no mutation; retained executable exact pin drift, revision/expiry/replay tests through accepted Session |
| LLM-018–019 | real Qt remote mock → canonical plan → explicit confirmation → installed WASM → committed canvas; reactive edit and distinct cold process under network-denying launcher, zero calls |
| LLM-020 | native one-shot room/input/ticket/owner fences, both transaction orders and atomic slot publication; Qt provider switch/reconsent/edit revocation |
| LLM-021 | redacted bounded observation/usage test and process observations |
| LLM-022 | native transport failures, non-message/tool attempts, oversized transient response before retry; Python DNS/TLS/header/length/error injections |
| LLM-023 | collector validates two distinct approved source-bound independent reviews and hashed executed logs |
| LLM-024 | complete hosted Linux wrapper, artifact download and independent `evidence/verify.py` checks run identity, workflow, source, hashes and retained accepted evidence |

The native conformance executable has27 cases plus the separate actual subprocess
unit test. Python transport has5 cases; Qt journey has7 cases/14 distinct app-worker
pairs. These counts exclude retained accepted suites. Native domain fixtures are
synthetic and are never an API credential. The supplied profile schema's generic
model field is narrowed by host policy to the tested snapshot; the placeholder
`CONFIGURE_SUPPORTED_MODEL_ID` is intentionally not usable remotely.

The provider uses the accepted normalizer to bound fields/types and independently
reconcile the model response. No broader natural-language capability is claimed.
The preview/approval tests retain the non-atomic topology-then-ingress boundary.

LIVE-001–003 must be separately recorded after scoped consent. Mock coverage of
provider errors is not evidence those errors were observed against the real API.


The three additional native OpenRouter cases cover the separate request/routing
profile without unsupported provider schema parameters, strict Chat normalization
and failure taxonomy, and the same one-shot consent/canonical provider boundary.
The fifth transport test proves the second fixed origin and rejects arbitrary
provider/URL selection before DNS. The genuine Qt seven-case suite now runs the
OpenRouter-shaped mock profile; original OpenAI conformance cases are retained.
Only supplemental Proposed agent-plane provider fixtures add `openrouter-chat`;
accepted suggestion/plan/manifest/UI/ontology and runtime contracts are unchanged.
