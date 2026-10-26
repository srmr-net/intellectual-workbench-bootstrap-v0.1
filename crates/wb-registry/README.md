# wb-registry

ADR-003 Linux local-directory registry; admission is not an execution permission or
security certification. The canonical manifest, ontology, accepted WIT and IPC
contracts are unchanged. All package assertions remain `unreviewed`.

`Registry::open(root, Policy::local(...))` creates/reopens a private host-owned
store. Configure absolute allowed source roots and trusted absolute Python,
`tools/registry_validate.py` and compiled `wb-registry-inspect` executable paths.
Install tools/requirements.txt in that Python environment. Missing prerequisites
fail closed. Source and store ancestors must be real directories, without links.

`admit_directory` copies through directory descriptors with `openat(O_NOFOLLOW)`,
nonblocking file descriptors, fstat regular-file/link-count checks and bounded
reads. ASCII names avoid Unicode-normalization ambiguities; case-fold collisions
are rejected. Safe staging is private, mode 0700; all validators reopen staged
bytes, never source paths. Undeclared files are rejected; multiple ports can share
a schema file, but different artifact roles cannot alias the same path.

Strict JSON, the original Draft 2020-12 manifest schema, local acyclic schema
references, Turtle, trusted pinned SHACL shapes, exact manifest/RDF reconciliation,
raw SHA-256 and the accepted Wasmtime registration/typecheck all precede publication.
No guest functions or initializers run. All UI packages are blocked pending a
portable package binding ownership contract; nonempty permission requests and RDF
permission resources are denied. Schema IDs/anchors/dynamic/recursive references, explicit vocabulary declarations and
JSON pointers through array indices are explicitly unsupported. JSON instance data in const/enum/default/examples is
preserved. Dependencies, remote references and unreferenced files are rejected.

Publication writes a content-addressed `objects/<record-sha256>/` containing payload
and record, followed by an atomic `releases/<sha256(id NUL version)>` receipt.
Payload files/directories and record are fsynced before object promotion; both
rename parents sync before the index marker is written/fsynced/renamed and its
parents synced. Only a marker makes a release visible. Startup under an exclusive
OS flock removes abandoned stages and objects lacking any release reference.
Corrupt releases remain unavailable; recovery never silently replaces them. If any
published marker/record/payload cannot be validated, orphan cleanup pauses so
possibly pinned objects are preserved for explicit host repair.

Every operation uses a separate flock file description, including independent
threads/processes; the lock covers staging and all integrity reads. Identical
reinstall returns the same receipt, even at release-count capacity. A different
record for the same identity/version conflicts; different identities sharing WASM
bytes retain separate metadata. There is no remove or garbage collection API.

`resolve_exact(ReleasePin)` returns an opaque store-scoped `ValidatedHandle`.
`read_verified_bundle` / `read_verified_artifact` rehash all recorded files and the
record before crossing into the kernel. Missing/mismatched pins never fall back.
`discover(Some(capability_iri))` reads only committed, integrity-valid native
metadata. Handles grant no run authority. `Kernel::cache_registry_release` consumes
these handles; ADR-002 room/instance execution approval remains mandatory.

Default policy: 64 files/directories, 2 MiB/file, 8 MiB/package, 8 nested directory
levels, 128 releases, 128 MiB of logical stored file bytes. Each commit preserves
an additional package + 4 MiB metadata + two marker allowances within the store
budget so identical reinstall still has staging space at capacity. This is a
conservative logical-byte quota, not physical block-allocation accounting. Worker
output uses a separate unlinked temporary file, capped at 4 MiB per sequential
worker. Worker CPU and wall budgets are 30 seconds, address space 2 GiB, open files
64; signals kill timeout process groups and kill workers on parent death. These
native worker bounds are separate from unchanged guest runtime budgets.

Threat model: host-private store, host-owned trusted executables, local Linux
filesystem implementing fsync/atomic rename. Same-UID store attackers, hostile
mounts, physical power-cut verification, disk hardware faults and production
compiler sandboxing are not established. Source I/O checks time cooperatively;
a hung kernel read/fsync cannot be given a hard wall deadline here. No signatures,
remote services, archive formats, automatic dependency solving or trust elevation.

Run `bash examples/wasm-components/validate-registry.sh` with pinned build tools
and Python requirements. Tests use production paths, real symlink swaps, sockets,
FIFOs, hard links, independent installers and SIGKILL at seven commit checkpoints.
`admit_observed` exposes observational fault-injection checkpoints, not validation
bypasses. SIGKILL validates process-crash recovery, not actual power loss.

ADR-004 narrowly adds opt-in `Policy.semantic_ui_v01` (default false). New UI
admission validates canonical UI/manifest/RDF/bindings and requires a ui.json
basename compatible with existing RoomSnapshot references. Immutable raw bytes,
file/descriptor/path/schema hashes and typed document must agree on admission and
all later reads. `read_verified_ui` requires opt-in; internal integrity/recovery/
quota reads remain profile-neutral so headless and UI packages can share a store.
`identity` exposes only the held root dev/inode for native adjacent-storage checks.
No release, WIT, manifest schema, ontology or accepted headless policy is replaced.
