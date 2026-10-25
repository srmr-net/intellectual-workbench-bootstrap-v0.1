# wb-contracts

Version 0.1 DTOs; ADR-002 remains Proposed pending hosted acceptance.

`ipc::EventEnvelope` is the canonical architecture envelope, using camelCase JSON
and strict unknown-field rejection. Use `EventEnvelope::parse` for validated
external JSON (supported version, identities and positive sequence); Deserialize
alone checks shape. `Endpoint`, `Connection`, `CompositionPlan` and `RoomSnapshot`
match the existing canonical schema shapes. Kernel validation additionally checks
the unchanged schemas and runtime constraints. `DispatchContext` deliberately has
no serialization implementation: revision, execution generation and invocation ID
are trusted host metadata, never added to guest JSON.

The legacy `Envelope` and `Edge` prototypes and their tests remain compatible;
they are not used by the new IPC path. `ipc::HostError` carries stable code,
subsystem, correlation, message and detail. ErrorCode includes the architecture
registry, ADR-002's QUEUE_FULL/STALE_RESULT/GUEST_ERROR/RESOURCE_LIMIT/CANCELLED and
explicit INVALID_TRANSITION/DUPLICATE_ID/INVALID_ENVELOPE validation errors.
Guest code/detail is returned as untrusted structured diagnostics; AuditEvent
records codes/IDs/timing only, never payloads or guest diagnostic messages.

`InstanceStatus` covers logical instances, not artifact installation states.
`AuditEvent.sequence` is the room's host allocation clock, shared with data events.
Serialization fixtures are in fixtures/ipc; conformance lives in wb-testkit.
No DTO conveys runtime authority, native grants or a guest host-service API.
