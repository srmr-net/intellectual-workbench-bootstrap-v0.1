# Intellectual Workbench — Component Package and Manifest Specification v0.1

**Status:** Proposed normative contract for initial local-registry vertical slice.  
**Dependencies:** Architecture and Interface Specification v0.1, Minimal Semantic Ontology v0.1.  
**Audience:** Component authors (AI agents), runtime, registry, security, integration and verification implementers.

## 1. Terminology and authority

MUST / MUST NOT are mandatory requirements for this proposed v0.1 contract; SHOULD denotes a recommended design; MAY denotes an option. `component.json` is authoritative for package identity, runtime path/digest, permissions, typed ports and file references. RDF expresses semantic discovery metadata and MUST be cross-validated against the manifest. JSON Schema validates data structures, not semantic equivalence. Host grants, not metadata, determine permission. **The public namespace is provisional pending ADR-005.**

## 2. Scope and exclusions

A v0.1 executable bundle contains one WASM component module, one manifest and one RDF Turtle description. Optional abstract UI and verification records are permitted. Pure metadata-only components, multiple executable modules, platform-native plugins, remote dependency fetch at runtime, untrusted QML/JS, signing, and formally verified claims are out of scope. The referenced WASM ABI (`org.workbench.component@0.1`) and archive encoding require ADR-001 and packaging ADR before claiming binary portability.

## 3. Logical package structure

```text
<component-id>-<version>.wbc/   # development directory; extension is a convention
  component.json              # mandatory
  logic.wasm                  # mandatory, hash checked
  capabilities.ttl            # mandatory
  ui.json                     # optional, declarative only
  schemas/input.json          # referenced port schemas
  schemas/output.json
  evidence/tests.json         # optional evidence, never automatic trust
```

References MUST be relative, normalized paths within package root. Absolute paths, `..`, backslashes, empty segments and symlinks are forbidden. Unknown unreferenced files SHOULD be rejected by strict publishers, while local development validators MAY warn. Files MUST NOT be accessed until all referenced paths and required digests validate. v0.1 does not define `.wbc` as a ZIP file format; deterministic archive canonicalization is reserved for the publishing ADR.

## 4. `component.json` contract

The companion `schemas/component.schema.json` is the normative machine-readable manifest-shape validation authority; additional semantic and security constraints in this document require separate implementation. Required top-level fields: `manifestVersion`, `id`, `version`, `runtime`, `metadata`, `permissions`, `ports`. Optional: `ui`, `evidence`. Unknown keys are rejected. IDs are lowercase, dotted reverse-domain-style strings; versions use SemVer; `manifestVersion` is exactly `0.1`. JSON MUST be UTF-8, JSON with duplicate keys MUST be rejected by production parsers. Object key order is not significant.

`runtime.kind` MUST be `wasm-component`; `runtime.interface` MUST be `org.workbench.component@0.1`; `runtime.module` is the payload path; `runtime.sha256` is the lowercase hex SHA-256 of its raw bytes. The runtime module must be a valid executable for the ABI as separately verified by the runtime. A hash is an integrity identity, **not** an authenticity or security proof.

`ports.inputs` and `ports.outputs` are arrays. Port names MUST be unique within each direction. Each port MUST contain `name`, `schema`, `semanticType`; `unit` and `precisionNote` are optional. `schema` points to a valid Draft 2020-12 JSON Schema. `semanticType` and `unit` are absolute semantic URIs. Runtime input/output JSON MUST validate against the referenced schema. Numeric quantities with financial precision SHOULD use explicitly constrained decimal strings. The host MUST reject invalid values before forwarding to components or dependent UI bindings.

`permissions` is an array of explicit requests (`name`, `scope`, optional `reason`). The initial mortgage reference requests `[]`. Permission scope models are provisional: v0.1 native host MUST reject unrecognised capability names and scopes, and must never turn a manifest request into a grant without policy approval. `evidence` files are records of asserted results, not host trust decisions.

## 5. RDF mapping and cross-file invariants

The Turtle file MUST contain exactly one `wb:Component` with `wb:componentId` and `wb:componentVersion` equal to the manifest. Its `wb:hasInputPort` / `wb:hasOutputPort` resources MUST match each manifest port by `wb:portName`, `wb:schemaRef`, `wb:semanticType`, and optionally `wb:unit`. Each component MUST have a `wb:hasArtifact` resource with the same path and SHA-256 as `runtime`. If `ui` is present, `wb:hasInterface` MUST reference that path using `wb:uiDefinitionRef`; absent UI must not be claimed in RDF. Permissions MUST match semantically; for v0.1, any nonempty permission list requires explicit policy-mapped RDF permission resources and an approved capability vocabulary. To avoid divergent permissions, initial conformance fixtures use `[]`. Claims `wb:hasAssuranceClaim` MUST point to existing evidence records and must not be treated as verification by themselves.

