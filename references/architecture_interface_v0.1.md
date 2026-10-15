# Intellectual Workbench
## Architecture and Interface Specification — Version 0.1

**Status:** Proposed implementation baseline  
**Date:** 2026-10-13  
**Applies to:** Initial Linux desktop vertical slice and portable Rust component infrastructure  
**Derived from:** Product Plan v0.2 (conversation), `product_design(1).md`, `brainstorming(1).md`  
**Audience:** Architecture, implementation, integration, verification and release agents

> **Decision status.** Statements marked **REQUIRED (v0.1)** are proposed binding implementation contracts for the first vertical slice. Statements marked **CANDIDATE** require an architecture decision record (ADR) or spike before implementation. The original design documents do not settle all of these details. This document makes conservative proposals; adopting this specification formally requires approving the open ADRs.

## 1. Scope and goals

### 1.1 Purpose

The Workbench is a local-first, agent-operated, component-based desktop environment. Users ask **Operator** to assemble reusable functionality into interactive, persistent workspaces. Computation runs primarily in WebAssembly components; capabilities, typed inputs/outputs, permissions and abstract UI are described in semantic metadata. A Qt/QML implementation renders those interfaces, while a Rust host enforces execution, security and persistence rules.

### 1.2 v0.1 demonstration contract

The first vertical slice **MUST** demonstrate: (1) loading independently built components; (2) discovering a mortgage calculator from metadata; (3) validating and instantiating a composition; (4) rendering inputs/results from abstract UI declarations; (5) changing inputs and recomputing without invoking Operator; (6) persisting and restoring the room; and (7) rejecting an unauthorised host capability request.

The build MAY use a scripted/stub Operator for the initial deterministic demonstration. A natural-language Operator and developer component workflow are later increments using the same interfaces.

### 1.3 Non-goals

The first slice does **not** require autonomous coding, a custom database engine, cloud registries, arbitrary QML execution, full OWL reasoning, formal proof of arbitrary components, multi-user collaboration, cross-device synchronisation, or universal native platform packaging.

## 2. Normative conventions and design invariants

**MUST**, **SHOULD** and **MAY** denote requirement strength. All identifiers and schemas below are proposed Workbench conventions, not established external standards.

**INV-01 — Kernel authority.** Only the trusted host may grant filesystem, network, process, registry-publication or credential capabilities. Component metadata is a *request*, not permission.

**INV-02 — No direct untrusted UI code.** Component packages express UI intent through a validated abstract interface. The trusted renderer maps that intent to known QML widgets. Arbitrary package-supplied QML is forbidden in v0.1.

**INV-03 — No Operator in the reactive loop.** Component inputs, outputs and state updates are scheduled deterministically without LLM involvement.

**INV-04 — Declared compatibility.** Runtime types, units, semantic identifiers, component versions and side-effect constraints are validated before ports are connected. RDF similarity does not authorise a connection.

**INV-05 — Reproducible restoration.** Room snapshots reference exact component identities, versions and digests; restoration never silently substitutes a different artifact.

**INV-06 — Explicit failure.** Invalid schemas, missing components, denied capabilities, cycles and execution failures produce typed, inspectable errors.

**INV-07 — Agent separation.** Agents propose declarative plans; the kernel validates and applies them. Agents do not obtain implicit privileges by virtue of their role.

**INV-08 — Open portability.** Domain components MUST NOT require Qt-specific code or Qt object types.

## 3. Logical architecture and trust boundaries

```text
+---------------------------------------------------------------+
| Qt/QML desktop application (trusted UI host)                  |
|  Left: pinned rooms/apps | Centre: Operator | Right: canvas   |
|  Abstract UI renderer <--> Reactive state projection          |
+--------------------------|------------------------------------+
                           | validated host API
+--------------------------v------------------------------------+
| Rust trusted kernel                                            |
| Lifecycle | WASM host | capability broker | plan validator     |
| typed routing | scheduler | artifact verifier | storage facade |
+----------+---------------------+-------------------+----------+
           |                     |                   |
     isolated WASM          metadata index      persistence
     component instances    RDF/JSON Schema     room/events/graph
           |                     |                   |
     calculation/tool       registry facade      local backing stores
     agent adapters         local v0.1           (candidate: SQLite +
                                                  embedded RDF store)
```

