# Contract status — v0.1

## Upstream baseline

- `references/architecture_interface_v0.1.md` — proposed implementation contract with explicit REQUIRED/CANDIDATE designations.
- `references/ontology/workbench.ttl` — provisional namespace (ADR-005 open).
- `schemas/component.schema.json` — canonical package manifest JSON Schema sourced from Component Package Specification v0.1.
- `references/component_package/SPECIFICATION.md` — authoritative packaging guidance for this scaffold.

## Settled for G0

Trust boundaries; manifest authority over RDF for runtime declarations; no ambient grants; validated declarative UI; reactive computation without Operator; local-first pinned persistence; acceptance CT-01..CT-15 in the architecture spec.

## ADR-gated or unimplemented

- ADR-001 component ABI / runtime pinned version: **OPEN**, WIT not executable contract.
- ADR-002 transport, queue and backpressure: **OPEN**, current bus structures are in-process prototypes.
- ADR-003 Rust↔Qt FFI/threading: **OPEN**, QML is illustrative only.
- ADR-004 SQLite/RDF recovery: **OPEN**, persistence crate is an interface placeholder.
- ADR-005 permanent namespace, SHACL and compatibility: **OPEN**.
- ADR-006 bundle format, digests: **OPEN**; existing package schema retained.
- ADR-007 actual host permission grants: **OPEN**, current capability broker rejects requests.
- ADR-008 decimal arithmetic: **OPEN**; no mortgage reference numerical result asserted.
- ADR-009 Operator interface: **OPEN**.
- ADR-010 development sandbox/merge rights: **OPEN**.

## Conformance status

No CT-01..CT-15 end-to-end passes are claimed. Rust unit tests are scaffolding tests only. Python schema fixture checks run locally when requirements are present.
