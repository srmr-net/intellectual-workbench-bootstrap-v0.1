# Codex implementation and evidence task — VSLICE-003

## Preconditions

1. Confirm accepted ADR-001–005 and VSLICE-001/002 are merged in the checked-out working branch; check `git status`, compare relevant schemas and contracts before editing. Do not assume PR #9's merge state without local confirmation.
2. Read `AGENTS.md`, accepted ADRs, `apps/desktop-qt/desktop/README.md`, and existing OP/UI/VS2 test harnesses. Identify precise hooks for Operator text input, proposal preview, confirmation, application and room restore.
3. Add `docs/implementation/vslice-003/` files. Do not overwrite implementation-owned evidence or redefine accepted contracts.

## Tasks

A. Provide a real centre-column user entry experience driving bounded Operator preparation through the already accepted `OperatorProvider` interface; deterministic fixture provider is default for all required tests.
B. Show a complete, non-executable preview of actual candidate + pin/inputs, validations and explicit confirmation. Wire Confirm/Reject via proposal-bound host approval, never model-originated authority.
C. Apply canonical plan through existing kernel, then submit inputs using existing typed ingress. Connect result projection to existing trusted semantic Qt renderer. Do not hardcode mortgage logic or QML.
D. Cover true failure modes: stale proposals, invalid/ambiguous user requests, malformed/injected suggestions, rejection, double confirm, ingress partial completion, busy queues and room switch fences.
E. Preserve durable pinning and offline cold restore with no provider/WASM call for display. Include observable counters and separate-process integration harness.
F. Add independently named executable tests mapped to all `VS3-001`–`VS3-026` gates. Keep existing accepted regression suites and WIT unchanged.
G. Obtain independent security/agent boundary review and Qt/host state review. Address findings, rerun tests, archive revisions and reports.
H. Run a clean GitHub-hosted Linux workflow including actual desktop/offscreen Qt, WASM guest build, complete regressions and evidence uploads. Obtain and independently verify downloaded artifact hashes. Set status Accepted only after every mandatory gate is evidenced.

## CI acceptance and evidence

Record exact tool versions, workflow URL, commit SHA, job status, commands, test counts, component/UI hashes, process IDs, timings, approval audit, provider call counts, results and reviewer sign-offs in `evidence/RESULTS.md`. For unsupported optional live-provider testing, report `NOT RUN (optional)` rather than implying coverage. Do not edit acceptance criteria or weaken existing checks to achieve green CI.

Keep new feature changes separate from baseline formatting-only fixes, if any. Distinguish passing local tests from hosted CI evidence.