Ingestion sequence: schema-validate manifest → validate paths and file presence → hash module bytes → parse JSON Schemas and Turtle → run SHACL → compare manifest/RDF identity, ports, artifact, UI and permission declarations → validate ABI and host permissions → admit to indexed local registry. A failure is transactional: no partially installed component is discoverable.

## 6. Abstract UI

If `ui` is declared, it MUST be JSON conforming to the versioned abstract UI schema defined by Architecture v0.1. Allowed primitive families: `form`, `column`, `row`, `text`, `decimal-input`, `integer-input`, `selector`, `button`, `readout`, `table`, `error-display`. Bindings are declarative paths, not executable expressions. The UI renderer MUST resolve binding paths against planned instance/port schemas and reject invalid bindings. v0.1 prohibits QML or JS supplied in a component package.

## 7. Dependency, version and resolution rules

v0.1 packages do not declare external package dependencies. Component compositions refer to pinned `id`, `version` and SHA-256. Component identity is case-sensitive and version changes never overwrite another published version. The local registry MUST reject conflicting bytes under the same `{id,version}`; it MAY store duplicates by digest while presenting only unique verified releases. No silent upgrades during room restoration.

## 8. Installation and lifecycle

States: `discovered` → `validated` → `installed` → `loadable` → `running` → `stopped`, with explicit `rejected` and `failed` branches. Discovery and validation MUST NOT invoke module code. Installation MUST write to a temporary location and commit atomically after all checks. Loading and running require capability authorisation. Removal MUST not destroy a pinned room's required artifact without an explicit policy; references to missing hashes fail with `COMPONENT_NOT_FOUND`.

## 9. Error contract

Report typed errors from architecture v0.1: `INVALID_MANIFEST`, `UNSUPPORTED_VERSION`, `COMPONENT_NOT_FOUND`, `SCHEMA_VIOLATION`, `CAPABILITY_DENIED`, `COMPONENT_TRAP`, and `INCOMPATIBLE_PORTS`. Package errors MUST include a machine-readable location (`file`, JSON Pointer or RDF subject) where possible. Sensitive host filesystem paths and credentials MUST be redacted from user-facing logs.

## 10. Reference test matrix

| ID | Scenario | Expected |
|---|---|---|
| PKG-001 | Valid manifest + matching file digests | Accept |
| PKG-002 | Unknown manifest field | Reject |
| PKG-003 | Unsupported manifest/runtime interface version | Reject |
| PKG-004 | Absolute, traversal or symlink path | Reject |
| PKG-005 | Missing module or schema | Reject |
| PKG-006 | Hash mismatch | Reject |
| PKG-007 | Duplicate port names | Reject |
| PKG-008 | Invalid JSON Schema | Reject |
| PKG-009 | Invalid Turtle / SHACL shape violation | Reject |
| PKG-010 | Manifest/RDF port semantic mismatch | Reject |
| PKG-011 | Manifest/RDF artifact digest mismatch | Reject |
| PKG-012 | UI binding references missing port | Reject |
| PKG-013 | Unrecognised permission | Reject |
| PKG-014 | Installation interrupted before commit | No partial installation |
| PKG-015 | Same id/version with conflicting digest | Reject |
| PKG-016 | Attempted unsigned origin trusted as verified | Reject trust elevation |

## 11. Immediate implementation tasks

1. Generate strict typed Rust DTOs from the manifest schema (or hand-write with contract tests). 2. Implement duplicate-key-safe JSON parsing. 3. Build file-path/symlink checks and module hashing. 4. Integrate Turtle parser and SHACL checks. 5. Implement exact manifest-to-RDF mapping validation. 6. Provide valid compiled WASM mortgage example with declared WIT interface once ADR-001 is resolved. 7. Add negative conformance fixtures for PKG-002 through PKG-016. 8. Implement transactional local package storage and pins. 9. Connect UI declaration validation. 10. Run package verification before registry indexing.

## 12. Decisions still open

- **ADR-001:** Wasmtime/WASI/WIT pinned revisions and compiled binary conformance.
- **ADR-005:** Stable public ontology namespace and identifiers.
- **New packaging ADR:** `.wbc` canonical archive bytes, signature envelope and deterministic build conventions.
- **Capability ADR:** Registered permission names and typed scope schema.
- **UI contract ADR:** Final `ui.schema.json` and binding grammar.

A binary-compatible published 0.1.0 release cannot be claimed until these decisions and their tests are resolved. This document is sufficient to begin the *manifest, local registry and semantic cross-validation* workstreams in parallel.
