# VSLICE-004 acceptance matrix

**No skipped mandatory tests.** VS4-001–026 are deterministic local/hosted executable checks; VS4-027–029 are review/hosted evidence gates; VS4-030 is a separate supervised real-provider gate. Each entry must link a named executable test or recorded manual process with evidence, expected outcome and pass/fail result. Do not invent passing tests.

| Gate | Required result | Profile |
|---|---|---|
| VS4-001 | Fresh Linux desktop launches with actual installed mortgage package using documented instructions. | CI |
| VS4-002 | Offline default and explicit OpenRouter selection; absent key produces no network call. | CI |
| VS4-003 | Credential entry is host-controlled, redacted, never persisted into rooms or logs. | CI |
| VS4-004 | Remote data disclosure preview contains only explicit request and bounded admitted catalog. | CI |
| VS4-005 | Consent withheld, denied or withdrawn prevents outbound request. | CI |
| VS4-006 | Consent and request state correctly exposed to keyboard and assistive UI. | CI |
| VS4-007 | Real desktop request traverses actual OpenRouter adapter via injected mock transport, not fixture-provider shortcut. | CI |
| VS4-008 | Bounded progress, cancellation and status UI behave under delayed response. | CI |
| VS4-009 | Strict valid suggestion reconciled with live local registry exact pin and schemas. | CI |
| VS4-010 | Forged candidate, unknown port, duplicate JSON key and invalid values fail closed, without room mutation. | CI |
| VS4-011 | Prompt-injection text in catalog/provider response cannot create actions or request privileges. | CI |
| VS4-012 | Preview displays accurate component, version/pin, fields/units, room, permissions and effects. | CI |
| VS4-013 | Remote consent alone never applies plan; separate genuine Qt approval is mandatory. | CI |
| VS4-014 | Reject, expiry, replay and double-click do not mutate/duplicate installed topology. | CI |
| VS4-015 | Revision/pin changes between preview and confirm invalidate proposal. | CI |
| VS4-016 | Approved canonical plan uses real ADR-002 kernel and real admitted WASM. | CI |
| VS4-017 | Topology commit versus typed initial ingress are separate; partial failure surfaced and safely recoverable. | CI |
| VS4-018 | Reference output is GBP 1461.48 from validated real WASM execution, not Qt/model arithmetic. | CI |
| VS4-019 | Edit principal to GBP 300000; reactive output GBP 1753.77; zero additional provider calls. | CI |
| VS4-020 | Rapid edits/reordering and cancellation fences prevent stale outputs and invalid projections. | CI |
| VS4-021 | Model refusal, rate limits, timeout, malformed response, network loss leave last committed room intact and show actionable status. | CI |
| VS4-022 | Mid-flight provider switch or room switch fences late responses; second room remains isolated. | CI |
| VS4-023 | Pin and restart in separate OS process offline; exact pin/inputs/layout/committed result restored with zero provider/guest calls for painting. | CI |
| VS4-024 | Offline mode still works after live mode disabled; no hidden network dependency in reactive editing. | CI |
| VS4-025 | Actual Qt keyboard, focus, accessible names/status and consent/approval separation exercised. | CI |
| VS4-026 | Telemetry, errors, snapshots and evidence are secret-safe and privacy-bounded; negative scan fixtures. | CI |
| VS4-027 | Independent privacy/security reviewer approves consent, secrets, egress, untrusted outputs and permission boundaries with recorded retests. | Review |
| VS4-028 | Independent desktop/runtime reviewer approves Qt usability, kernel integration, partial recovery and cold restoration with recorded retests. | Review |
| VS4-029 | GitHub-hosted Linux CI success AFTER final source changes; all prior ABI/IPC/REG/UI/OP/LLM/VS1–3 gates preserved; logs/hashes verified. | Hosted |
| VS4-030 | Separately supervised real OpenRouter desktop walkthrough: user consent, live response, preview, separate approval, real WASM calculation, reactive edit without extra inference, offline restart, sanitized model/usage/latency evidence. | Live |

## Evidence for each gate
Record test identifier/name, command or manual steps, expectation, observed result, repository commit, proof location, and classification. A green summary without individual evidence is not sufficient for acceptance. The live acceptance must state which model was actually used and is not evidence of other providers/models.
