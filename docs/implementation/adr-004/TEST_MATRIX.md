# ADR-004 conformance test matrix

All twenty gates are mandatory; tests must be real and independently verifiable. Record test command, pass/fail, artifact paths and exact commit in `evidence/RESULTS.md`. No acceptance based on local-only evidence.

| ID | Required observable result |
|---|---|
| UI-001 | `uiVersion=0.1` valid reference fixture passes strict schema validation, including duplicate-key-safe parser; unknown version rejected. |
| UI-002 | Reject unknown control types/fields, nested executable strings/JS/QML references and unsafe resource URLs before any UI instantiation. |
| UI-003 | Unique IDs, mandatory accessible labels, text/size limits, form-only root and 32-control limit enforced. |
| UI-004 | Direction checks reject writable readout, output-as-input and undeclared manifest ports. |
| UI-005 | Input and output JSON Pointer fields resolve against actual local port schemas; non-existing fields, wrong scalar kind and remote refs rejected. |
| UI-006 | Manifest ↔ RDF ↔ UI reference reconciliation rejects mismatches; accepted ontology is not modified. |
| UI-007 | Registry extends admission to UI-bearing packages with path safety and immutable content; tampered UI metadata and attempted release overwrite rejected. |
| UI-008 | Prior headless packages continue to install, resolve and execute unchanged, and retain `None` UI. |
| UI-009 | Trusted renderer creates form/controls dynamically from admitted metadata; no component-ID-specific QML branches. |
| UI-010 | Host rejects arbitrary QML/JS injections and never evaluates package-controlled executable code. |
| UI-011 | Genuine Qt offscreen widget test verifies form appearance, labels, ordering, keyboard tab navigation and accessible roles. |
| UI-012 | Rust→Qt projection changes occur on GUI thread; bridge teardown/room switch causes no use-after-free or leaked callbacks. |
| UI-013 | UI edit passes typed binding and full input JSON Schema validation; malformed decimal, integer and incomplete input cause no compute or room revision change. |
| UI-014 | Valid user edit passes through real ADR-002 ingress, invokes installed WASM, and displays **committed** resulting mortgage payment, no direct guest→QML shortcut. |
| UI-015 | Revision/generation fence rejects late completion and stale/out-of-order projection; active UI never regresses after room switch. |
| UI-016 | Bounded event queue signals `QUEUE_FULL`; uncommitted edit coalescing does not erase committed events. |
| UI-017 | Two rooms with different values remain isolated; no forged source/room/permission or cross-room edit is accepted. |
| UI-018 | Quit, restore and reopen display exact pinned component UI and last committed input/payment without AI; unavailable pin fails explicitly. |
| UI-019 | Independent Qt/security review **and** independent state/registry review completed, findings resolved with retest evidence. |
| UI-020 | Green GitHub-hosted Linux Qt/offscreen CI, all UI gates, existing ABI-001–010, IPC-001–015, REG-001–020, VSLICE-001–020, formatting/Clippy/contracts; archived hashes and logs. |

For UI-014 confirm at least two distinct inputs and expected numerical outputs using the accepted VSLICE-001 numerical fixture methodology. For UI-011 record accessibility inspection instrumentation rather than claiming accessibility from a screenshot. Review records must name a reviewer independent of the implementation author/agent context.
