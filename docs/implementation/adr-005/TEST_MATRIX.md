# ADR-005 — Acceptance test matrix

All gates `OP-001`–`OP-024` are mandatory, executable, non-ignored, non-skipped. Each requires named test(s), command, exit status and evidence artifact; do not substitute a README claim or a mocked runtime for a product-path test.

| ID | Required observable result |
|---|---|
| OP-001 | `OperatorProvider` deterministic fixture and typed request/suggestion DTOs compile and validate strict schemas without remote network. |
| OP-002 | Read-only search lists only installed/admitted local packages; invalid/uninstalled/unsupported packages and UI-less candidates are not offered for the UI demo. |
| OP-003 | Candidate context is bounded and excludes local filesystem paths, secrets, unrelated room data and executable instructions. |
| OP-004 | Typical mortgage request selects installed calculator by opaque catalog key; no hardcoded version, SHA or registry path. |
| OP-005 | Deterministic parameter normalisation maps GBP 250,000; annual 5%; 25 years to actual accepted mortgage input contract, with expected output vector. |
| OP-006 | Ambiguous/missing values return explicit clarification/error; no fabricated rate/term/unit. |
| OP-007 | Malformed JSON, duplicate keys, unknown fields, invalid types and oversized model output are rejected without side effects. |
| OP-008 | Prompt-injection strings from candidate metadata or user content cannot cause tool invocation, unauthorised actions, altered instructions or authority escalation. |
| OP-009 | Forged/unoffered candidate key and pin drift fail, without fallback to a similar version or mutation. |
| OP-010 | Proposed fields and units are checked against actual manifest, admitted UI bindings and input schema; invented, wrong-direction or invalid fields rejected. |
| OP-011 | Host constructs canonical `planVersion: 0.1` plan with correct actual room revision, pin, instance and valid UI ref; accepted plan schema unchanged. |
| OP-012 | A plan with cyclic/invalid topology, incompatible ports, missing resources or unexpected permissions fails in existing validator before application. |
| OP-013 | Preview clearly shows exact selected component, intended input values, effects and approval requirement; model explanation shown as untrusted. |
| OP-014 | Explicit approval is single-use and bound to proposal digest, room revision and expiry; declined, expired or replayed approvals fail. |
| OP-015 | Concurrent/stale room revision or changed pin/grants between preview and apply returns conflict without room mutation. |
| OP-016 | Successful approved proposal is committed only via existing ADR-002 topology API; no direct state/router bypass. |
| OP-017 | Initial values enter through actual host UI/typed ingress, real WASM executes and committed mortgage output matches accepted numerical vector. |
| OP-018 | Input-ingress failure after topology commit is reported as partial workflow completion, with no false rollback or false committed result. |
| OP-019 | Subsequent slider/form edits recalculate through existing UI/IPC/WASM path with **zero additional Operator/provider calls**. |
| OP-020 | Offline restore uses exact pinned UI and committed state without model/provider call, even when network/provider disabled. |
| OP-021 | Cancellation, provider timeout/unavailability and bounded queue saturation return typed errors and do not mutate the room. |
| OP-022 | Two rooms and competing requests cannot leak data, approval or selected pins; audit logs redact sensitive content and record correlation/revision/status. |
| OP-023 | Two independent reviews completed with documented findings/resolutions: (A) agent prompt-injection/privacy/authorisation; (B) registry, plan, lifecycle, revision and input boundary. |
| OP-024 | All OP gates plus ABI-001–010, IPC-001–015, REG-001–020, UI-001–020, VSLICE-001/002 regressions pass in hosted Linux CI. Archive logs, exact versions, commit SHA, generated hashes, approval evidence and run URL. |

## Mandatory demonstrator

Run the real Linux desktop process. Enter the request "I need a mortgage calculator for a £250,000 loan over 25 years at 5%." Use the fixture provider deterministically for CI, enable a separately authorised external provider only as optional smoke testing, verify preview/approval, real installed pin, calculation and later direct reactive edit. Capture component pin, provider invocation count, room revision before/after, observed readout, and exit code. No mortgage-specific special case in Qt or the kernel.
