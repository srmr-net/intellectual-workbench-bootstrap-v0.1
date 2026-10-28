# VSLICE-003 — Operator-Driven Component Discovery and Assembly

**Status:** ACCEPTED (Linux deterministic offline Operator profile) — 2026-10-28; [verified evidence](evidence/RESULTS.md)
**Version:** 0.1 · 2026-10-27  
**Target:** Linux desktop / deterministic offline Operator profile.  
**Prerequisites:** Accepted ADR-001–005, VSLICE-001–002. Accepted contracts are normative and MUST NOT be modified to pass this slice.

## 1. Purpose and scope

Prove the complete *user-facing* interaction envisioned in the product plan. A person types a natural-language request in the centre column; the bounded Operator searches admitted local components, returns a candidate and parameters, the trusted host constructs and validates a canonical plan, the UI previews exact effects, a user approves, and the existing kernel applies and runs a real WASM component on the right canvas. The component can be pinned, interacted with and restored offline without model re-inference.

This milestone is NOT a new planning architecture. ADR-005 already establishes proposal structure, authority, security and provider abstraction. This slice integrates and evaluates that architecture in an actual desktop journey.

## 2. Reference user journey (mandatory)

1. Launch the real Qt Workbench Linux desktop process, with a pristine private test user-data directory and locally installed mortgage component. The test uses real accepted registry admission and package data, not a hardcoded component path.
2. User types: **“I need a mortgage calculator for a £250,000 loan over 25 years at 5%.”** into the Operator input in the centre column. The UI shows request state, bounded progress, and either a preview or a typed failure.
3. Operator discovers a **currently admitted** installed candidate by read-only metadata lookup. The deterministic fixture provider selects it by the offered opaque key; fixture tests must not assume a particular version or SHA-256.
4. Trusted code normalises the input using explicit, tested unit and field maps to the actual immutable mortgage input schema; verify GBP, annual percentage, term years, and no invented parameters.
5. Preview displays component identity/version/digest (digest may be truncated visually but full pin must be available), input values with units, affected room, expected effect, approval requirement and any requested capabilities. Text emitted by a model is marked untrusted. No room mutations occur at preview.
6. User confirms via a genuine Qt action (not a test-only direct `apply`). The approval is bound to exact proposal/room revision/expiry and is single use. The host re-resolves pin/grants, revalidates canonical `planVersion: "0.1"` plan and applies through accepted kernel API.
7. After the topology is committed, initialise inputs using the **existing typed ingress**. Actual Wasmtime executes the installed WASM component; validated output is committed by the room bus and displayed through the trusted semantic UI. The reference monthly payment is **GBP 1461.48** (excludes fees). Never compute the result in QML/Operator.
8. Change loan principal through the rendered form. The host performs real reactive compute and updates the readout. Record that the provider invocation count does not change after initial assembly.
9. Pin the room, close the desktop cleanly, restart in a **new OS process**, and show exact installed pin, UI, inputs and last committed output. Restoration must work with provider/network disabled and must not execute a guest merely to paint committed state.
10. Repeat in a second room to prove isolation, including cancellation or rejection scenarios.

## 3. Boundaries and ownership

| Layer | Responsibilities | Prohibited shortcuts |
|---|---|---|
| Qt desktop | Text input, accessible progress, preview, confirm/reject, selected-room canvas, state/error presentation | Mortgage-specific QML branches, model-supplied QML, direct kernel mutation |
| Operator adapter | Explicit request → bounded provider suggestion using read-only catalog | Direct install, unrestricted tools, file/network access without host configuration |
| Host planning/approval | Reconcile offered candidate, exact pin, schemas, units, permissions, revision; create canonical plan, single-use approval | Accept model-created executable plan or approval token |
| Registry | Resolve admitted immutable local package + semantic metadata + UI | Remote fetch or unverified substitution |
| Kernel/bus | Commit validated topology, typed input ingress, failure handling, revision fences | Silent auto-retry on conflict, non-atomic output commit |
| Qt renderer | Render validated abstract UI + committed projections | Evaluate package-authored QML or JavaScript |
| Persistence | Restore pins, room UI, committed inputs/outputs offline | Infer missing state through model; silently replace pinned component |

## 4. Contracts

- Retain accepted ADR-001 WIT, component manifest and RDF/SHACL, `planVersion: "0.1"`, UI JSON, operator suggestion/preview and room snapshot formats.
- No extra input fields smuggled into topology plan: topology apply and typed ingress are **distinct steps**; after a successful topology commit, ingress failure is a **partial workflow**, not a magically rolled-back transaction.
- Operator/provider output is untrusted. Strict parsing, bounded catalog, candidate-key validation, authorisation and exact-pin revalidation continue to apply.
- Store no model secrets in room snapshots. Use correlation IDs and structured statuses; avoid logging raw prompts by default.
- User approval defaults to explicit for this product demonstration, including zero-permission local plans.
- Show clarification on missing principal, rate, term or ambiguous unit; do not guess.
- Provide deterministic fixture-provider CI using no network, no external API token and a documented dependency and build set. A real model adapter test MAY be run separately, but is NOT a mandatory gate and MUST NOT be confused with offline-profile acceptance.

## 5. UX requirements

Three-column desktop layout from VSLICE-002 stays intact. Operator entry, submitted user message, status, exact proposed component and values, clearly labelled Confirm and Reject actions, and visible errors must be keyboard accessible and testable in actual Qt. Confirm remains disabled while validation is pending or failed. Rejection leaves topology/state unchanged. Timeout, cancellation, stale preview, missing component and ambiguous intent must produce actionable, non-deceptive errors. Preserve last good committed canvas on failure. Subsequent edits and restore must not call the provider.

## 6. Transaction and failure semantics

- Prepare is read-only; record room revision and offered candidate set. Applying with changed room revision/pin, expired/replayed approval, cancelled request or changed privilege set fails closed without topology changes.
- Successful plan commit is observable as a monotonic room revision. Input ingress afterward is a separate validated operation. If ingress fails, surface the partial state and offer a safe explicit retry/recovery; do not claim the entire request rolled back.
- Rejected provider outputs, malicious catalog strings, forged candidate keys, malformed JSON, unexpected output fields and missing tools must not mutate the room or leak secrets.
- Guest calculation failures, queue saturation and stale projection delivery must be handled by existing kernel fences and Qt failure indications.
- A repeat request must not silently duplicate component instances or apply twice because of a double click; enforce single-use approvals/idempotence policy and test it.

## 7. Deferred

Multi-component assembly, automatic code generation, remote registries, live-network provider as a shipping requirement, multi-agent orchestration, unprompted changes, permanent AI memory, cross-platform packaging, power-loss database recovery, and broader semantic inference. This slice does not accept those features by implication.

## 8. Exit gate

`VS3-001`–`VS3-026` executable and passing locally **and in GitHub-hosted Linux CI**, accepted regression gates passing, independent agent/security and Qt/runtime reviews approved, evidence hashes and CI URLs archived. Only then mark **ACCEPTED (Linux deterministic offline Operator profile)**. Separately report any optional external-model trial, including cost, permissions, quality and failures; it does not substitute for deterministic gates.
