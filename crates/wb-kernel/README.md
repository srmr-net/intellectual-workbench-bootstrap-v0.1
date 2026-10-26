# wb-kernel

Version 0.1, ADR-002 validation integration. The existing rejection facade remains.
The new `Kernel` and `room::Room` APIs implement a pure computation integration
profile accepted by ADR-002 after verified hosted IPC-001..015 results.

`Kernel::cache_verified_bundle` consumes bytes/manifest/schema documents resolved
by a trusted package verifier. It calls the existing ADR-001 Host::register, checking
manifest, raw digest, schema and WIT without executing guest code. This is an
immutable admission cache, not discovery/installation/RDF verification. Conflicting
bytes under the same id/version are rejected and resolution requires the exact pin.
No registry filesystem/network lookup or implicit replacement is performed.

Execution approvals default to deny. The trusted host must explicitly approve an
exact (room, logical instance, component id/version/digest) with approve_execution.
This approves import-free execution only; native capability grants remain denied
by wb-capabilities and every Wasm call still uses the unchanged ADR-001 host.
Approvals are checked on plan, prepare/start, input acceptance, invocation start
and result commit; they are revocable and never serialized/restored as credentials.
Manifest permissions cannot grant authority; nonempty requests are denied.

create_room gives the sole controller for a live room ID. Serialize all its methods
with &mut Room (or an application-owned mutex); no room lock is held by guest code.
apply_plan is revision-CAS staging: resolve pins/approvals, reserve quota leases,
validate ports/semantics/units and graph, then commit the entire topology. Failures
release staging reservations. Logical lifecycle is declared → prepared → running →
stopped; explicit prepare re-admits stopped/failed instances into a new generation.
Trap/resource failure marks an instance failed; no implicit retry or LLM recovery.
Remove/replacement cancels first, removes incident edges and releases leases.
Plans are complete topology proposals, not implicit partial mutation commands.

UI adapters are sealed, room-scoped native handles with explicit destination
allowlists. submit_ui validates and stages an event in the bounded queue; its
returned canonical envelope acknowledges queue acceptance, not a state commit.
The scheduler starts FIFO UI waves: it commits that input exactly once as a new
room revision, freezes the aggregate payload, and drains affected descendants in
stable topological order before committing the next staged UI event. A new staged
input cannot invalidate an unrelated active computation. Revoked adapters/approvals
are rechecked before state commit. dispatch.accepted marks queue acceptance;
room.revised marks authoritative input/topology commit. Lifecycle control commands
use the room sequence and instance generations, not extra input-state revisions.

next_invocation returns a sealed, non-cloneable ticket. Run it outside the room
lock, optionally in a worker thread, then pass its consumed Completion to complete.
Every ticket exclusively calls existing Host::call (fresh Store, empty imports,
existing fuel/memory/epoch limits). Room state is not accessible to the ticket.
One logical invocation is active per room; a finite global invocation lease pool
also bounds quarantined workers/completions. At-most-once means one attempt, not
exactly-once effects. External control/topology changes can invalidate detached
work: exact revision/generation plus invocation identity are checked before commit.
Late results return STALE_RESULT and never clear a newer invocation's authority.

stop is direct control, independent of full data queues. It revokes active commit
authority and queued work throughout the stopped instance's descendant set, audits
CANCELLED with the original invocation correlation, and changes the generation.
An already executing pure call can finish within existing Host fuel/epoch ceilings;
its isolated worker has no authority to commit. Stop does not add guest cancellation
imports or claim hard real-time interruption. Starting a new generation cannot
revive an old completion. Dropping a room also cancels detached tickets.

complete validates the entire candidate output batch and every registered edge,
destination state/approval/schema and queue capacity before publishing projections
or deliveries. Duplicate output port entries are rejected as ambiguous batches.
Failed batches preserve previous values but mark affected projections invalid with
a structured error; descendants are suppressed. Even a first failure creates typed
invalid output projections with no value. An existing projection revision identifies
the provenance of its preserved last value, not a new successful commit. Guests cannot choose envelope
source, room, destination, IDs or sequences. HostError retains guest code/detail
for explicit diagnostic access; default audit records no payload or guest detail.

snapshot produces the existing logical room JSON shape, with authoritative input
port aggregates only. Identically named input/output ports remain legal: derived
outputs never overwrite input in the snapshot. restore_room checks schema, exact
pins, graph, quotas, current execution approval and aggregate schemas; it constructs
fresh generations and queues root inputs for recomputation. No Store, memory,
in-flight queue, UI adapter or grant is restored. Stored derived values, if present,
are validated but ignored. This profile addresses whole port payloads as
instance.port; field-level import/migration and durable transactions remain ADR-004.
No database, Qt, privileged guest services or native host imports are introduced.

Run bash examples/wasm-components/validate-ipc.sh with the documented pinned tools.
It retains the full ABI suite and adds named IPC tests and machine-readable evidence.
See docs/implementation/ADR-002-VALIDATION.md and docs/adr/evidence/ADR-002-results.md.
Initial quotas/queue/host ceilings are spike parameters, not production admission
policy; original compilation/schema/native-lifting limitations remain.

ADR-003 adds `Kernel::cache_registry_release(&Registry, &ValidatedHandle)`: it
revalidates registry-pinned bytes and metadata before caching through the existing
ADR-001 host. No lifecycle, IPC, revision or approval semantics change. A registry
handle never grants execution; explicit ADR-002 policy approval remains required.

`headless::HeadlessRoom` wraps the existing native room controller for a CLI/test
host. `start` requires an already cached, explicitly approved exact pin; it applies
one instance plan and performs prepare/start before issuing an allowlisted native
ingress adapter. `attach` issues a fresh adapter for a restored controller and does
not grant/start an instance. `submit` delegates to accepted submit_ui semantics;
room()/room_mut() expose the same sole host controller for scheduling/observation.
The required UI reference is inert host metadata, never a package UI or renderer.
No lifecycle, queue, revision, grant or error semantics are altered.

`wb-mortgage-headless` is a real native entry point over registry admission and
these APIs. It grants only an explicit pure-execution approval for the selected
pin, uses a bounded20M fuel workload policy (original Host defaults unchanged) and
prints only committed projection plus actual trace metadata. See
examples/mortgage-headless/README.md and the VSLICE-001 task for reproduction,
financial bounds, negative tests and evidence. The original metadata-only mortgage
package is not installed by this path.
