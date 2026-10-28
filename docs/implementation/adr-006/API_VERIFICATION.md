# OpenAI API verification — 2026-10-28

Primary documentation checked before implementation:
- https://developers.openai.com/api/docs/models/gpt-5.4-mini
- https://developers.openai.com/api/docs/guides/structured-outputs?api-mode=responses
- https://developers.openai.com/api/reference/resources/responses/methods/create

The model page lists Responses and Structured Outputs, snapshot
`gpt-5.4-mini-2026-03-17`, and reasoning effort `none`. The adapter pins this
snapshot; no implicit model fallback. Responses structured output uses
`text.format` with `type:json_schema`, `strict:true`, and a closed object schema.
Every property is required; every object uses `additionalProperties:false`.

The accepted suggestion schema contains an extensible input map. The provider
translation closes it to the already accepted domain normalizer's fields/types
and enums the offered opaque candidate keys. The accepted schema is untouched;
local strict parsing, canonical value reconciliation, catalog membership, exact
pins, room revision, host plan validation and separate approval remain authority.

The request explicitly disables storage, streaming, background operation and
tools, and requests reasoning none. The response handler rejects non-message
outputs, refusal/filter, incomplete output, unexpected model, malformed/duplicate
JSON, invalid suggestion/value/membership and excess tokens/bytes. No raw provider
error text reaches Qt. Mock fidelity and current docs do not substitute for the
separate actual provider smoke test.

## User-selected OpenRouter profile — 2026-10-28

After the actual OpenAI request returned429, the user explicitly requested
`nvidia/nemotron-3-ultra-550b-a55b:free` through OpenRouter with a different private
host credential. This is a separate adapter/profile, not generic API compatibility.
Primary sources checked:
- https://openrouter.ai/api/v1/models/nvidia/nemotron-3-ultra-550b-a55b:free/endpoints
- https://openrouter.ai/nvidia/nemotron-3-ultra-550b-a55b%3Afree
- https://openrouter.ai/docs/guides/features/structured-outputs
- https://openrouter.ai/docs/guides/routing/provider-selection
- https://openrouter.ai/docs/guides/best-practices/reasoning-tokens

The exact free endpoint is served by `nvidia`; it lists reasoning, max_tokens,
tools/tool_choice but **does not support response_format/structured_outputs**.
The paid similarly named model has different capabilities and is not substituted.
No provider-enforced JSON claim is made. ADR-006§2.3 prefers a provider facility
when available and explicitly keeps local strict validation as authority. This
profile asks for the closed schema in a fixed prompt, omits unsupported schema
parameters, and rejects non-JSON/markdown, duplicate/extra fields, invented keys,
noncanonical values and all accepted Suggestion failures without repair or retry.
No accepted parser, schema or gate is weakened for this model.

Endpoint is compiled as https://openrouter.ai/api/v1/chat/completions; TLS/DNS,
redirect, host-owned credential and cancellation controls remain identical.
Routing is fixed to `provider.only:[nvidia]`, `allow_fallbacks:false`,
`require_parameters:true`, and max_price prompt/completion/request0. No other
provider/model or paid fallback is allowed. This is a routing filter, not a claim
of independently measured or guaranteed billing. Request max_tokens512 and
reasoning disabled/excluded; reasoning can consume the shared output budget, so
length/empty-content failures remain fail closed. Refusal/content_filter, length,
tool calls, HTTP402/408/429 and usage fields have separate Chat API normalization.

OpenRouter's free model page discloses NVIDIA logging for security and product
improvement and warns against confidential/personal data. Remote UI disclosure and
separate scoped live consent cover this policy. Only the synthetic mortgage intent
and bounded admitted catalog may be used in the live validation. No ZDR or zero
provider-retention claim is made. OPENROUTER_API_KEY or a private host file is used;
an OpenAI environment credential is never automatically sent to OpenRouter.