### 3.1 Trusted native computing base

The trusted base consists of the Rust kernel, the Qt/QML host renderer, capability-specific native adapters, artifact verification code, and database integrations. They require privileged security review and cannot be replaced by a component without a subsequent explicit architecture revision.

### 3.2 Component plane

Ordinary capabilities (calculation, transformation, notebook logic, agent adapters, future code/build/test tools) SHOULD be components. Individual components interact with other instances through host-mediated ports, not through shared process memory or unmediated host calls.

### 3.3 Agent plane

Operator receives user intent, queries available capabilities and proposes a composition plan. A UI-planning agent MAY propose abstract layouts. Neither agent is part of the scheduling loop. Provider adapters allow local or remote model invocation through separately authorised access.

### 3.4 Separation of responsibilities

| Subsystem | Owns | Must not own |
|---|---|---|
| `kernel` | Host capability authority, lifecycle, validation, scheduling | Natural-language interpretation |
| `wasm-host` | Component invocation and resource limits | User-facing presentation |
| `bus` | Typed message routing and correlation | Semantic inference |
| `semantic` | Manifest validation, RDF index, explicit compatibility rules | Permission grants |
| `reactive` | Data dependencies, dirty propagation, current values | LLM-driven state updates |
| `storage` | Room/event persistence and recovery | Execution policy decisions |
| `ui-host` | Qt rendering and user confirmation | Arbitrary WASM access to native APIs |
| `operator` | Intent, plan proposal, explanations | Direct privileged execution |

## 4. Repository structure and project contracts

Proposed Rust-first monorepo:

```text
workbench/
  Cargo.toml
  crates/
    wb-contracts/        # versioned message and plan DTOs
    wb-kernel/           # instance lifecycle and orchestration
    wb-wasm-host/        # WASM adapter, resource constraints
    wb-bus/              # port routing, envelope and events
    wb-capabilities/     # capability authorization/brokering
    wb-semantic/         # RDF/JSON Schema registry interface
    wb-reactive/         # typed state + dependency scheduler
    wb-storage/          # persistence facade
    wb-registry/         # local bundle resolver and verifier
    wb-testkit/          # fixtures and conformance tests
  apps/desktop-qt/       # native shell and QML components
  components/
    mortgage-calc/       # reference WASM component
    percentage-calc/     # eventual self-production exercise
  schemas/
    manifest.schema.json
    composition.schema.json
    ui.schema.json
    room.schema.json
  ontology/workbench.ttl
  fixtures/
    valid/
    invalid/
  docs/adr/
  tests/e2e/
```

**REQUIRED:** Each crate provides its own unit tests, a public API README, contract-version declaration and test fixtures where appropriate. Automated coding agents MUST respect crate ownership and change shared contracts only through reviewed ADRs.

## 5. Component package and manifest interface

### 5.1 Package shape

A component bundle is a directory in development and a deterministic archive for publication. Its logical contents are:

```text
mortgage-calc.wbc/
  component.json       # required, authoritative package manifest
  logic.wasm           # required for executable components
  capabilities.ttl     # required semantic description
  ui.json              # optional abstract presentation
  schemas/input.json   # optional per-port schema
  schemas/output.json
  evidence/tests.json  # optional verification evidence
```

**REQUIRED:** Canonical content digests identify each distributable payload. Registry installation verifies hashes *before* code is loaded. Signatures and remote provenance are deferred to the registry increment; the local registry MUST retain digest and origin information from day one.

### 5.2 Minimal manifest (proposed canonical JSON)

```json
{
  "manifestVersion": "0.1",
  "id": "org.workbench.example.mortgage-calc",
  "version": "0.1.0",
  "runtime": {
    "kind": "wasm-component",
    "interface": "org.workbench.component@0.1",
    "module": "logic.wasm",
    "sha256": "<64-lowercase-hex-digest>"
  },
  "metadata": "capabilities.ttl",
  "ui": "ui.json",
  "permissions": [],
  "ports": {
    "inputs": [{
      "name": "loan",
      "schema": "schemas/input.json",
      "semanticType": "https://example.org/wb#MortgageTerms"
    }],
    "outputs": [{
      "name": "payment",
      "schema": "schemas/output.json",
      "semanticType": "https://example.org/wb#MortgagePayment"
    }]
  }
}
```

