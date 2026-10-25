# Canonical IPC JSON fixtures

`valid/envelope.json` uses the architecture's exact envelope fields. Negative
fixtures reject unsupported protocol version, unknown trusted metadata, bad
endpoint shape, negative sequence, missing room and duplicate structural JSON keys.
IPC-014 exercises serialization/rejection; no existing contract schema is changed.
Dispatch revision/generation/invocation data is deliberately not portable payload.
Runtime lifecycle, authority, routing and state semantics are tested with actual
components in wb-testkit/tests/ipc.rs, not inferred from JSON validity.
