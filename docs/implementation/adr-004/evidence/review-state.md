# Independent admission, state and restoration review

Reviewer: `/root/review_ui_state`, with no implementation authorship. Date: 2026-10-26. Status: APPROVED for the reviewed source bytes below; this is not ADR acceptance.

Read repository instructions, accepted ADR-003/VSLICE integration and ADR-004 specification/task/matrix. Reviewed the new registry opt-in/descriptor verification, Python and Rust binding validators, strict transport DTOs, Session drafts/revisions/generations, native worker framing/shutdown, and logical snapshot persistence. Existing kernel, bus and accepted WIT remain authoritative.

## Findings and corrections

- UI admission formerly permitted `form.json`, while unchanged room UI references require basename `ui.json`. Independently reproduced a real worker failure after successful immutable installation. Admission now rejects this incompatible basename in both validators; a regression test covers it.
- Rejected or full-queue Shutdown formerly ended the native reader/main loop. Correction terminates only an accepted request, acknowledges final stdout flush and avoids joining a reader blocked on open stdin. Reviewed the actual wire regression, including refresh after forged shutdown.
- Snapshot authority now checks the opened root inode against the registry, uses a held private directory descriptor, rejects symlink/nonregular/multiply-linked/oversized restoration leaves and writes via create-new temporary file, file sync, atomic rename and directory sync. This is scoped host storage, not a new durable database contract.
- Every accepted input is saved before publishing its committed projection. Qt restoration now checks a newer edit made after explicit Save, so stopping/reopening must preserve the latest displayed result.
- Generic registry loading remains profile-neutral, preserving mixed-store headless quota/recovery; only admission/UI reading require opt-in. CheckedMetadata omits absent UI descriptors to preserve headless records.

No unresolved source-level blocker identified in these corrected paths. Session validates ownership/generation before drafts, checks revision before submission and commits only complete valid aggregate input through the existing Room adapter. Restoration checks exact pin and UI reference and obtains fresh explicit native approval. Last successful output values remain visible with failure diagnostics under existing kernel semantics.

## Independent checks

- `REGISTRY_PYTHON=/tmp/wb-spec-001-venv/bin/python cargo test --locked -p wb-ui-host --test ui`: 16 passed; wire test failed at its 45-second receive deadline under concurrent test load; 138.43 seconds total. Failure is recorded rather than counted as passing.
- An isolated wire rerun also reached the same 45-second harness timeout while the preceding full suite was still running. A subsequent independent raw-wire probe with a 120-second per-receive deadline passed (details below).
- `PYTHONPATH=tools /tmp/wb-spec-001-venv/bin/python -m unittest tools.test_ui_contract`: passed, one corpus test with positive/negative fixture subcases.
- Earlier targeted `ui_008` mixed-store legacy-policy check passed. Real native-worker renamed-UI reproduction established the basename defect before its correction.
- Initial Python invocation without PYTHONPATH failed import resolution; corrected command above passed.

Hosted Linux CI, broader regression gates and final acceptance are separate mandatory evidence. Reviewer approval cannot replace them. No claim is made for power-cut recovery, hostile same-UID process isolation or multi-window concurrent room writers beyond the documented scope.

## Final focused result and review boundary

A real `target/debug/wb-ui-worker` was launched with the generated mortgage UI package, a fresh temporary registry, `/tmp/wb-spec-001-venv/bin/python`, the real registry metadata worker and ABI inspector. Python sent length-prefixed JSON over actual stdin/stdout. With client stdin kept open, it received Ready, sent wrong-room Shutdown and observed rejection, sent valid Refresh and observed acceptance, then sent valid Shutdown and observed `saved` plus exit status 0. Each receive had a 120-second deadline. This independently validates the corrected authority/liveness path without relying on Session-only tests. The earlier 45-second harness failures remain recorded above.

Approval covers admission/state/restoration source inspection and these independent checks. No open code blocker remains in that scope. The source digest sidecar binds this review to the exact bytes; later implementation edits require review of affected bytes. This approval does not claim successful full-suite execution or hosted CI.

## Reviewed source SHA-256

- `crates/wb-contracts/src/ui.rs`: `35ff35758ece0187d94c478376b17abc8e3244b80430f01e859dfb4f6dc910ea`
- `crates/wb-semantic/src/ui.rs`: `05afda332babfc5ce36c6d31bd3ce517c27aa4b955f1bc130c483e240329a34e`
- `crates/wb-registry/src/lib.rs`: `4cc72b81102c290c2fa7362833d3dd954d38b2c3b4de5a7539f7ab9d45af3134`
- `crates/wb-ui-host/src/lib.rs`: `5e3b48f5315f9d897c4cfdd10b639db3b1541abceba920f35bd779c8f5667d50`
- `crates/wb-ui-host/src/framing.rs`: `b71e1d767bb7a63ab27b15d0621e840c1f3efe469e4bb30fff207cfa9258185b`
- `crates/wb-ui-host/src/bin/wb-ui-worker.rs`: `1de21f4e4be6e518c9576d1e320557e5f1fe351334b61f5d2c3b992dd0538339`
- `crates/wb-ui-host/tests/ui.rs`: `3075e993b2560c53b32e7927f4f61f2d9f9939ac37582cb673230d269e19bb12`
- `tools/registry_validate.py`: `077895af41893cd859096daefa92d4552b1806a3400283a2dcb21acb98efb128`
- `tools/ui_contract.py`: `dee8d0c43d5e4e5192f50a156420fa6b0616bab7f8f9ade03d2dd3444038d9ef`
- `schemas/ui.schema.json`: `f183fe5d5566d6420fbf6c6ab052a7aa7dc86e41e694a5371551ef51b55c5380`
- `schemas/ui-bootstrap.schema.json`: `2d43149da0b6c844504fea681028c9aefab533628bcf5cb9b893a5885d4af11e`

Follow-up test-only review: the wire receive deadline changed from 45 to 120 seconds and stage names were added to timeout diagnostics. All ownership, refresh, shutdown and exit assertions remain intact; production startup limits are unchanged. Author-provided `target/ui-wire-test.txt` records the original 45-second harness passing in 17.94 seconds when isolated. This is corroborating author evidence, distinct from the independent raw-wire probe above. Reviewed test hash refreshed; all other reviewed source hashes remain unchanged.

## Draft acknowledgment follow-up

Independently inspected the Qt bridge ACK correction: pending draft versions now include only the submitted Edit field or explicit Apply fields; Save/Refresh carry none. Existing version comparison still preserves a newer local draft when an older reply arrives. This closes the observed cross-field draft-loss path without altering Rust input/commit semantics. The genuine Qt test retains an unsent term edit through Save and an accepted rate edit, then requires explicit Apply to produce 0.50 and clear that draft. Source inspection approved; focused Qt execution is performed by the other independent reviewer, not counted here as independently run.

Additional reviewed hashes:
- `apps/desktop-qt/src/UiBridge.cpp`: `64a08b5b67d07160a9a5ab9af382399b870d214cd90296bbcbe6c6e5960f1bc5`
- `apps/desktop-qt/tests/UiConformance.cpp`: `ff0b0c34dead20d0cd3e6f45252a6a15097c8fd77abb2f9264e98401be5d474b`
