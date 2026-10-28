# Codex prompt — VSLICE-004

Read `AGENTS.md`, accepted ADR-001–006, VSLICE-001–003, and `docs/implementation/vslice-004/{SPECIFICATION,TEST_MATRIX,VALIDATION_TASK,REVIEW_CHECKLISTS}.md`. Inspect the actual repository state and existing OpenRouter integration before editing.

Implement **VSLICE-004: Live Operator — End-to-End User Experience** as a Linux desktop product milestone, reusing the accepted provider adapter, Operator proposal/approval, registry, kernel, WASM, trusted semantic Qt UI, and persistent room contracts without modifying accepted ABI or schemas.

Make provider selection/configuration, truthful outbound disclosure, separate consent, progress/cancellation, structured preview, genuine approval, error recovery, reactive editing and offline cold restoration usable in the real Qt app. Add executable **VS4-001–026** tests that use a mocked remote transport but real app/registry/WASM/runtime boundaries. Require **two independent reviews**, full regression suite and **successful GitHub-hosted Linux CI** on the final source. Then complete separately supervised **VS4-030** against the real OpenRouter adapter with sanitized evidence. Record VS4-027–029 evidence and exact model/version/usage/latency where available. Keep status Proposed until all gates pass.

Do not add multi-component planning, remote registry, generation, new guest privileges, generated QML, other-provider acceptance, hidden approval, hardcoded mortgage outputs or insecure credential storage. Preserve the existing working offline path. Produce a reviewable PR and `docs/implementation/vslice-004/evidence/RESULTS.md` with commands, hashes, reviewer findings and CI/live links. Do not claim tests ran without evidence.
