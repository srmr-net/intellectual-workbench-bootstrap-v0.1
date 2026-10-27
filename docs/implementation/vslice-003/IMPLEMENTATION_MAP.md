# VS3 implementation boundaries

The accepted ADR005 implementation already covers a genuine initial-empty-workspace
journey. VS3 adds a separate product entry under apps/desktop-qt/vslice-003: submitted
message and keyboard-accessible confirmation/rejection, plus sequential assembly
of fresh empty rooms alongside durable populated rooms. It does not broaden the
single-component native planner or attach a provider to reactive edits/restore.

Each new target has a host-generated unused stable id and private accepted Session.
The provider sees only the accepted minimized catalog and current explicit utterance.
Native Session handles concurrent-room approval/cancellation isolation; the single
Qt composer permits one pending intent, rejecting overlap instead of sharing consent.
A preparation captures workspace revision/content hash. Any intervening persisted
edit/pin/selection invalidates confirmation, even if the empty native room itself
has not changed. Fresh preview replaces the cancelled stale Session when necessary.

The FileStore callback only appends the new topology checkpoint to the revalidated
snapshot. Initialization uses accepted typed ingress and actual installed WASM. Its
validated output replaces only that target checkpoint. Prior room pins/state/pinning
are preserved byte-for-byte within the snapshot; workspace revision/checksum advance.
A failed topology checkpoint does not masquerade as success because other saved rooms
exist. Native topology may already have committed; durable/initialization booleans
report the separate stages. Accepted form repair remains explicit and model-free.

Accepted source/hash-bound regression collectors deliberately require exact old bytes.
The product uses a standalone locked manifest and additive CI workflow, preserving
old source/tests/locks/workflow instead of editing acceptance receipts. Both manifests
receive locked Rust checks. The outer wrapper retains the full OP→VS2→UI→VS1→REG→IPC→ABI
chain and adds named VS3 process/worker/evidence gates.

Deferred architecture: general NLP/external providers, populated-room planning,
autoapproval/repair/retry, multi-component graphs, remote stores, permanent namespace,
general numeric conventions and full database/power-loss recovery. No accepted
contract blocks the scoped integration; no trust-boundary change is required.

The accepted Desktop restarts its private view counter when rematerialized. The new
worker maintains a monotonic process-local outer epoch and translates typed view
generations at the boundary; native guest/runtime generations remain unchanged.
Only an explicit selection or quit may acknowledge a just-completed handoff using
the last observed epoch. Old/future edits fail closed. New native process tests cover
these fences, successful refresh, repeated assembly and old-room preservation. No
new field, WIT export or accepted Desktop/bridge behavior is introduced.

The new entry's trusted generic renderer differs in one model-lifetime expression:
`bridge.ready ? bridge.controls : []`. It disposes selected-view controls during
selection and recreates them afterward. Identical UI metadata in two rooms otherwise
keeps the old editor ownerEpoch and silently rejects edits. This fixes product-view
ownership without rebinding old delayed callbacks to new-room authority. Accepted
Main.qml, original renderer tests and schema remain byte-identical; new QML is
compiled host-owned resource data, never loaded from component packages.
