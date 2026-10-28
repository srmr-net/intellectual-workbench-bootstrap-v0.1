# Bootstrap backlog (dependency ordered)

All tasks require changed-code tests, documented interfaces and agent verification evidence. Gate names follow Architecture Specification §16.

| ID | Gate | Depends on | Owns | Exit condition |
| --- | --- | --- | --- | --- |
| SPEC-001 | G0 | none | schemas/, fixtures/, tools/ | Valid/invalid fixtures for manifest, composition, UI, room; JSON Schema checks |
| SPEC-002 | G0 | SPEC-001 | wb-contracts | Rust DTOs round-trip JSON and validate versioned types |
| RUNTIME-001 | G1 | ADR-001, SPEC-002 | wb-wasm-host, components/ | Two independently compiled modules load; traps isolated (CT-01/13) |
| SEC-001 | G2 | ADR-007, SPEC-002 | wb-capabilities | Undeclared host access rejected/audited (CT-05) |
| GRAPH-001 | G2 | SPEC-001, ADR-005/006 | wb-semantic, wb-registry | Invalid package rejected; type and unit mismatches refused (CT-02/03/04) |
| BUS-001 | G2 | ADR-002, SPEC-002 | wb-bus | Bounded typed envelopes and routing; backpressure tests |
| REACTIVE-001 | G3 | BUS-001, SPEC-002, ADR-008 | wb-reactive | DAG, stale revision rejection, deterministic mortgage (CT-07/08/09) |
| UI-001 | G3 | SPEC-001, ADR-003 | apps/desktop-qt | Unknown/executable UI rejected; native controls (CT-06) |
| STORAGE-001 | G4 | SPEC-001, ADR-012 | wb-storage | Pinned restore, crash recovery, no stale grants (CT-10/11/12/14) |
| VSLICE-001 | Headless | accepted ADR-001/002/003 | mortgage guest/package, native headless adapter, tests/evidence | Real exact-pinned registry→kernel→WIT→room calculation; VSLICE-001..020 and all regressions pass; [task](vslice-001/TASK.md) |
| DEMO-001 | G3–G4 | runtime, graph, reactive, UI, storage | integration only | Mortgage UI updates without Operator, save/reopen |
| AGENT-001 | G5 | SPEC-002, SEC-001 | wb-kernel | Structured plan validation before mutating room (CT-15) |
| BOOTSTRAP-001 | B1 | G5, ADR-010 | development components | Generate/test/install new module under approval |

**Suggested parallel lanes after SPEC-001/002:** runtime spike; semantic/registry; UI bridge spike; storage design spike. No concurrent edits to shared schema contract without synchronisation.

VSLICE-002 (Accepted, 2026-10-27): first installed interactive desktop/persistent canvas;
owner Codex, [task](vslice-002/TASK.md), dependencies accepted ADR001–004/VSLICE001.

OP-001 / ADR005 (Accepted, 2026-10-27): bounded local single-component Operator; owner Codex,
[task](adr-005/TASK.md), depends accepted ADR001–004 and VS1/VS2.

VS3-001 (Accepted, 2026-10-28): complete bounded desktop Operator journey; owner Codex;
[task](vslice-003/TASK.md); depends accepted ADR001–005 and VS1/VS2.

LLM-001 / ADR006 (Proposed): host-controlled opt-in live provider; owner Codex;
[task](adr-006/TASK.md); depends accepted ADR001–005 and VS1–VS3.
