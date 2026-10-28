# ADR-006 numbered conformance tests

All `LLM-001`–`LLM-024` are mandatory deterministic tests in local and hosted Linux CI. Use mocked HTTP server/transport with no external credentials. Never skip tests when credentials are absent.

| Gate | Required result |
|---|---|
| LLM-001 | Offline fixture provider remains usable with no network, secrets or remote consent. |
| LLM-002 | Remote provider defaults disabled; explicit consent and valid configuration required before outbound request. |
| LLM-003 | Actual outgoing request contains only explicit user input and bounded admitted-catalog projection; no stored documents or local paths. |
| LLM-004 | Endpoint allowlist, HTTPS requirement, redirect prohibition and no model-supplied URL enforced; deny private/loopback destinations in production configuration. |
| LLM-005 | Credentials read from approved host source, never inserted in prompts, snapshot, transport URL or logs. |
| LLM-006 | Mock server validates correct structured-output API request and no tool invocation exposed. |
| LLM-007 | Valid mocked suggestion passes canonical ADR-005 strict parsing, catalog-key and exact-pin revalidation. |
| LLM-008 | Duplicate keys, unknown properties, malformed UTF-8/JSON and excess bytes rejected without mutation. |
| LLM-009 | Model-invented candidate keys, IDs, URLs and port names rejected. |
| LLM-010 | Refusal and safety-filter responses reported as `REFUSED`; no preview or mutation. |
| LLM-011 | Truncated/incomplete responses and schema incompatibility reported separately, fail closed. |
| LLM-012 | Authentication/authorization failure has no retries and no secret-bearing diagnostic data. |
| LLM-013 | Rate limit, `Retry-After`, transient 5xx and bounded retry budget tested, no unbounded recursion. |
| LLM-014 | Host byte, catalog, output-token, concurrency and timeout limits enforced. |
| LLM-015 | Cancellation before and during transport invalidates late results and pending previews. |
| LLM-016 | Room revision drift and candidate-package drift invalidate prepared suggestions without auto-apply. |
| LLM-017 | Remote consent does not substitute for exact proposal-bound user approval; reject/replay/expiry gates retained. |
| LLM-018 | User-approved remote-derived suggestion executes through actual accepted canonical plan, ingress, WASM and reactive Qt path. |
| LLM-019 | After assembly, UI edits make zero new provider calls; cold restart displays committed state with zero provider calls. |
| LLM-020 | Multi-room isolation, provider-switch cancellation and concurrent request fences proven. |
| LLM-021 | Redacted telemetry records correlation, provider/model, limits/usage and statuses without raw messages or tokens by default. |
| LLM-022 | Failure injection covers network loss, DNS/TLS errors, invalid headers, unexpected tool-call attempts and oversized error bodies; fail closed. |
| LLM-023 | Two independent reviews: security/privacy and provider/runtime integration, with findings resolved and retested. |
| LLM-024 | Hosted Linux CI green; all previous gates/regressions remain green; evidence includes commit, tooling, hashes and reproducible test commands. |

## Additional live acceptance (mandatory for *Accepted live integration*, not for CI)

- `LIVE-001`: opt-in real provider, credential loaded through host facility; model ID and provider recorded; successful structured suggestion for installed mortgage package.
- `LIVE-002`: exact candidate/port/pin reconciliation, human preview/approval and genuine WASM execution; subsequent edits without extra provider calls.
- `LIVE-003`: record latency, usage/cost where exposed, refusals/timeouts if observed, provider limitations, sanitized logs and assurance that no secret was uploaded. Negative cases may be demonstrated with mock transport rather than deliberately inducing billed failures.

A green mocked CI run alone is **not** evidence of a working live provider.
