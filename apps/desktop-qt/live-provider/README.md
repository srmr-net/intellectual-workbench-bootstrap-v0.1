# Host-controlled Operator provider (ADR-006, Proposed)

This additive Linux entry uses the accepted VS3 shell, generic semantic renderer,
registry, Operator Session and native ingress/WASM path. Build the standalone
`host/Cargo.toml`, then `qmake6 operator.pro` in a build directory. The default is
always the deterministic offline provider. `live-http` enables the actual HTTPS
transport; `test-transport` enables only synthetic, socket-free UI fixtures. Hosted
CI builds the latter, with no credentials. Production must build `live-http` only.

The host reads optional `WB_LLM_PROVIDER` (`openrouter-chat` by default after the
user-selected provider switch, or explicit `openai-responses`), `WB_LLM_MODEL`
(only the selected vetted model), strict JSON
`WB_LLM_BUDGET`, and `WB_LLM_SECRET_FILE` (absolute owner-only regular file, no
links), or provider-specific `OPENROUTER_API_KEY` / `OPENAI_API_KEY`. Invalid optional remote configuration never disables
offline assembly or cold restoration. Credentials load only after consent and
request validation; neither Qt nor guest receives credentials. The isolated TLS
helper gets a short-lived secret through private stdin, never argv or environment.

Select the configured remote provider, enter the explicit request, then consent to the next remote request.
Consent is consumed when that exact input is queued, bound to its host cancellation
ticket and room, and cannot be reused. Editing revokes armed consent. Provider
switch/revocation invalidates in-flight results and previews. Separate composition
approval remains mandatory, bound to exact proposal hash, revision and expiry.

The native `Control` owns configuration/generation, one-shot permits, bounded
redacted observations and admission counts. `LiveProvider` implements unchanged
`OperatorProvider`. `Transport` is replaceable and has no kernel/registry handles.
`response` constructs the closed Responses schema and normalizes envelopes through
the accepted strict Suggestion parser. Profile fields cannot select endpoints.
Only https://api.openai.com/v1/responses and
https://openrouter.ai/api/v1/chat/completions are compiled in; verified TLS/SNI and public
DNS-address pinning are required, with no proxies, redirects or tools.

Effective limits intersect ADR-005: 4096 input bytes, 12 candidates (default8),
16KiB catalog JSON, 64KiB whole HTTP request, 16KiB suggestion, at most30s overall.
Default policy is20s,512 output tokens,16KiB HTTP response and1 call. Explicit
host policy may lower/raise within kit limits; canonical limits still apply.
Default retries are disabled. The Rust API permits one transient retry at most
within the same total deadline, Retry-After<=2s and call budget; no schema repair.

Cancelling kills/reaps the local HTTPS subprocess and fences late results. It
cannot unsend an admitted request, guarantee server-side inference cancellation,
or guarantee cessation of billing. For OpenAI, `store:false` disables API response storage;
provider data policies still apply. No hard monetary cap is claimed. The host
keeps only bounded status/model/correlation/count/usage observations by default,
never raw prompts, response/error text or credentials. Python/Rust allocator and
OS copies are not claimed to be cryptographically erasable.

Topology apply and typed initialization remain separate accepted operations.
Initialization failure is reported after durable topology checkpoint; no atomic
rollback claim is added. Reactive edits and cold committed-state display never
call this provider. Accepted ABI/schema/ontology files are unchanged.

Run `bash apps/desktop-qt/validate-live-provider.sh` for the retained full accepted
chain and new deterministic gates. Actual provider LIVE-001–003 is separate,
requires scoped remote-data/budget consent and private host credentials, and must
never run in hosted CI. A green mock run cannot accept ADR-006 by itself.

Approval and revocation are linearly ordered by a host transaction guard: a switch
that wins the guard rejects the old proposal; an approval that wins it completes
before the switch acknowledgment. Revocation does not retroactively undo an
already accepted apply. The UI's re-consent operation is stricter than the native
four-room lease limit: reconfiguration revokes all previous-generation requests.
The native lease test issues independent synthetic permits at one generation to
exercise per-room/global bounds and cancellation isolation directly.

The live validation driver can publish a preview and wait for a fresh separate
human decision file matching its digest before operating the existing Qt approval
checkbox/button. This is explicit test instrumentation, not normal-mode authority
and not a substitute for recording the human decision. The accepted host still
checks the real digest/revision/expiry at apply. The live script requires separate
remote consent, refuses CI, does not auto-retry, and validates offline cold display.


The user-selected OpenRouter model is exactly
`nvidia/nemotron-3-ultra-550b-a55b:free`, served only by NVIDIA with routing
fallbacks disabled and zero-price filters. This endpoint does not advertise
provider JSON Schema enforcement: a fixed prompt requests the same closed JSON
shape and the accepted canonical strict parser remains mandatory. Markdown,
extra/duplicate fields and noncanonical values fail closed; there is no JSON
repair or automatic fallback. NVIDIA's free endpoint logs requests for security
and product improvement. This policy and the lack of provider schema mode are
shown before remote consent. No provider confidentiality/ZDR guarantee is made.
The original OpenAI adapter remains explicitly selectable by host configuration,
but remote consent and provider-specific credentials never cross profiles.
