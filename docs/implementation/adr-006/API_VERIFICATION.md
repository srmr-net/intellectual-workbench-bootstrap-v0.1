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
