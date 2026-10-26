# ADR-003 Validation Results — TEMPLATE

**Date:** TBD  
**Commit:** TBD  
**PR:** TBD  
**Hosted workflow URL:** TBD  
**Status:** PROPOSED until all acceptance gates are met

## Environment and exact pins
- OS/kernel:
- Rust/Cargo:
- Wasmtime:
- cargo-component:
- `component.schema.json` SHA-256:
- Ontology/SHACL SHA-256:
- Toolchain and guest lockfile SHAs:

## Gate matrix
| Gate | Local result | Hosted result | Evidence/log file | Notes |
|---|---|---|---|---|
| REG-001 | NOT RUN | NOT RUN | — | — |
| REG-002 | NOT RUN | NOT RUN | — | — |
| REG-003 | NOT RUN | NOT RUN | — | — |
| REG-004 | NOT RUN | NOT RUN | — | — |
| REG-005 | NOT RUN | NOT RUN | — | — |
| REG-006 | NOT RUN | NOT RUN | — | — |
| REG-007 | NOT RUN | NOT RUN | — | — |
| REG-008 | NOT RUN | NOT RUN | — | — |
| REG-009 | NOT RUN | NOT RUN | — | — |
| REG-010 | NOT RUN | NOT RUN | — | — |
| REG-011 | NOT RUN | NOT RUN | — | — |
| REG-012 | NOT RUN | NOT RUN | — | — |
| REG-013 | NOT RUN | NOT RUN | — | — |
| REG-014 | NOT RUN | NOT RUN | — | — |
| REG-015 | NOT RUN | NOT RUN | — | — |
| REG-016 | NOT RUN | NOT RUN | — | — |
| REG-017 | NOT RUN | NOT RUN | — | — |
| REG-018 | NOT RUN | NOT RUN | — | — |
| REG-019 | NOT RUN | NOT RUN | — | — |
| REG-020 | NOT RUN | NOT RUN | — | — |

## Commands and logs
Record exact commands, exit codes, stdout/stderr paths and unmodified test totals.

## Admission fixture hashes
Record raw WASM sha256, canonical manifest metadata hashes and registry record content.

## Safety review
Document reviewer, review commit, path/symlink/race tests, crash injection and recovery details.

## Limitations / blocked items
Distinguish simulated failures from actual OS kill/restart tests and partial test support from full enforcement.

## Acceptance decision
Do not change status without an actual hosted run link, verified artifact hashes and all mandatory passing gates.
