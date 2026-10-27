# VSLICE-002 — Conformance test matrix

**24 mandatory gates** (`VS2-001`–`VS2-024`). Record real automated test names/commands, measured outcomes and artifacts in `evidence/RESULTS.md`. Do not mark tests passed based only on unit mocks or screenshots. Tests must be non-ignored, non-skipped.

| ID | Required observable outcome |
|---|---|
| VS2-001 | Documented Linux launch command starts **real desktop process**, Qt initialises, exits cleanly; capture exit code, Qt version and app binary hash. |
| VS2-002 | Three-column shell appears with left pin selector, centre status/context, right canvas; visible status is usable without Operator. |
| VS2-003 | UI-bearing mortgage component selected from **installed registry metadata**, not a hard-coded release path or mortgage-only branch. |
| VS2-004 | Exact component version+SHA-256 pin and admitted `ui.json` identity selected; tampered/unavailable pin fails visibly. |
| VS2-005 | Trusted generic renderer constructs declared fields and output readout from accepted semantic UI; no component-authored QML or JS executed. |
| VS2-006 | Actual manifest/input schema/UI bindings match accepted component; all three input fields route through permitted aggregate port and output is read-only. |
| VS2-007 | Fresh room computes at least two distinct validated mortgage scenarios, matching existing VSLICE-001 vectors/rounding through real installed WASM and bus. |
| VS2-008 | Valid input edit produces a real host ingress event and committed room result; Qt readout never substitutes local mortgage math. |
| VS2-009 | Invalid decimal/negative principal, incomplete form and invalid term show field-specific accessible diagnostics, with **no compute or committed revision advance**. |
| VS2-010 | Changing values rapidly cannot display stale result after newer committed revision; cancellation/generation fences exercised. |
| VS2-011 | `QUEUE_FULL` handling is user-visible and doesn't silently discard committed projections; bounded resource behavior measured. |
| VS2-012 | `STATE_CONFLICT` displays deterministic repair/reapply flow; no silent revision override or unnoticed auto-retry. |
| VS2-013 | Guest error/trap and output validation failure retain last good committed value, surface correlated diagnostic, and never commit partial result. |
| VS2-014 | UI and Rust bridge respect Qt GUI-thread ownership; rapid switch/teardown does not invoke deleted objects or leaked subscriptions. |
| VS2-015 | Keyboard-only edit, label association, focus retention, tab order and accessible errors verified by actual Qt instrumentation. |
| VS2-016 | Two rooms hold different pins/inputs/output states without cross-room contamination; forged room/instance/control edit rejected. |
| VS2-017 | Pin/unpin controls persist room identity and canvas selection across navigation; view is generic and metadata-driven. |
| VS2-018 | **Cold process restart** restores exact pinned component/UI, selected room, last committed inputs and payment without guest execution or AI solely for rendering. |
| VS2-019 | Clean shutdown flushes acknowledged writes; repeated quit/start and restore produce stable versioned snapshots without drift. |
| VS2-020 | Corrupt/truncated/unsupported snapshot and missing/mismatched pinned component produce explicit non-destructive recovery UI; no silent substitute. |
| VS2-021 | Snapshot loading enforces size, duplicate-key, path/symlink and schema limits; no unsafe payload/QML execution or elevated permissions. |
| VS2-022 | Offline restore succeeds with denied network/AI services; no mandatory remote dependency and no implicit downloads. |
| VS2-023 | Two independent reviews completed, findings fixed and retested: (A) Qt UX/accessibility/threading and (B) state/persistence/registry/security. |
| VS2-024 | All 24 gates plus accepted ABI-001–010, IPC-001–015, REG-001–020, UI-001–020 and VSLICE-001–020 pass in **GitHub-hosted Linux CI**, including real Qt/offscreen app test, Clippy/formatting/Python checks; archived logs/hashes and run URL. |

### Evidence minimum

For every gate cite a named test, executable command, log path and commit hash. UI tests must exercise the installed component and trusted bridge, not fixture-only projections. Collect screenshots or video only as supplemental evidence. For persistence, prove process boundary using **two separate PIDs** and describe the storage location and cleanup policy in temporary test directories. Explicitly distinguish clean shutdown durability from power-loss crash consistency.