Identifiers and ontology URIs shown are **illustrative** until an ADR assigns a stable public namespace. Component references use `{id, version, sha256}` and never just a mutable registry label.

### 5.3 Port types

Each port has a stable name, direction, machine-readable schema, semantic type and optional units/precision constraints. V0.1 uses JSON Schema 2020-12 for **data model validation**; JSON is the canonical fixture representation. The internal transport encoding may differ.

V0.1 compatible edge rule: an edge is valid iff both ports exist; their structural contracts are compatible or an explicit tested conversion component is inserted; their semantic meanings and units are compatible; the destination permissions and version constraints are satisfied; and the resulting execution graph is schedulable.

### 5.4 Module interface choice

**CANDIDATE ADR-001:** Prefer the WebAssembly Component Model with a versioned WIT interface and Wasmtime host bindings. The exact Wasmtime, WASI and WIT revision MUST be pinned by a compatibility spike. Avoid claiming cross-runtime portability without a second-runtime conformance test.

**Proposed WIT sketch — non-final until ADR-001:**

```wit
package workbench:component@0.1.0;

interface computation {
  record message {
    port: string,
    payload-json: string,
  }
  record component-error {
    code: string,
    message: string,
  }
  compute: func(input: message) -> result<list<message>, component-error>;
}

world compute-component {
  export computation;
}
```

This is intentionally a pure, stateless entry point for the first mortgage slice. Stateful components, streaming, host capabilities and asynchronous service calls require follow-on interface versions; they MUST NOT be smuggled into ad hoc JSON fields.

## 6. Message bus contract

### 6.1 Canonical event envelope

```json
{
  "protocolVersion": "0.1",
  "messageId": "0189e5cb-0e59-7b20-b5fb-c3ff12345678",
  "roomId": "room-local-001",
  "source": {"instanceId": "mortgage-1", "port": "payment"},
  "destination": {"instanceId": "canvas-1", "port": "monthlyPayment"},
  "correlationId": "0189e5cb-0e59-7b20-b5fb-c3ff12345678",
  "sequence": 12,
  "schemaId": "org.workbench.example.mortgage-payment@0.1",
  "payload": {"monthlyPayment": "1432.25", "currency": "GBP"}
}
```

For the initial implementation, **REQUIRED**: process-local routing, per-source deterministic ordering, monotonic room-local sequence assignment, bounded queues, typed validation and structured error outcomes. Exactly-once *effect execution* and distributed delivery guarantees are explicitly out of scope.

### 6.2 Errors and failure events

Shared error codes:

| Code | Meaning |
|---|---|
| `INVALID_MANIFEST` | Bundle manifest fails validation |
| `INCOMPATIBLE_PORTS` | Edge contracts cannot be safely connected |
| `SCHEMA_VIOLATION` | Message does not satisfy port schema |
| `CAPABILITY_DENIED` | Host refuses resource request |
| `COMPONENT_NOT_FOUND` | Pinned artifact unavailable |
| `COMPONENT_TRAP` | WASM invocation traps or exceeds limits |
| `CYCLIC_DEPENDENCY` | Composition contains disallowed cycle |
| `STATE_CONFLICT` | State revision changed during update |
| `RESTORE_FAILED` | Stored room cannot be safely reinstantiated |
| `UNSUPPORTED_VERSION` | Protocol or ABI version is unsupported |

Errors MUST include a stable code, human-readable message, source subsystem, correlation ID where available and machine-readable detail; logs MUST redact credentials.

### 6.3 Bus encoding

**CANDIDATE ADR-002:** JSON payloads and Rust structs for the first process-local implementation, with Protobuf or FlatBuffers adopted only after benchmarked need. The internal encoding is not part of the public component semantics. No assumption is made that serialisation and execution overhead are negligible.

