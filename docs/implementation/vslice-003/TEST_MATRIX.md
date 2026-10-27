# VSLICE-003 — Conformance Test Matrix

**26 mandatory gates, VS3-001–VS3-026.** Each gate requires a concrete non-ignored executable test, command, expected/observed result, process and component evidence, commit ID and log. Fixture-only and mocks-only tests do not fulfil the real desktop path.

| ID | Required observable evidence |
|---|---|
| VS3-001 | Real Linux desktop process launches with temporary isolated profile and installed mortgage package; capture PID, Qt version, exit code and app hash. |
| VS3-002 | Three-column UI presents usable Operator intent entry, status and canvas; keyboard and accessible naming verified with real Qt. |
| VS3-003 | Entering canonical mortgage request reaches bounded fixture provider through actual desktop boundary, not test-only direct API. |
| VS3-004 | Read-only discovery returns only admitted executable UI-bearing local candidates; candidate version/digest obtained from registry. |
| VS3-005 | Fixture selects a candidate using offered opaque key; no fixed version, digest, filesystem path or mortgage-only selection branch. |
| VS3-006 | GBP 250000, annual 5%, 25 years normalise to accepted field/unit contract; 1461.48 GBP expected after actual WASM execution. |
| VS3-007 | Missing/ambiguous rate, loan amount, term or unit yields clarification; provider invents no values and room is unchanged. |
| VS3-008 | Malformed, duplicate-key, unknown-field, oversized or mistyped suggestions fail before preview/application with typed errors. |
| VS3-009 | Adversarial user and component description instructions cannot override provider boundary, invoke tools, or grant authority. |
| VS3-010 | Preview displays real exact pin, chosen input values/units, intended effects, room and approval requirement; no mutation before approval. |
| VS3-011 | Preview validates accepted component schemas, RDF/UI bindings, canonical planVersion 0.1 and expected room revision. |
| VS3-012 | Real Qt Reject leaves room topology, revision and committed state unchanged; cancellation during prepare similarly safe. |
| VS3-013 | Real Qt Confirm uses single-use proposal-bound approval; replay, double-click, expiry and mismatched room are rejected. |
| VS3-014 | Stale revision, removed/replaced pin or policy/grant drift at apply fails closed with no mutation or substitute release. |
| VS3-015 | Approved canonical plan is applied via accepted ADR-002 kernel and immutable ADR-003 resolution; audit revision/pin. |
| VS3-016 | Typed ingress initialises actual installed mortgage WASM; accepted bus commits output and Qt displays GBP 1461.48. |
| VS3-017 | Ingress failure after topology commit is displayed as truthful partial workflow with explicit retry path; no fake rollback/result. |
| VS3-018 | Subsequent valid user edit updates payment via ingress/bus/WASM and invokes Operator/provider zero additional times. |
| VS3-019 | Invalid user edit, guest error, output validation failure or queue saturation preserves last committed output and surfaces a correlated error. |
| VS3-020 | Room switching and rapid edits never project stale result into selected room; generation/revision guards tested. |
| VS3-021 | Two rooms and concurrent intent requests cannot share approvals, pin choice, private content or committed inputs. |
| VS3-022 | Clean exit and cold restart in second OS process restore exact pinned canvas, values and last output with zero guest/model calls for display. |
| VS3-023 | Offline restore works with provider/network disabled; corrupt/missing pinned component yields non-destructive recovery UI without fallback. |
| VS3-024 | Provider cancellation, deadline, unavailable adapter and malformed metadata fail predictably; redacted logs exclude secrets/raw unrelated history. |
| VS3-025 | Two independent reviews: agent boundary/prompt-injection/privacy and Qt/kernel integration/revision/UX; findings retested and closed. |
| VS3-026 | Hosted Linux CI runs every VS3 gate and retained ABI, IPC, REG, UI, VS1, VS2, OP suites plus Rust, Qt, Python, formatting and Clippy; evidence verified. |

## Evidence rules

- Capture app PID(s), selected installed immutable component pin and UI digest, revisions before/after prepare/apply/ingress, provider invocation count before/after reactive edit and restore, stdout/structured events and exit codes.
- For `VS3-022`, demonstrate two distinct OS PIDs with same isolated durable profile, provider disabled in second process, no guest invocation for rendering.
- CI must run real Qt/offscreen or Xvfb tests and actual WASM/registry path, not merely DTO unit tests; require no skips.
- Keep accepted ABI-001–010, IPC-001–015, REG-001–020, UI-001–020, VS1-001–020, VS2-001–024 and OP-001–024 suites passing. Document actual names if repository differs.
- Captured screenshots are supplemental, not sole evidence. Record test counts accurately after implementation rather than predicting totals.
