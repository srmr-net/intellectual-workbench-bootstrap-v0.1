# VSLICE-001 — Product-level Conformance Matrix

**Acceptance rule:** every gate is mandatory for this v0.1 milestone. Implement as automated non-ignored tests where possible. Record test function/fixture name, command, outcome, host and CI log location per gate. If a gate requires a separate adversarial component, build it from pinned source. **Do not accept a mock-based substitute for the core registry→kernel→Wasm→bus path.**

| ID | Scenario | Required assertion |
|---|---|---|
| VSLICE-001 | Build valid headless mortgage guest | Compiled Component Model binary conforms to accepted WIT world; no WASI/native imports; locked build recorded. |
| VSLICE-002 | Package admission | Complete dir manifest/schema/Turtle/digest/ABI pass actual ADR-003 admission, producing committed receipt. |
| VSLICE-003 | Exact pin and integrity | `(id,version,sha256)` resolves to verified artifact; bad digest pin fails, no fallback. |
| VSLICE-004 | Lifecycle integration | Prepare/start/stop real registered instance through ADR-002 kernel; invalid transitions fail with no guest execution. |
| VSLICE-005 | Full calculation path | User input enters bus/room, triggers real WIT call, output commits into room projection with host-assigned provenance/correlation. |
| VSLICE-006 | Numerical reference vectors | Validate all `fixtures/mortgage-vectors.json` positive cases, 0% rate and rounding boundaries using an independently implemented reference. |
| VSLICE-007 | Input schema/domain failures | Wrong JSON shape/type, extra keys, zero principal, zero/too-long term, negative/out-of-range rate, malformed decimal rejected without state mutation. |
| VSLICE-008 | Output validation/atomicity | Adversarial guest returning undeclared output, malformed JSON or invalid schema fails whole batch; previous committed state unchanged. |
| VSLICE-009 | Manifest and RDF tampering | Hash, schema mapping, semantic port mismatch, or ABI mismatch rejected at admission; no partial registration. |
| VSLICE-010 | No guest invocation on discovery | Registry discover/admit/resolve cannot invoke guest `compute`; inspected imports and test instrumentation establish non-execution. |
| VSLICE-011 | Capability isolation | Pure guest works with empty grants; guest requiring WASI or native imports rejected; no capability escalation via metadata. |
| VSLICE-012 | Correlation and ordering | Multiple accepted changes have monotonic room-local sequence, correctly attributed source, correlation and deterministic resultant order. |
| VSLICE-013 | Stale result fence | Cancel/restart or newer room revision prevents late result from committing; old projection not overwritten. |
| VSLICE-014 | Bounded queues | Saturated ingress reports typed `QUEUE_FULL` (or accepted equivalent); committed changes never silently dropped. |
| VSLICE-015 | Multi-room isolation | Two rooms with same component pin have independent state, sequences and outputs; no cross-room delivery. |
| VSLICE-016 | Snapshot/restore and pin | Restored room retains exact pin and committed result/topology; missing/tampered pin fails closed without upgrade. |
| VSLICE-017 | Deterministic rebuild/invocation | Clean local guest rebuild digest comparison documented (not assumed cross-machine); repeated inputs yield stable computed output. |
| VSLICE-018 | Failure and recovery | Trap, budget limit and interrupted installation do not partially mutate registry/room; subsequent healthy invocation succeeds. |
| VSLICE-019 | Independent reviews | Financial/schema and security/integration reviews record reviewer identities, findings, remediation and sign-off; no open mandatory blockers. |
| VSLICE-020 | Hosted CI and regressions | All ABI-001–010, IPC-001–015, REG-001–020, full workspace tests, Clippy, formatting, contracts and VSLICE gates green on GitHub-hosted Linux; logs and verified hashes archived. |

## Suggested acceptance example
```text
Given a valid locally packaged mortgage WASM component and matching Turtle,
When the registry admits the package and the kernel resolves its exact pin,
And a host-authorized room input requests P=250000.00, rate=5.00%, n=300,
Then the room's result projection contains monthly_payment="1461.48",
And the result is traceable through accepted source, sequence, instance, invocation and pin,
And guest code was called via the accepted ADR-001 WIT ABI.
```

## Evidence matrix template columns
`Gate ID | Test symbol | Command | Local PASS/FAIL | Hosted PASS/FAIL | Log/artifact path | Reviewer note`. Do not write PASS until supported by real results.