## 7. Composition plan and validation interface

Agents output plans; they do not directly mutate the runtime topology.

```json
{
  "planVersion": "0.1",
  "roomId": "room-local-001",
  "expectedRoomRevision": 3,
  "instances": [{
    "instanceId": "mortgage-1",
    "component": {
      "id": "org.workbench.example.mortgage-calc",
      "version": "0.1.0",
      "sha256": "<verified-digest>"
    }
  }],
  "connections": [{
    "from": {"instanceId": "form-1", "port": "loan"},
    "to": {"instanceId": "mortgage-1", "port": "loan"}
  }],
  "ui": {"definitionRef": "org.workbench.example.mortgage-calc/ui.json"},
  "requestedPermissions": []
}
```

This partial example assumes `form-1` exists in the room; a full plan MUST reference only existing or newly declared instances.

**Plan application MUST:** validate JSON, resolve immutable component versions, validate port/semantic compatibility, detect cycles, check capabilities, check room revision, reserve resources, and commit an atomic topology revision or leave the previous topology untouched. Any operation needing human approval is held *before* commit. Agent explanations are displayed separately from executable plan data.

## 8. Reactive state and scheduler

### 8.1 State addressing

State paths use `{roomId, instanceId, portOrProperty}`. Each room maintains an incrementing `revision`. Runtime input updates are validated against the destination schema and applied as a room revision. Only descendants of changed ports are invalidated.

### 8.2 Scheduling

**REQUIRED (v0.1):** Directed acyclic compute graphs. A topological scheduler ensures consistent order. Each computation receives a coherent input snapshot. Repeated rapid changes MAY coalesce; stale asynchronous outputs MUST not overwrite newer input revisions. Feedback loops require a future, explicitly specified delay/state component.

### 8.3 Decimal arithmetic

Reference financial components MUST use decimal or integer minor-unit arithmetic with documented rounding policy, not unqualified floating-point assumptions. A mortgage example is a behavioural reference, not financial advice.

### 8.4 Operator wake-up rules

The Operator is invoked only on explicit user instruction, requested topology change, or an escalation event routed through a distinct policy. It MUST NOT be invoked on ordinary slider updates, validations or graph recomputation.

## 9. Abstract UI description and Qt binding

### 9.1 UI definition example

```json
{
  "uiVersion": "0.1",
  "root": {
    "type": "form",
    "title": "Mortgage calculator",
    "children": [
      {
        "type": "decimal-input",
        "id": "principal",
        "label": "Principal",
        "unit": "GBP",
        "bind": "mortgage-1.loan.principal"
      },
      {
        "type": "decimal-input",
        "id": "annualRate",
        "label": "Annual interest rate",
        "unit": "percent",
        "bind": "mortgage-1.loan.annualRate"
      },
      {
        "type": "decimal-input",
        "id": "termMonths",
        "label": "Term in months",
        "bind": "mortgage-1.loan.termMonths"
      },
      {
        "type": "readout",
        "id": "payment",
        "label": "Monthly repayment",
        "bind": "mortgage-1.payment.monthlyPayment"
      }
    ]
  }
}
```

The names and bindings above are example schema content, to be formalised in `schemas/ui.schema.json`. `bind` is a declarative path, never executable QML or JavaScript.

### 9.2 UI control vocabulary

V0.1 minimum: `form`, `column`, `row`, `text`, `decimal-input`, `integer-input`, `selector`, `button`, `readout`, `table`, and `error-display`. Charts, notebooks and richer responsive layout are later increments.

The renderer MUST reject unsupported controls, unknown bindings, unsafe expressions and invalid widget attributes. Labels, keyboard focus and accessible names MUST be preserved. Theme and display scaling are host responsibilities.

### 9.3 Qt bridge

**CANDIDATE ADR-003:** Rust kernel communicates with the Qt host through a narrow, explicitly versioned FFI or message boundary. Neither raw Rust pointers nor Qt objects may cross WASM component interfaces. Qt signals/slots can update the state projection, but persistent state remains under the Rust state service.

## 10. Semantic ontology and component discovery

