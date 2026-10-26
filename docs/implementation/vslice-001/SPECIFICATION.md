# VSLICE-001 — Headless Installable Mortgage Calculator

**Status:** Accepted — 2026-10-26; all VSLICE gates, reviews and hosted Linux evidence verified ([results](evidence/RESULTS.md))
**Version:** 0.1 / 2026-10-26  
**Classification:** Product vertical slice, **not** a new ADR or public interface revision.  
**Owners:** `wb-registry`, `wb-semantic`, `wb-wasm-host`, `wb-kernel`, `wb-bus`, `wb-reactive`, `wb-contracts`, verification.

## 1. Objective
Demonstrate **one continuous, evidence-backed pipeline**: compiled, pure WASM Component Model mortgage engine → complete local v0.1 directory package → validated admission → immutable local registry → exact-pin resolution → ADR-002 lifecycle and message routing → ADR-001 WIT invocation → schema-validated output. This is the first product-level cross-system gate, not a prototype that calls a guest directly and labels it integrated.

## 2. Authority and unchanged contracts
- **ADR-001 (Accepted):** `wit/component.wit`, `workbench:component@0.1.0`, synchronous `compute(message) -> result<list<message>, component-error>`, Wasmtime host with no WASI/native guest imports.
- **ADR-002 (Accepted):** host-owned lifecycle; room IDs, pinned instances, monotonic room-local sequence numbers; bounded queues; deterministic DAG; atomic output batches; stale-generation and revision fences; JSON envelope; source-controlled permission decisions.
- **ADR-003 (Accepted):** local directory-only admission, strict canonical manifest / RDF / SHACL / digest / ABI verification, transactional immutable registry, `(id,version,sha256)` resolution; UI-bearing packages and nonempty permission requests remain unsupported in the accepted v0.1 registry scope.
- **Component package v0.1:** `component.json` authoritative; `logic.wasm`, `capabilities.ttl`, local JSON Schema files; any additional fields or runtime interface changes require separate approval.
- **Ontology v0.1:** provisional public namespace; do not rename or expand classes in this task.

Where accepted implementation names differ from this document's illustrative names, **reuse the actual exported APIs** and record the mapping in the evidence. Do not silently change an accepted DTO, WIT, schema, ontology or behavioural contract.

## 3. In scope
1. Deterministic, financially sensible **fixed-rate, fully amortising monthly mortgage payment** engine compiled to a Component Model binary conforming to accepted WIT.
2. One non-UI package with `permissions: []`, an aggregate input port and declared output port, local JSON Schemas, exact artifact hash, matching Turtle and SHACL.
3. A *single host-owned integration path* that takes the installed exact pin, retrieves verified bytes, prepares/starts a logical instance, accepts a schema-valid room input, invokes the guest via the accepted host, commits validated output, and exposes the committed result through the bus/room state.
4. End-to-end CLI/test harness (CI-friendly; no Qt required), positive and negative fixtures, logs, metrics and acceptance evidence.
5. Room revision, ordering, bounded-message handling, safe failure and restoration/pins according to ADR-002, only to the extent applicable without adding persistent guest memory.

## 4. Explicit exclusions
No Qt/QML or dynamic UI; no Operator/AI requests; no live interest-rate lookup; no tax, fees, escrow, variable rates, overpayments or arbitrary amortisation schedule; no storage engine rewrite; no remote registry, semver range solving, signing or installer execution; no guest filesystem/network access; no new ABI or host imports; no production-trust certification. Do not broaden ADR-003 to permit UI packages.

## 5. Reference component shape
Suggested example name: `examples/mortgage-headless/`. Use the **actual** `component.schema.json` and RDF vocabulary, not an invented simplified schema. Logical files:
```
mortgage-headless/
  component.json
  logic.wasm
  capabilities.ttl
  schemas/mortgage-input.schema.json
  schemas/mortgage-output.schema.json
```
The component declares **one aggregate input** (`calculate`) and **one output** (`payment`), avoiding unspecified multi-port atomic snapshot semantics. The input value is encoded in the accepted WIT `message.payload-json`; the guest output uses the WIT message port `payment` and JSON payload. All required fields, manifest permission values, semantic URIs and RDF mappings must follow the actual existing normative schema and ontology. If a required semantic URI does not exist, use a non-conflicting absolute URI for the domain meaning while preserving existing ontology terms and document it; do not assert inference or units the registry cannot validate.

### 5.1 Input contract (reference domain schema)
```json
{
  "principal": "250000.00",
  "annual_rate_percent": "5.00",
  "term_months": 300
}
```
All monetary/rate values are **canonical decimal strings** (base-10, non-negative). `principal` is currency-valued without any exchange-rate semantics; the fixture uses an illustrative monetary unit and is not financial advice. `annual_rate_percent` is a percentage (5.00 means five percent, not 0.05). `term_months` is a positive integer. JSON Schema establishes syntax/ranges and additionalProperties rules; the guest validates domain requirements as a second line of defence.

Recommended v0.1 domain policy (capture exact decisions in implementation):
- principal > 0, ≤ 1,000,000,000; at most two fractional decimal digits;
- annual_rate_percent ≥ 0 and ≤ 100; at most six fractional decimal digits;
- term_months integer 1–600;
- reject NaN, Infinity, negatives, zero principal, zero term, huge exponent strings and noncanonical decimal forms;
- compute using a deterministic decimal library or suitably proven numerical strategy; avoid unchecked f64 monetary accumulation and integer overflow;
- return a **string** `monthly_payment` with exactly two fractional digits, **round-half-up to the nearest minor currency unit** (explicit v0.1 fixture convention); this rounding is a product-specific test policy, not a change to shared numeric/semantic contracts.

