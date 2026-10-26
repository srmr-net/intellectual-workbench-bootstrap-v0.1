# ADR-004 Validation Evidence — template

**Decision:** PROPOSED  
**Repository commit:** TODO  
**PR:** TODO  
**Hosted Linux CI run URL:** TODO  
**Date:** TODO

## Environment/toolchain
- Host OS/architecture: TODO
- Qt/Qt Quick version and offscreen platform: TODO
- Rust/Cargo version: TODO
- Rust↔Qt bridge implementation/version: TODO
- Wasmtime and guest toolchain (unchanged pins): TODO
- Lockfiles and hashes: TODO

## Gate report
| Gate | PASS/FAIL/BLOCKED | Test name/command | Evidence file/artifact hash |
|---|---|---|---|
| UI-001 | TODO | TODO | TODO |
| UI-002 | TODO | TODO | TODO |
| UI-003 | TODO | TODO | TODO |
| UI-004 | TODO | TODO | TODO |
| UI-005 | TODO | TODO | TODO |
| UI-006 | TODO | TODO | TODO |
| UI-007 | TODO | TODO | TODO |
| UI-008 | TODO | TODO | TODO |
| UI-009 | TODO | TODO | TODO |
| UI-010 | TODO | TODO | TODO |
| UI-011 | TODO | TODO | TODO |
| UI-012 | TODO | TODO | TODO |
| UI-013 | TODO | TODO | TODO |
| UI-014 | TODO | TODO | TODO |
| UI-015 | TODO | TODO | TODO |
| UI-016 | TODO | TODO | TODO |
| UI-017 | TODO | TODO | TODO |
| UI-018 | TODO | TODO | TODO |
| UI-019 | TODO | TODO | TODO |
| UI-020 | TODO | TODO | TODO |

## Regression summary
ABI-001–010: TODO; IPC-001–015: TODO; REG-001–020: TODO; VSLICE-001–020: TODO.
Rust/Clippy/rustfmt/Python contracts: TODO.

## Integrity
- UI-bearing component pin + binary SHA-256: TODO
- UI file/schema digest: TODO
- Lockfile versions: TODO
- CI artifact names and SHA-256: TODO

## Independent reviews
1. Qt threading, accessibility and executable UI trust — reviewer, date, findings, retests: TODO
2. Registry admission, state, stale fence and restoration — reviewer, date, findings, retests: TODO

## Exceptions and architecture deviations
TODO (none only if verified).

## Acceptance
All mandatory tests ran and passed in hosted CI? TODO.
Decision approver and acceptance date: TODO.
