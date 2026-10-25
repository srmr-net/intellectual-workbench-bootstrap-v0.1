# Contract status — v0.1

## Upstream baseline

- `references/architecture_interface_v0.1.md` — proposed implementation contract with explicit REQUIRED/CANDIDATE designations.
- `references/ontology/workbench.ttl` — provisional namespace (ADR-005 open).
- `schemas/component.schema.json` — canonical package manifest JSON Schema sourced from Component Package Specification v0.1.
- `references/component_package/SPECIFICATION.md` — authoritative packaging guidance for this scaffold.

## Settled for G0

Trust boundaries; manifest authority over RDF for runtime declarations; no ambient grants; validated declarative UI; reactive computation without Operator; local-first pinned persistence; acceptance CT-01..CT-15 in the architecture spec.

## ADR-gated or unimplemented

- ADR-001 component ABI / runtime pinned version: **Accepted** for the pure stateless WIT ABI and Wasmtime 49.0.1 on hosted Linux; [verified evidence](adr/evidence/ADR-001-hosted-results.md). Production kernel integration remains separate.
- ADR-002 lifecycle, process-local IPC and reactive integration: **Proposed**, executable validation profile implemented; hosted IPC-001..015 evidence pending.
- ADR-003 Rust↔Qt FFI/threading: **OPEN**, QML is illustrative only.
- ADR-004 SQLite/RDF recovery: **OPEN**, persistence crate is an interface placeholder.
- ADR-005 permanent namespace, SHACL and compatibility: **OPEN**.
- ADR-006 bundle format, digests: **OPEN**; existing package schema retained.
- ADR-007 actual host permission grants: **OPEN**, current capability broker rejects requests.
- ADR-008 decimal arithmetic: **OPEN**; no mortgage reference numerical result asserted.
- ADR-009 Operator interface: **OPEN**.
- ADR-010 development sandbox/merge rights: **OPEN**.

## Conformance status

No CT-01..CT-15 end-to-end passes are claimed. Rust unit tests remain scaffolding checks; the separate ABI spike passes ABI-001..ABI-010 with independently compiled guests on GitHub-hosted Linux. Python schema fixture checks run locally and in CI when requirements are present.
