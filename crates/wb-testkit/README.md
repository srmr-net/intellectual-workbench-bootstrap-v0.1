# wb-testkit

Version 0.1. The IPC integration suite in tests/ipc.rs implements IPC-001..015
and regression cases for wave ordering, pending UI during active work, descendant
cancellation, input/output name collisions, fanout backpressure, live revocation
and reservation rollback. It invokes actual Wasm components through wb-kernel and
the unchanged ADR-001 adapter. Concurrent mailbox producers test monotonic ordering.

Build guests first with bash examples/wasm-components/build.sh, then run
cargo test --locked --workspace. Missing binaries fail; there are no skip paths.
Existing ABI tests and frozen WIT are retained. Integer arithmetic avoids ADR-008.
fixtures/ipc contains canonical JSON envelope positives/negatives. The aggregate
fixture is a separate import-free guest compiled from the same authoritative WIT.

ipc_015's local declaration test is not hosted evidence. The evidence collector
records all named IPC/ABI successes, versions, hashes and tested SHA; acceptance
requires an actually successful GitHub-hosted Linux workflow and independent review.
Full reproduction: bash examples/wasm-components/validate-ipc.sh.