### 10.1 RDF model

The v0.1 ontology SHOULD include: `Component`, `Capability`, `InputPort`, `OutputPort`, `SemanticType`, `PermissionRequest`, `AbstractInterface`, `AssuranceClaim` and `Artifact`. Predicates describe identity, ports, typed units, side effects, UI references and provenance.

Illustrative Turtle:

```turtle
@prefix wb: <https://example.org/wb#> .
@prefix ex: <https://example.org/components/> .

ex:mortgage-calc a wb:Component ;
  wb:providesCapability wb:CalculateMortgagePayment ;
  wb:hasInputType wb:MortgageTerms ;
  wb:hasOutputType wb:MortgagePayment ;
  wb:requiresPermission wb:NoExternalPermissions .
```

The URIs are placeholders, not assigned production identifiers. The ontology SHOULD be published as versioned RDF with examples and validation shapes; SHACL is a candidate for graph-structure validation.

### 10.2 Discovery rules

V0.1 search indexes explicit capability identifiers, tags and ports. The selector returns candidates with reasons and suitability constraints. Automatic inference across arbitrary RDF classes is deferred. Discovery results are advisory; the plan validator remains authoritative.

## 11. Permissions and security contracts

### 11.1 Capability request

```json
{
  "componentInstance": "repo-reader-1",
  "capability": "filesystem.read",
  "scope": {"paths": ["/home/user/projects/workbench/**"]},
  "reason": "Inspect project source files",
  "duration": "session"
}
```

This is a later development-component example; the mortgage component requests no privileges.

### 11.2 Grant semantics

The host evaluates a request against policy and explicit user approval where required, and issues a non-forgeable runtime grant scoped to instance, resource, operation and expiration. Grants cannot be transferred through component output messages or embedded in persisted documents as valid credentials. On restore, grants are re-evaluated against host policy.

### 11.3 Isolation

WASM modules run with memory, fuel/time and resource limits. Host APIs MUST implement policy checks, argument bounds and audit logging. WASM sandboxing limits an attack surface; it does not prove computational correctness or eliminate host bugs.

### 11.4 Development components

Code/build/test/repository components are allowed only through narrow host-backed capabilities, isolated working directories, scoped credentials and human review for protected changes. A development agent cannot directly deploy or replace the running kernel in v0.1.

## 12. Workspace persistence and recovery

### 12.1 Logical room manifest

```json
{
  "roomFormatVersion": "0.1",
  "roomId": "room-local-001",
  "revision": 4,
  "name": "Mortgage scenarios",
  "components": [{
    "instanceId": "mortgage-1",
    "id": "org.workbench.example.mortgage-calc",
    "version": "0.1.0",
    "sha256": "<verified-digest>"
  }],
  "connections": [],
  "state": {
    "mortgage-1.loan.principal": "250000.00",
    "mortgage-1.loan.annualRate": "4.5",
    "mortgage-1.loan.termMonths": 300
  },
  "ui": {"definitionRef": "org.workbench.example.mortgage-calc/ui.json"}
}
```

This schema is a *logical interchange format*, not an implementation requirement to store all room state in one JSON record. Event history, pinned application metadata and chat transcripts are linked records in the storage abstraction.

### 12.2 Storage interface

```rust
// Illustrative API; final signatures require ADR review.
pub trait RoomStore {
    fn read_snapshot(&self, id: &RoomId) -> Result<RoomSnapshot, StoreError>;
    fn commit_revision(
        &self,
        id: &RoomId,
        expected_revision: u64,
        change: RoomChange,
    ) -> Result<u64, StoreError>;
    fn list_rooms(&self) -> Result<Vec<RoomSummary>, StoreError>;
}
```

### 12.3 Initial backing stores

**CANDIDATE ADR-004:** SQLite holds transactional room manifests, typed state and event records; an embedded RDF index supports semantic discovery. The first implementation SHOULD use one durable source of truth and rebuildable semantic indexes to avoid a brittle cross-database distributed transaction. A later integration may use Oxigraph as the RDF engine. Store specifics are intentionally behind `RoomStore` and `SemanticIndex` traits.

