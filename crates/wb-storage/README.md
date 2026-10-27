# wb-storage

The existing CONTRACT_VERSION/placeholder test remains compatible. VSLICE002 adds
`vs2`, an internal Linux v1 clean-quit snapshot interface; ADR012 database recovery
remains open. It grants no execution rights and serializes neither Store memory
nor saved capabilities.

`FileStore::open(absolute_path)` anchors every ancestor without following symlinks,
creates only the final private directory, checks owner/mode and holds a no-follow
regular single-link `.workspace.lock` with exclusive nonblocking flock. A second
writer fails WORKSPACE_BUSY. Normal host use chooses a trusted/private parent.
`load` returns absent or a fully decoded Workspace; it never repairs/deletes bytes.
`commit(expected_revision, candidate)` checks the current snapshot and CAS, seals a
complete candidate, writes a unique private same-directory temp, fsyncs, atomically
renames, fsyncs directory and returns the committed copy. A random native-kernel
nonce prevents reuse of temp names left by prior crashes. Such leftover files are
ignored/preserved for diagnosis; full crash-journal/retention recovery is deferred.

Internal format1 contains workspace revision/selection, up to16 exact-pinned room
checkpoints, stable instance/room identity, UI path/raw/schema digests, pin flag,
label, unchanged RoomSnapshot inputs and a coherent committed output map. Canonical
content SHA-256 detects accidental data corruption; it is not a signature against
a malicious trusted host UID. The desktop layer verifies exact registry pins and
full authoritative port schemas before any room/view becomes available.

The loader rejects files over1MiB, duplicate keys at any depth, unknown fields,
unsupported versions, structural/schema/identity mismatch and checksum corruption.
It opens snapshot leaves with NOFOLLOW|NONBLOCK, regular-file/nlink/owner/mode
checks, and checks size before/after bounded read. The internal JSON Schema is
fixtures/vslice-002/snapshot-v1.schema.json; canonical room/composition references
are registered locally with network retrieval unavailable. Neither canonical
manifest/domain/UI schema nor ontology is modified.

Clean-shutdown guarantee: every acknowledged state/selection/pin transition was
already fsynced before presentation. Atomic replace prevents torn current records;
this does not settle physical power-cut, filesystem fsync failure, database replay
or cross-store deletion transactions. A failed directory sync is indeterminate and
must enter explicit read-only recovery rather than retry or claim acknowledgement.

Run `cargo test --locked -p wb-storage`. Whole-product acceptance uses
`bash apps/desktop-qt/validate-vslice-002.sh` plus independent review/hosted evidence.
