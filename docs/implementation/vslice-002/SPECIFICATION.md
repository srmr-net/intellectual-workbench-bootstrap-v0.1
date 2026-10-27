# VSLICE-002 — Interactive Mortgage Calculator and Persistent Canvas

**Status:** PROPOSED · **Version:** 0.1 · **Type:** product-level integration milestone

## 1. Goal and user-visible outcome

An end user starts the **Linux** Workbench desktop application, chooses an **installed and precisely pinned mortgage component**, sees a trusted declarative form on the canvas, edits principal / annual rate / term, and sees a payment readout calculated through the **actual WASM and ADR-002 room pipeline**. The user can pin the canvas, close the entire process, reopen the Workbench, and recover the **same immutable component/UI identity and last committed values**, with no Operator or model inference.

### Definition of complete

The application launches as a working Qt desktop process and exposes a minimal shell: **left pinned tools/rooms, centre activity/context, right semantic canvas**. The centre may be a non-chat status view; no functioning AI agent is required. UI is instantiated exclusively from **admitted** `ui.json` metadata and the accepted generic renderer. The user can edit all declared mortgage inputs, inspect readable result/errors, pin a workspace, quit and restart the application. No component-name-specific form or stubbed calculation can satisfy acceptance.

## 2. Dependencies, authority, and non-negotiable invariants

- **ADR-001 Accepted:** `wit/component.wit`, synchronous Component Model compute export, Wasmtime, no guest WASI/native imports. Do not modify.
- **ADR-002 Accepted:** host-owned lifecycle, typed message bus, deterministic reactive scheduling, optimistic room revisions, output-batch atomicity, stale-generation/revision fences and bounded queues. Do not bypass.
- **ADR-003 Accepted:** local package admission, immutable exact pins `(id, version, sha256)`, digest/manifest/RDF/ABI checks. Do not rewrite an admitted release.
- **VSLICE-001 Accepted:** real mortgage math and current input/output port schema; preserve all numerical vectors and result rounding. Do not create a new mortgage math implementation for the GUI.
- **ADR-004 Accepted (Linux scoped):** validated `uiVersion: 0.1` semantic UI, strict bindings and schema checks, trusted QML renderer and narrow typed bridge, with UI-bearing package admission separately gated. Preserve generic rendering, UI trust and GUI-thread discipline.
- The uploaded architectural and ADR specifications are design sources; **the current accepted repository code and evidence are authoritative** for exact DTOs, command names and fixtures. Resolve discrepancies explicitly in an implementation note.

## 3. Scope

### In scope

1. A launchable Linux desktop executable via documented command, providing a minimal three-column shell and a generic component picker for **locally installed** UI-bearing components.
2. Selection and exact-pin resolution of the admitted mortgage UI package, room/instance creation or reuse, and generic semantic form rendering.
3. Editable principal, annual percentage rate, and term, bound to the **actual accepted aggregate mortgage input**. Draft edits are validated as a complete input object; only valid edits reach ADR-002 ingress.
4. Calculated monthly payment projected from the **committed output state**; no direct WASM-to-QML view updates.
5. Visible and accessible validation errors; no computation or committed revision for malformed or incomplete draft; recoverable queue/conflict/failure states.
6. Pinning the canvas in the left column and switching between at least two independent rooms without cross-room events.
7. Durable **snapshot persistence** of pinned room metadata, component/UI exact pins, semantic UI layout selection and the last fully committed input and payment; clean quit, cold start and rehydration with no AI.
8. Safe handling of unavailable/mismatched pins and corrupt/incompatible snapshots; no silent component replacement, output fabrication or privileged fallback.
9. End-to-end UI tests on Linux plus full regression and hosted CI evidence.

### Out of scope

Operator/chat reasoning, cloud models, remote registries, installation via browser, dynamic/generated QML, Windows/macOS packaging, general-purpose notebook, richer charts/layouts, translation/localisation, multi-user sync, encrypted multi-device backup, custom SMMED database engine, historical transaction replay, automatic silent upgrades, production-level crash-proof journaling, native guest privileges or changes to WIT.

## 4. User workflow and shell

1. Start `workbench-desktop` using the documented command, in an ordinary window or a headless Qt test environment.
2. Select *Installed components → Mortgage calculator* (display name sourced from admitted manifest).
3. Host resolves exact immutable pin and builds an instance in a room; validated `ui.json` instantiates the trusted generic form renderer in the right canvas. Initial values come from either a new room's explicit defaults or the restored committed snapshot.
4. Change principal, rate or term. Draft UI values may be displayed optimistically but **readout must be visibly labelled as stale/pending or remain prior committed** until the result is committed.
5. On valid completed input, submit an edit with current room revision/execution generation. WASM computes; host validates/commits; projection changes payment readout and revision.
6. Invalid edit yields error attached to the field and accessible diagnostic. There is no guest invocation and no committed revision change.
7. Pin room: left sidebar records stable room identity and label; switching rooms unsubscribes old projections and changes canvas identity.
8. Quit app normally, confirm persistence completion, relaunch as a **new OS process**, select pinned room and verify exact pins, values, last committed result and controls. Neither HTTP nor LLM access is required.

