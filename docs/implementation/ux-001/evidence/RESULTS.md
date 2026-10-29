# UX-001 results

Status: **Proposed**. VSLICE-004 remains **Proposed**. No successful live-provider acceptance is claimed.

Implementation: `375b200c85b5164d00ca8b0891436bd900f8e16d`, branch `ux-001-workspace-settings`; baseline master `9c0b27e64dd2d95c46d2f4e1df7bfb83a376dd31` (PR13).

The default `./workbench` presents workspaces/pins/activity, Operator conversation and a trusted semantic canvas. Settings owns read-only Components, the supported provider profile and Diagnostics. Session provider opt-in reuses the accepted adapter and single proposal/approval execution path. WIT, host/kernel interfaces, schema/ontology, registry scope and snapshot format remain unchanged. The product-owned trusted generic renderer differs from the legacy renderer only by suppressing unchanged focus-only submissions; actual edits use the existing typed draft/ingress path.

## Executed local checks

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --locked --all-targets -- -D warnings`: passed.
- `python3 tools/check_contracts.py`: passed (30 fixtures, RDF and reference JSON).
- Full `cargo test --workspace --locked` attempt failed in `wb-operator::proposal_tests::real_admission_preview_approval_ingress_and_replay`: metadata-worker `QUOTA_EXCEEDED`. Other Qt/WASM work was running concurrently. This failure is retained in `local/workspace-tests-attempt-001.txt`; it is not a passing full regression gate.
- Independent UX/accessibility execution: all nine focused Qt cases passed (11 Qt totals including setup/cleanup), zero failures/skips. Real QML keystrokes recomputed 1461.48→1753.77 without another remote transport call. Source-bound review receipts and logs accompany this record.
- Independent security/runtime execution: cases 4/5/6/8/9 passed, zero failures/skips; three provenance negative/positive tests and unchanged-contract inventory checks passed. No actual credentials or inference were used in deterministic tests.

The focused suite covers navigation/Settings, component relocation, explicit approval, actual QML input and WASM, unchanged-focus no-op, errors, truthful partial initialization, stale approval recovery, pin focus, distinct offline cold processes and private passive traces. See CONFORMANCE.md. Two initial findings about misleading saved/partial status and stale refresh were fixed; pin focus and startup recovery were fixed and retested. Development failed test runs are not acceptance evidence.

## Supervised desktop attempt

On 2026-10-29 the user opened the genuine Wayland app, configured the session provider and clicked Ask. The host returned `MISSING_PARAMETER` during local review. Trace evidence records **zero remote attempts**, zero guest invocations, no created room and a clean acknowledged close. App PID 994061; worker PID 994108. `live/attempt-001.jsonl` is allowlisted/sanitized; `live/binaries.json` identifies the exact app/worker and presentation sources. Request text and credential sources/contents are excluded. Model routing selected `nvidia/nemotron-3-ultra-550b-a55b:free`, but no model executed, so usage/latency are unavailable.

The user reported that the flow was wrong and requested a pivot toward a long-running Operator chat experience with component work set aside. This attempt does **not** pass the real OpenRouter walkthrough. No further inference or repeated consent ceremony is being performed. The existing provider is a structured tool-suggestion adapter, not contextual chat; a future conversational mode requires explicit scope and must retain the separate trusted tool approval boundary.

## Remaining gates

Full clean local regression rerun, hosted Linux CI/archive verification and a successful separately supervised live journey remain outstanding. No acceptance status is advanced. Minor review observations about legacy workspace-row wording and read-only Settings hints are documented in the UX review; the new chat iteration must not be treated as covered by these source-bound reviews.
