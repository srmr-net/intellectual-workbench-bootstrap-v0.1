# wb-desktop-host — VSLICE002

Status PROPOSED until mandatory product gates/reviews/hosted evidence pass.
This trusted native desktop orchestrator discovers only installed UI-bearing
packages using accepted registry APIs. It does not install while opening a view,
select floating versions, grant declared permissions, or bypass the accepted room
controller. `wb-desktop-install` is the separate explicit local admission command.

`Desktop::open(registry, FileStore)` loads the internal v1 workspace and verifies
EVERY saved room before publishing any view: exact component/version/raw-WASM pin,
UI path/raw/schema digests, unchanged logical RoomSnapshot and full aggregate input/
output schemas. Cold display projects the validated durable committed checkpoint
without creating a Kernel, executing a guest or invoking a model. Stored outputs
are never injected into authoritative kernel state or treated as new guest output.

A subsequent COMPLETE VALID edit creates a fresh explicitly approved kernel room
using accepted create/plan/prepare/start/authorize_ui/submit_ui APIs, or lazily uses
Kernel::restore_room for a saved checkpoint. Any restore recomputation occurs only
then and is not published. The requested aggregate goes through real ADR002 ingress,
WASM invocation, output-batch validation and commit. Only a successful coherent pair
replaces the durable checkpoint; failure preserves the prior pair and surfaces a
correlated diagnostic. The trace contains actual ingress/audit plus native-generated
correlation separate from the GUI local token. Invalid/incomplete/forged edits never
activate a room, execute WASM or advance committed display revision.

Desktop view epochs fence selection and stale callbacks independently of recreated
kernel execution generations. Both domains are checked explicitly before dispatch;
no client stale counter is silently promoted to authority. Same-generation revision
conflicts preserve drafts for explicit Apply; no automatic compute retry. GUI Save/
Pin and accepted edits cannot erase unrelated unsent drafts. Navigation discards
uncommitted local drafts; saved checkpoints always contain committed state only.

Native transport `desktop.1` is separate from accepted UI/WIT/bus protocols: four-byte
big-endian length, max1MiB,32 incoming and32 outgoing frames. Outgoing committed
responses block rather than drop. Desktop metadata bounds128 installed UI packages,
16 rooms,32 controls, id200/version64/label160; quotas fail visibly. Live kernel audits
are recycled after256 entries (next explicit valid edit creates a fresh incarnation);
trace retention128; accepted kernel audit semantics remain unchanged. Valid maximum
metadata is tested beyond the former64KiB UI frame limit. Errors/tokens are bounded.

Recovery is latched/read-only for corrupt/incompatible snapshots, unavailable saved
pins or storage failure. No later pin/edit/quit overwrites original diagnostic bytes.
Selection re-verifies immutable bytes; no automatic substitute, repair or download.
Every successful transition saves before ACK. Quit stops edits, drains accepted work,
flushes the final reply and exits; EOF never overwrites recovery files.

Executables:

- wb-desktop-worker REGISTRY WORKSPACE PYTHON SEMANTIC_WORKER ABI_WORKER
- wb-desktop-install REGISTRY PYTHON SEMANTIC_WORKER ABI_WORKER PACKAGE...
- wb-desktop-offline APP ARGS... — native conformance launcher that denies ordinary
  IPv4/IPv6 socket syscalls, probes EPERM, then execs the real app. It is not a general
  adversarial network sandbox or guest service; Unix Qt/DBus IPC remains available.

Reproduce through `bash apps/desktop-qt/validate-vslice-002.sh`. No Operator, remote
registry, guest WASI/Qt service, numerical implementation or database engine is added.
