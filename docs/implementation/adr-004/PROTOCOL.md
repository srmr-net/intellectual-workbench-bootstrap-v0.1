# Native presentation protocol 0.1

This trusted host interface is separate from the accepted component WIT and bus.
Frames are four-byte big-endian lengths followed by strict UTF-8 JSON,1–65536bytes.
Commands/replies each have capacity128. Qt independently caps pending operations,
read accumulation and outstanding write bytes. All DTO objects reject unknown keys;
requests reject duplicate keys before deserialization. No canonical bus source,
route, message ID or correlation is supplied by QML.

Requests contain protocol, localToken, key{roomId,instanceId,uiId},
expectedRoomRevision, expectedGeneration and command. Counters are exact canonical
u64 decimal strings. Commands: edit{controlId,proposedValue},
apply{proposedValues}, refresh, save, shutdown. Editors send canonical decimal text
or integer text; Rust restores integer JSON types, checks field contracts, then
validates the entire aggregate through accepted kernel ingress. Incomplete/invalid
input never computes or advances authoritative revision. Wrong key/generation
changes no draft; a same-generation revision conflict preserves it for explicit
Apply. Shutdown terminates only when accepted, after its final reply is flushed.

Replies contain protocol, kind, localToken, document (Ready only), projection and
accepted. Projection includes exact key, roomRevision, executionGeneration,
projectionSequence, values and diagnostics. Display values are text even for JSON
integers, preserving u64 without Qt numeric rounding. No tentative guest output is
shown. Original output validity/error diagnostics remain visible with last value.
Qt requires current view epoch/room/instance and monotonic generation/revision/
sequence. Accepted acknowledgements clear only submitted fields whose local draft
versions have not changed. Save/Refresh cannot clear unsent drafts.

## Stable diagnostics

| Code | Meaning |
|---|---|
| INVALID_UI | Invalid/duplicate-key JSON, schema shape, field or control |
| UNSUPPORTED_UI_VERSION | Absent/unknown uiVersion |
| UI_DUPLICATE_ID | Reused stable control ID |
| UNSAFE_UI_CONTENT | Executable/resource/unsafe literal metadata |
| UI_BINDING_MISMATCH | Wrong port/direction/field/type, invalid UI path/profile |
| UI_QUOTA_EXCEEDED | UI, scalar, request or counter bound exceeded |
| UI_POLICY_DISABLED | Registry render API requires explicit UI profile |
| UI_OWNERSHIP_DENIED | Wrong protocol/token or room/instance/UI ownership |
| INCOMPLETE_INPUT | Required aggregate input missing from draft |
| SCHEMA_VIOLATION | Scalar or aggregate input violates authoritative schema |
| STATE_CONFLICT | Expected room revision is stale; explicit Apply required |
| STALE_GENERATION / STALE_RESULT | Cancelled/outdated execution or invalid output |
| QUEUE_FULL | Bounded event queue rejected request; explicit retry only |
| INVALID_UI_FRAME / INVALID_UI_REQUEST / INVALID_UI_COUNTER | Invalid presentation transport data |
| UI_UNAVAILABLE | Admission, exact-pin restore, process or UI unavailable |
| UI_STARTUP_TIMEOUT / UI_OPERATION_TIMEOUT | Finite native process deadline exceeded |
| UI_DIAGNOSTIC_LIMIT / UI_TRANSPORT_ERROR | Native transport resource/write error |
| SNAPSHOT_IO | Private host snapshot could not be saved |

Other accepted kernel/registry error codes retain their existing meanings. Error
text is literal, non-executable host diagnostics; package labels confer no rights.
The native GUI-thread invariant is enforced as a programming error rather than a
recoverable guest event. This profile adds no guest services or permission grants.
