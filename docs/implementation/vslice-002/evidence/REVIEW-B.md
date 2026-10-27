# Independent review B — persistence, registry and state boundaries

Reviewer: Codex `/root/review_vs2_persistence`, distinct from implementation author.
Date: 2026-10-26. Integration base: `c117632f7c50ed0cf729fadd814ac8548c22db0d`.
Final test-only follow-up: 2026-10-27.
Implementation was reviewed as a working tree; the adjacent source hash manifest
identifies reviewed bytes independently of a later commit/archive.

## Scope and findings

Reviewed `wb-storage::vs2`, the desktop host, framing, worker, installer and offline
probe; corresponding native/storage tests and snapshot schema; accepted ADR004,
Room/Session restore APIs, VSLICE002 specification and review checklist. No
implementation was edited by this reviewer.

Two initial medium findings were fixed by the author and reviewed again:

1. Retained kernel audit storage grew with every edit. The desktop now recycles a
   runtime after its audit exceeds256 entries; only a later explicit valid edit
   recreates it. Existing kernel/Session behavior is preserved.
2. Valid metadata could exceed the old64KiB response frame. A separate bounded
   desktop transport now permits1MiB; the original accepted UI transport remains
   unchanged. Registry limits128 releases; desktop labels/identities are bounded.

Documentation follow-up was completed: the storage/desktop API READMEs and
implementation map now describe the adapter and actual `Kernel::restore_room`
integration. Final robustness changes were also reviewed: random native getrandom
temporary filenames prevent stale crash-file collisions, and rejected request
tokens/control identities cannot be reflected as oversized response payloads.

## Boundary assessment

Snapshots carry a version, stable room/instance identity, exact component and
UI/schema pins, selected canvas, inputs and complete committed output batches.
Strict recursive JSON parsing, frozen local schemas, content checksum, quotas and
schema/binding checks precede display. Checksums detect corruption; they do not
claim authentication against a malicious account owner.

Every room is verified before a restored projection is exposed. Cold display
creates no kernel runtime and invokes no guest. A complete valid explicit edit
activates the accepted kernel controller with fresh native execution approval;
intermediate restore computations are suppressed. Saved outputs are never
injected into kernel state. Key, revision and desktop epoch checks precede
activation; the desktop epoch is intentionally distinct from kernel generations.

Only a successful complete output checkpoint is persisted and then acknowledged.
Failed input/output batches retain the prior successful input/output pair and
correlated diagnostic. Invalid/incomplete edits execute no guest. No saved grants,
ambient guest access, arbitrary routes, QML or JavaScript are introduced.

Storage holds a private directory descriptor, rejects symlink ancestors/leaves,
nonregular/multiply-linked or public leaves, uses an exclusive writer lock and
revision CAS, writes an exclusive private same-directory temporary file, fsyncs,
renames and syncs the directory. Corrupt/incompatible records and missing exact
pins latch read-only recovery; original workspace bytes survive quit. Directory
replacement cannot redirect an already-open store.

The offline helper probes denial of ordinary IPv4/IPv6 socket creation before
exec; it is a conformance aid, not a general adversarial network sandbox. Native
guest execution still uses the accepted import-free host boundary. Clean-quit
durability is covered; cross-store transactions and power-cut recovery remain
deferred under ADR012.

## Independent execution

- `cargo test -p wb-storage --test vs2`:5 passed,0 failed/ignored,0.31s;
  rerun after final random-temp-name change:5 passed,0 failed/ignored,0.51s.
- `target/debug/wb-desktop-offline /bin/true`: exit0;
  `OFFLINE_IPV4_IPV6_DENIED`.
- `cargo test -p wb-storage -p wb-desktop-host -- --test-threads=1`:
  superseded and interrupted with exit130 during the original72-package stress
  fixture. Prior004/006/007/008/009/010 and140-edit011 gates passed. This interrupted
  command is not recorded as a suite pass. The author replaced the excessively
  costly fixture with an actual-WASM16-room response-size case; its rerun follows.
- Final rerun of that complete command: exit0;15 desktop native gates passed in
  92.13s,5 storage gates passed in0.23s, and the retained storage compatibility test
  passed. No failures/ignored/skipped tests. Original combined output is archived
  in `review-b-native-tests.txt`. This run includes the replacement actual-WASM
  response-size fixture and final malformed-token/control-identity changes.
- `cargo test -p wb-desktop-host --test vs2
  vs2_021_wire_rejects_extra_identity_and_nested_duplicate_keys -- --exact`:
  final malformed-token/control-identity checks pass,1 passed,14 filtered,5.54s.
- Final test-only follow-up: `cargo test --locked -p wb-storage --test vs2`:
  6 passed,0 failed/ignored/skipped,0.27s. Runtime decoding now exercises the
  complete static positive/negative corpus, including a committed snapshot and
  a nested duplicate with the original valid checksum.
- `cargo test --locked -p wb-desktop-host --test vs2
  vs2_020_invalid_snapshot_latches_readonly_recovery -- --exact`:
  1 passed,0 failed/ignored,14 filtered,5.67s. Added cases preserve a real saved
  snapshot's checksum while duplicating an identical nested member, and separately
  seal an invalid output value through FileStore before desktop restore. Both
  fail closed with zero guest calls and preserve original bytes after quit.
  Core implementation hashes remain unchanged. See `review-b-followup-tests.txt`.

The distinct Qt reviewer covers genuine process/GUI instrumentation. This review
does not substitute for GitHub-hosted evidence or claim whole-milestone acceptance.

Decision: APPROVE for the reviewed source hashes and scoped persistence/security
profile. No unresolved implementation finding. This approval is independent of
the required hosted CI evidence and whole-milestone acceptance decision.