### 12.4 Recovery algorithm

On opening a room: (1) read versioned snapshot; (2) validate schema; (3) resolve each pinned digest; (4) validate topology; (5) instantiate components with *no implicit privilege escalation*; (6) restore state; (7) recompute derived values; (8) render the saved layout. If any critical dependency is absent or invalid, mark the room as degraded and offer explicit recovery actions. Do not silently update versions.

### 12.5 Migration

Room format and component-state schema versions MUST be explicit. Migrations MUST be deterministic, independently tested and reversible via saved backup or snapshot. A failed migration leaves the original room recoverable.

## 13. Component lifecycle state machine

```text
discovered -> validated -> installed -> instantiated -> running
                                            |            |
                                            v            v
                                          failed <---- stopped
                                            ^            |
                                            +-- restore/retry
```

Transitions MUST be explicit and audited. Installing a package never automatically grants host permissions. Starting an instance checks current capabilities and resource quotas. A trapped component is stopped or quarantined without crashing unrelated instances; affected output ports enter an error state.

## 14. API boundaries and versioning

| API boundary | Public contract | Versioning rule |
|---|---|---|
| Package | `component.json` + binary digests | `manifestVersion` and semver component version |
| WASM ABI | WIT world / generated bindings | Explicit major/minor supported versions |
| Bus | Message envelope and port schemas | `protocolVersion`, reject unsupported major |
| Composition | Declarative plan | `planVersion` and room revision precondition |
| UI | Validated semantic JSON | `uiVersion`, reject unknown controls by default |
| Semantic ontology | RDF vocabulary | Namespace / ontology release version |
| Persistence | Snapshot and event schema | `roomFormatVersion`, tested migrations |
| Capability broker | Permission request/grant | Named capability version and typed scope |

Breaking a contract requires a new version and conformance tests. Runtime adapters MAY support more than one version, but must not silently reinterpret payload meaning.

## 15. Verification and conformance plan

### 15.1 Mandatory automated tests

| ID | Test | Pass condition |
|---|---|---|
| CT-01 | Independent component compile/load | Two separate Rust/WASM packages load through one host contract |
| CT-02 | Invalid manifest | Rejected before execution with `INVALID_MANIFEST` |
| CT-03 | Typed edge mismatch | Invalid connection rejected with `INCOMPATIBLE_PORTS` |
| CT-04 | Semantic mismatch | Compatible JSON shape but incompatible units is rejected |
| CT-05 | Capability denial | Attempted undeclared file access is denied and audited |
| CT-06 | UI safety | Unknown or executable UI expressions are rejected |
| CT-07 | Reactive mortgage | Changing principal updates payment with no Operator call |
| CT-08 | Rapid updates | Late result from old revision cannot overwrite newer value |
| CT-09 | Cycle rejection | Composition cycle rejected with `CYCLIC_DEPENDENCY` |
| CT-10 | Room round-trip | Save, relaunch, restore exact pinned versions/topology and inputs |
| CT-11 | Missing pinned artifact | Controlled degraded state, no silent substitution |
| CT-12 | Crash recovery | Durable last committed revision survives a forced process crash |
| CT-13 | Isolated WASM trap | Trapped component does not crash host or other instances |
| CT-14 | Permission restoration | Saved room never revives stale capability grants silently |
| CT-15 | Agent plan boundary | Invalid or unapproved proposed plan does not modify room |

### 15.2 Mortgage fixture

Fixture assumes amortising fixed-rate loan, principal GBP 250,000, nominal annual rate 4.5%, 300 monthly payments and monthly rate = annual rate / 12. Expected output is calculated with a fixed decimal specification and stated rounding to pennies. Acceptance checks match the independently implemented reference fixture, including the zero-interest boundary, invalid term and negative principal cases. **The expected numerical results are to be locked after an independent reference calculation and review**, not guessed from an LLM response.

### 15.3 Agent-produced change criteria

For all changes: compilation; formatting/lint; unit tests; contract conformance; integration tests for touched boundaries; reproducible fixture results; and independent review of changed trust-boundary code. Agents MUST NOT modify failing test expectations merely to make a build green without explicit specification change approval.

