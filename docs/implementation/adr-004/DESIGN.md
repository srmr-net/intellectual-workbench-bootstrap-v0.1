# ADR-004 implementation profile

The supplied canonical schema replaces an unaccepted G0 proposal. Preserve the
original bytes as schemas/ui-bootstrap.schema.json (original $id) and execute all
original 30 fixtures/reference JSON against that exact schema; it never enables
runtime UI. Canonical schemas/ui.schema.json has distinct ADR-004 $id, strict
object bindings and only form/decimal-input/integer-input/readout. No acceptance
check is weakened or silently retargeted to a permissive runtime schema.

Existing accepted references to ADR-004 storage mean the old unresolved placeholder.
This user-provided kit assigns ADR-004 to UI. Durable database recovery remains
unresolved and is tracked separately; no accepted ADR text is renumbered/changed.

Qt choice: first-party Qt Quick/QML (local6.11.2; hosted Ubuntu Qt6.4 public APIs),
QProcess/QObject asynchronous transport to a trusted Rust worker. Qt is LGPLv3/GPLv3
or commercial; this source uses normal dynamic system Qt linkage. No CXX binding
library or shared raw pointers. All QObject operations stay on GUI thread. Kernel,
Wasm and original bus remain together in the Rust process; new UI framing does not
replace the accepted bus transport. Frames max64KiB, commands128, projections128;
committed replies block under backpressure and are never silently dropped. Qt caps
both pending commands and write/read accumulation. View epochs fence queued callbacks.

Revision/generation/projection sequence are decimal strings on the Qt wire so values
above2^53 remain exact. Qt cannot supply canonical routes/message IDs/correlations.
Rust binds control IDs against an immutable admitted UI descriptor and uses sealed
native ingress. Invalid/incomplete edits only affect bounded host drafts; valid full
objects go through existing submit_ui/Invocation/complete. A same-generation revision
conflict preserves draft for explicit Apply; wrong key/generation changes no draft.

Admission policy is opt-in semantic_ui_v01; default headless admission stays unchanged.
Optional checked UI descriptor is omitted for None, retaining old record bytes and
idempotence. Internal integrity/quota/recovery remains profile-neutral in mixed stores;
UI render access requires the opt-in policy. Raw UI bytes and frozen schema digest,
manifest path, RDF AbstractInterface and descriptor must agree. New UI package identity
uses the unchanged accepted mortgage WASM, not a mutable replacement of its release.

UI bindings are single decoded RFC6901 object-member keys, with ~0/~1 supported,
no indexes/nesting/prototype keys. Scalar types must be direct and explicit after
accepted local reference expansion; ambiguous schema unions/combinators fail closed.
Decimal editors emit canonical unsigned decimal strings only, integer editors strict
integer values; field and full-object constraints still apply. Units are literal hints.
All required aggregate input fields must have one editable binding for this profile.

Snapshots use the unchanged logical RoomSnapshot and current exact registry UI record.
A trusted host-owned file permits quit/reopen, with no database/power-cut guarantee.
Fresh startup explicitly approves only the selected exact pin; snapshot identity/UI
reference must match it. UI does not authorize execution or revive saved grants.
Failure preserves prior output VALUE while accepted validity/error/generation changes
remain visible. Nothing derives runtime authority from package UI labels or RDF.

Review corrections: projected integer scalars use exact text (including u64::MAX),
and integer edit parsing supports both signed i64 and unsigned u64. Trusted labels
wrap at large text sizes. New admission requires basename ui.json to match the
accepted room-reference grammar; nested safe directories remain possible. Snapshot
authority is anchored to the same registry root dev/inode and held private dirfd;
restore leaves are no-follow/nonblocking regular single-link bounded files. Each
accepted input is saved before its committed projection is published. Ordinary
quit/reopen is tested without explicit Save; backlog teardown cannot lose an
already displayed value. Rejected/full Shutdown does not terminate the worker.
