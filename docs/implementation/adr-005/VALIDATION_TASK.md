# ADR-005 implementation and validation instructions

1. Confirm parent branch contains accepted ADR-001–004 and VSLICE-001–002 commits/evidence. If PR #8 is unmerged, use its validated branch or arrange merge first. Record the exact base SHA. Read `AGENTS.md` and the committed schemas; prefer actual current files over bootstrap examples.
2. Land `docs/adr/ADR-005.md` and keep status PROPOSED. Store this kit under `docs/implementation/adr-005/` and place supplemental proposal schemas in a non-conflicting, versioned location pending schema review.
3. Reconcile accepted composition plan and actual installed mortgage manifest/ports. Implement deterministic read-only candidate projection without hardcoded mortgage identifiers or registry paths. Preserve accepted WIT, runtime, registry, UI and room snapshot contracts.
4. Implement bounded `OperatorProvider` and strict JSON parser. Start with offline fixture provider. A remote provider adapter is optional and must not be required by CI. Never expose provider secrets/URLs to component binaries.
5. Implement typed suggestion, preview and host-side canonical plan construction. Revalidate candidates, fields, schema, permissions and expected room revision before applying. Implement reviewable, proposal-bound single-use consent.
6. Wire a minimal Operator request + preview + approval UI into trusted Qt. Do not give the model raw host commands, tool execution, or arbitrary QML.
7. After valid topology commit, initialize inputs only through accepted ingress. Distinguish plan commit from later input acceptance in logs and errors. Verify no Operator re-inference after ordinary edits or restore.
8. Add non-ignored tests for OP-001–024, including adversarial metadata, schema abuse, fake pins, replayed approval, race/drift, timeout and missing capabilities. Preserve and run all accepted suites.
9. Obtain two **independent** reviews: (A) prompt injection/privacy/authorization and (B) kernel/registry/state integration. Independently reproduce findings and fixes; report reviewer identity/role and review commit.
10. Run Clippy, formatting, JSON/Turtle/Python contract validators, full Cargo suite, Qt offscreen/integration and real desktop-process smoke tests; run the same checks in GitHub-hosted Linux CI. Archive commands, versions, hashes, logs and stable artifact URLs. A workflow file existing is not a hosted run.
11. Mark ADR-005 Accepted only after all 24 gates, both reviews and hosted CI pass. Otherwise document BLOCKED/PROPOSED status and remaining evidence. Do not weaken existing tests or claim general multi-component planning.

## Suggested agent task splits

- OP-A: schemas and adversarial fixtures (contracts owner).
- OP-B: local semantic discovery and minimal candidate projection (registry/semantic owner).
- OP-C: provider adapter, suggestion parsing and privacy policy (agent owner).
- OP-D: preview, authorisation/revision-fenced application (kernel/capabilities owner).
- OP-E: Qt conversation/preview adapter and integration tests (desktop owner).
- OP-F: independent review, evidence packaging, CI (verification owner).

Do not concurrently edit shared contract files without one designated integrator. New tests must exercise real accepted runtime for the product-path gate, even if provider output is deterministic.
