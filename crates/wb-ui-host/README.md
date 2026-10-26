# wb-ui-host

Accepted ADR-004 trusted native presentation adapter, scoped Linux profile.
See [verified evidence](../../docs/implementation/adr-004/evidence/RESULTS.md) and
[protocol/errors](../../docs/implementation/adr-004/PROTOCOL.md).
`Session` consumes only a verified registry UI handle plus an explicitly approved
exact component pin. It owns a room, sealed native UI ingress, bounded draft map,
validated binding table and committed projections. It calls accepted kernel APIs;
no guest/ABI/lifecycle protocol is changed. A rejected same-generation revision
conflict retains drafts for explicit Apply; wrong ownership/generation cannot alter
them. Invalid or incomplete drafts never invoke the guest or advance room state.

`wb-ui-worker SOURCE STORE PYTHON SEMANTIC_WORKER ABI_WORKER ROOM new|restore`
is a trusted desktop child process, not a sandbox service for guests. Native startup
explicitly approves only the chosen pure room/instance/pin. It admits/resolves real
immutable packages and runs the original kernel/bus in the worker. UI existence
never grants permissions. `restore` checks the unchanged RoomSnapshot against that
same exact release/UI and fresh approval. Missing pins fail visibly.

Wire: four-byte big-endian nonzero length, maximum65536 UTF-8 JSON bytes, strict
duplicate-key parser. Incoming commands and outgoing replies each capacity128.
Input overload returns QUEUE_FULL. Outgoing committed replies block rather than
being dropped. Protocol0.1 supports Edit, Apply, Refresh, Save, Shutdown. Only
accepted Shutdown ends the process and the final reply is flushed first. Room,
instance and UI identity, expected revision and generation are mandatory; no
client-selected bus routes, message IDs or canonical correlations. Projection
counters are decimal strings. Display scalar values are strings too, preserving
u64 precision through Qt; authoritative JSON input/output types remain unchanged.

Private adjacent ui-snapshots are anchored by held directory descriptors to the
same registry root inode. Regular single-link restore files, private owner/mode,
no-follow/nonblocking opens, bounded reads, fsync+rename apply. Accepted edits save
before publishing their projection, so bounded Qt teardown cannot lose an already
shown commit even if later queued work is killed. Save/EOF also snapshot. This is
quit/reopen support, not a SQLite/RDF recovery or physical power-loss guarantee.

Run `bash apps/desktop-qt/validate.sh` with pinned guest/Rust tools and real Qt.
Tests include native admission/session/framing, actual framed worker rejection,
real installed mortgage WASM, GUI/accessibility and full accepted regressions.
