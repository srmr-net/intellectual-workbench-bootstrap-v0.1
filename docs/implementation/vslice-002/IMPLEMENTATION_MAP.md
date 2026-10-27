# VSLICE-002 implementation map / scoped decisions

Accepted registry discover/resolve_exact/read_verified_bundle/read_verified_ui APIs
supply installed identities and validated bindings. The actual mortgage manifest
has no displayName/name: picker falls back to its admitted id (not a hard-coded
mortgage label). Manifest/UI/WASM bytes and their exact pins remain unchanged.

Accepted Qt Main.qml is a generic ApplicationWindow; a new QWidget three-column
shell embeds that actual QQuickWindow using createWindowContainer. A separate
bounded DesktopBridge implements its existing presentation interface. The accepted
QML, UiBridge, worker, UI/ABI/IPC/REG/VSLICE tests and collectors stay byte-preserved.
New shell lists are host widgets containing only literal admitted metadata.

wb-storage is an interface placeholder. Its new vs2 module provides a documented
internal version1 snapshot and anchored private Linux atomic file adapter, not a
database engine. One workspace writer lock prevents concurrent processes from
clobbering acknowledged state; snapshot revision CAS also fences stale writers.
Corruption or unavailable exact pins latches read-only recovery without overwrite.

Accepted RoomSnapshot saves inputs, not derived output; Session::restore recomputes.
The new desktop host stores a coherent successful input/output checkpoint plus
exact component/UI/schema pins. It validates every room/schema/pin before initial
view, then projects that durable committed checkpoint without guest execution.
No saved output is injected into authoritative kernel state. On the first complete
VALID edit only, it lazily activates accepted Kernel::restore_room, suppresses its
intermediate recomputation, and submits the requested edit through accepted typed
ingress. Desktop epoch/revision/generation are checked before activation and map
explicitly to the fresh native room/adapter authority. Invalid/incomplete/forged drafts
cannot trigger activation. Subsequent native commits alone replace the checkpoint;
failed input batches retain prior durable successful pair and show diagnostics.

Workspace selection/pinning and successful commits are saved before acknowledgement.
Normal quit stops edits, drains accepted commands and flushes before final close;
GUI ownership never blocks on admission/compute/storage. This establishes clean-quit
persistence, not physical power-loss journaling, database recovery or saved grants.
Full persistence/database and cross-store transactions remain ADR012 future work.

The new host uses public accepted Room APIs directly (not private Session fields),
so actual ingress envelopes/audit are inspectable. The accepted Session implementation
remains byte-preserved. Separate desktop transport is1MiB/32 frames; metadata quotas,
live-audit recycling and128 trace entries bound native presentation memory. Snapshot
content checksum detects accidental corruption, not malicious trusted-host forgery.