### 5.2 Payment mathematics
For principal P, periodic monthly rate r = annual_rate_percent / 1200, and number of months n:
- if r = 0: A = P / n
- otherwise: A = P × r / (1 − (1 + r)^(-n)).
Ensure the arithmetic strategy is robust for small nonzero r and large n, and round **only the published payment result**. The function is a calculation utility, not loan advice or APR modelling. Encode/parse decimal strings in a manner independent of local OS locale/timezone.

### 5.3 Output and errors
Successful output reference:
```json
{ "monthly_payment": "1461.48" }
```
The guest must produce the **declared output port only**, with a schema-valid JSON payload; each guest invocation uses the single accepted WIT message. Invalid business input returns a typed WIT `component-error` with stable domain codes where possible (e.g., `INVALID_MORTGAGE_INPUT`), without panicking. Host schema failures (input or output) must be distinguishable from guest domain errors and runtime traps; no raw secrets or machine paths may appear in user-facing errors. No partial result is committed on error.

## 6. Normative cross-subsystem execution sequence
1. Build compiled **Component Model** binary with the accepted WIT. Hash raw bytes; populate manifest digest and reconciled RDF metadata.
2. **Admit** directory with `wb-registry`: path-safe staged validation, schemas/RDF/SHACL, digest, ABI, transaction commit; do not execute guest functions during admission/discovery.
3. **Resolve exact pin** `(id, version, SHA-256)` and obtain an opaque validated registry handle; read verified bytes at the registry-to-host boundary.
4. **Prepare** a room-scoped logical instance with the immutable pin using accepted ADR-002 lifecycle and no implicit permission grants; then transition to `running` through allowed host-controlled transitions. Reject missing/different pins.
5. **Accept room input** through the host-owned ingress/bus with canonical envelope and checked schema, idempotence/sequence expectations and bounded queuing. Guest cannot specify room/destination or forge canonical metadata.
6. **Schedule compute** using ADR-002 deterministic DAG rules; freeze input snapshot with room revision and execution generation, invoke ADR-001 host with accepted WIT; guest has zero WASI/native service imports.
7. **Validate entire guest output batch** for declared port/schema, size and counts before commit; commit only if current revision/generation still match. Publish committed result via host bus/projection, not directly from guest memory.
8. **Record correlation and evidence** across registry pin → instance ID → invocation ID → room revision → emitted canonical message. A failure must not mutate prior successful output or commit partial batches.

## 7. Authority and integrity boundaries
- Registry digest = identity/integrity, **not authorship**.
- Admission is **not execution**; discovery and installation cannot call guest `compute` or initializers.
- Registry handle is **not a permission grant**; host validates permissions at prepare/start (none requested by fixture).
- Only host creates message IDs, provenance, destination and room sequences; guest has access solely to `port` and `payload-json`.
- Do not trust JSON Schemas bundled with unadmitted packages or allow remote `$ref`.
- Artifact changes between resolution and execution must be detected (or prevented by documented immutable verified handle semantics).
- No stdout-only demonstration may substitute for checking committed bus/room state.

## 8. State, restore and concurrency
- One logical component instance has a room-scoped ID and an exact package pin.
- Pure computation is stateless between invocations; retained state is host-owned.
- Repeat identical input within the same accepted state should yield identical response bytes/values subject to the published decimal and canonical JSON policy.
- A completed result may only commit if it matches the room's current revision and instance execution generation. Cancel/restart and topology changes fence stale completions.
- Committed state, instance pin and routing projection must survive snapshot/restore where ADR-002 guarantees them; restore must not silently fetch/upgrade a missing or mismatched package.
- Test interleaved rooms and backpressure with bounded queues. Do not assume physical time order across rooms.

## 9. Deliverables
- Production-facing glue code in existing crates, not a bypass-only demo service.
- Real compiled WASM guest source and locked build metadata.
- Complete package creation step or deterministic instructions that compute matching digest/RDF; **not** a bogus hand-filled hash.
- CLI/integration harness that exercises accepted registry, kernel, bus and Wasmtime interfaces end-to-end.
- `VSLICE-001` through `VSLICE-020` tests, fixtures and evidence; separate independent reviews; hosted Linux CI.
- Updated backlog/status and only necessary documentation of any actual API mappings or deviations.

## 10. Definition of done
All 20 named gates pass, no skip/ignored/mocked substitutes for mandatory integration gates, all existing ABI-001–010 / IPC-001–015 / REG-001–020 continue to pass, Clippy and formatting clean, package digest recorded, independent reviewers sign off, and successful GitHub-hosted Linux run with link and archived evidence. Status remains **PROPOSED** until all are met and a separate acceptance note is recorded. A passing local run, green workflow definition or reviewer opinion alone is insufficient.

## 11. Change control
Any requirement forcing ABI, manifest, ontology, registry scope, accepted lifecycle semantics or permissions changes is a *blocking architecture issue*, not permission to improvise. Record it and seek a narrowly reviewed follow-up decision; don't quietly modify accepted contracts. The UI demo remains a subsequent vertical slice.