## 5. Persistent room snapshot v0.1

A host-owned, versioned durable record; the precise backend may use existing storage abstraction and SQLite/JSON file as long as semantics are met. **Do not invent a database engine in this task.** Store normalized state, not raw Qt widgets or executable QML. Suggested logical fields (map into accepted DTOs; these are NOT a new public JSON contract):

```
format_version: 1
room_id: stable opaque id
room_revision: monotonic committed revision
pinned: boolean
label: display-only string
instances: [
  { instance_id, component_id, component_version, artifact_sha256,
    ui_identity_or_digest, lifecycle_rehydration_metadata }
]
canvas: { selected_instance_id, semantic_ui_id }
committed: { input_by_instance, output_by_instance }
```

### Snapshot rules

- **Only commit-consistent state** is persisted; never persist a half-completed output batch or draft as a committed result.
- Persist after accepted state transitions and pin/layout changes. A documented debounce is allowed only if a final flush completes before normal quit; acceptance checks must prove no acknowledged state is lost during clean shutdown.
- Write a new complete snapshot to a private temporary file in the same directory; flush, atomically rename, and apply directory sync where supported, or use an established SQLite transaction. Reject truncated, duplicate-key, unsupported-version and structurally invalid records; size/instance counts must be bounded. Protect against path traversal and symlink escape.
- At launch, load/validate snapshot **before** materialising room instances. Resolve each exact pin and require the UI identity/digest to match accepted immutable registry data. Only then restore committed values to the trusted renderer. No guest compute and no AI required purely to display restored committed values.
- A mismatch or corrupted record must display a clear recovery state, preserve the original bytes for diagnosis and prevent unsafe partial restoration; do not silently reset the user's workspace.
- If design requires cross-store consistency beyond the available implementation, document limitations, provide a deterministic recovery/reconciliation policy and isolate the scope. Full power-loss crash recovery remains deferred.

## 6. State and thread boundaries

- GUI thread owns QObjects, widget creation, focus and accessible UI changes.
- Background worker handles registry resolution, WASM execution and persistence I/O; no synchronous blocking I/O on the GUI thread during steady interaction.
- UI sends **typed edit requests** referring to the currently selected room/control and expected revision; Rust remaps to approved bindings and canonical bus provenance. UI cannot choose arbitrary target ports, impersonate a component or grant capabilities.
- Accepted drafts use the existing aggregate mortgage input schema. Decimal text normalization is presentation-only; canonical payload remains an accepted decimal string.
- Commit-to-render uses room+instance+revision+generation fencing; stale projections or disposed views cannot overwrite current readout.
- During app shutdown, stop accepting edits, drain or cancel work per documented policy, flush committed state, detach subscriptions and destroy Qt-owned objects on the GUI thread.
- `QUEUE_FULL`, `STATE_CONFLICT`, guest failure and storage failure yield visible diagnostics. Do not retry privileged or non-idempotent operations automatically.

## 7. Persistence and migration policy

Initial snapshot schema version is `1`. Unknown versions fail visibly. No migration is required beyond the current version; migration support is a future task. An existing headless room lacking UI metadata remains headless/unavailable rather than being silently decorated with a new UI. An immutable component release pinned before a UI change remains resolvable; create a new release for changed UI bytes. Running offline is a tested requirement for restored local rooms.

## 8. Delivery / acceptance

- Implement `VS2-001`–`VS2-024` in `TEST_MATRIX.md` as individually identifiable executable tests (test-to-evidence mapping required).
- Include a real `QT_QPA_PLATFORM=offscreen` or Xvfb Qt integration invocation **and** separately verify launch as a new app process; neither a mocked widget tree nor a direct-WASM calculator call qualifies.
- Preserve all accepted ABI, IPC, REG, UI, VSLICE-001 conformance gates; keep WIT, domain schemas, ontology and immutable releases unchanged.
- Obtain two independent reviews and a successful Linux GitHub-hosted CI run; archive versions, hashes, logs, test counts, screenshots (supplemental only), review findings and run URL.
- Mark milestone **ACCEPTED** only after both reviews and archived hosted evidence meet all gates.