## 16. Incremental implementation and gate order

| Gate | Engineering deliverable | Acceptance |
|---|---|---|
| G0 | Contracts, schema skeleton, ADRs and test harness | Schema fixtures validate; code owners defined |
| G1 | Standalone WASM host and two test components | CT-01, CT-02, CT-13 |
| G2 | Manifest, semantic index, routing, permissions | CT-03, CT-04, CT-05 |
| G3 | Reactive engine and Qt semantic renderer | CT-06 to CT-09; mortgage UI operational |
| G4 | Durable room store and restore path | CT-10 to CT-12, CT-14 |
| G5 | Operator plan proposal and validation | CT-15; natural-language planning demo |
| B1 | Development tools exposed as components | New component generated, tested and locally installed |
| B2 | Self-development workflow | Tested change to Workbench submitted for human review |

B1 and B2 are target follow-ons, not dependencies of the first mortgage vertical slice.

## 17. Architecture decision record backlog

| ADR | Decision | Required by |
|---|---|---|
| ADR-001 | WASM Component Model, WIT ABI and runtime version | G1 |
| ADR-002 | Message encoding, scheduling and backpressure | G2 |
| ADR-003 | Rust–Qt interface and threading model | G3 |
| ADR-004 | SQLite/RDF persistence consistency and crash recovery | G4 |
| ADR-005 | Stable RDF namespace, SHACL validation and semantic compatibility | G2 |
| ADR-006 | Artifact digest, bundle format and local registry layout | G1/G2 |
| ADR-007 | Permission grant tokens and native resource adapters | G2 |
| ADR-008 | Currency/decimal precision and numeric consistency | G3 |
| ADR-009 | Operator execution provider and structured plan generation | G5 |
| ADR-010 | Agent development sandbox / merge / publishing trust boundary | B1 |

An ADR MUST record: context, proposed decision, alternatives, consequences, evidence, owner and validation plan.

## 18. Recommended first agent work packages

1. **SPEC-001:** Implement machine-checkable JSON schemas for manifest, plans, abstract UI and room snapshots; add positive and negative fixtures.
2. **SPEC-002:** Define `wb-contracts` Rust types and deterministic serializers for all above schemas.
3. **RUNTIME-001:** Prototype pinned WASM/WIT host and two independent pure compute modules; record ADR-001 evidence.
4. **SEC-001:** Implement capability broker that denies all undeclared host access and produces audit events.
5. **GRAPH-001:** Implement local package registry, metadata parser and explicit compatibility verifier.
6. **REACTIVE-001:** Implement DAG scheduler, typed state revisions and stale-response prevention.
7. **UI-001:** Implement semantic form/readout widgets in Qt without executing supplied QML.
8. **STORAGE-001:** Implement durable room snapshot revision transactions and crash recovery.
9. **DEMO-001:** Assemble mortgage component, demonstrate reactive updates and complete save/reopen test.
10. **AGENT-001:** Add Operator plan-output adapter and deterministic plan validation.

Each package requires a fixed contract, bounded repository paths, fixture-driven acceptance tests and an independent integration check. Parallelise only packages that do not modify the same contract surface.

## 19. Deferred design work

Explicitly **not frozen** in v0.1: arbitrary streaming agent-to-agent protocols, remote federated registries, signature trust networks, notebook/document semantics, richer charts, multi-user rooms, graph inference beyond explicit declarations, globally coordinated storage transactions, unrestricted agent-generated UI, selective formal proof tooling and autonomous production deployments. These require later specifications and compatibility strategies.

## 20. Definition of v0.1 architecture completion

The architecture is implementable when: every trust boundary has a documented interface; the ABI and Qt bridge spikes have passed; package, plan, UI and room formats validate against schemas; all normative conformance cases have executable fixtures; and an independent agent can implement the mortgage vertical slice without inventing new public contracts or gaining undescribed host privileges.

**Product principle:** Operator decides *what* should be assembled; the kernel decides *whether and how* it can be safely executed; the reactive runtime executes it; the UI host presents it; persistence makes it durable.
