# UX-001 — Simplified Operator Workspace and Settings Refactor

Status: Proposed. Owner: Codex. Parent VSLICE004 remains Proposed. Baseline: merged PR13/master9c0b27e. User-supplied UX-001 requirements2026-10-29 govern this task.

Goal: make the direct Operator the default product launcher, present workspaces/pinned tools/recent activity in the sidebar, conversation/suggestions/progress in the centre and the selected semantic tool on the canvas. Relocate component inspection, provider/model settings and diagnostics into Settings.

Owned paths: apps/desktop-qt/operator-direct/, new apps/desktop-qt/{build,run}.sh, new root workbench launcher and WORKBENCH.md, docs/implementation/ux-001/, backlog. Preserve accepted source-bound desktop shell/bridge/renderer/worker and existing workflow wrappers byte-for-byte. Add a scoped presentation adapter around those widgets rather than duplicate orchestration or change host interfaces.

Dependencies: accepted ADR001–006 and VS1–3; PR13 direct Operator; real admitted mortgage package. No WIT/schema/kernel/registry/permission/persistence changes. No extra Operator execution path, arbitrary QML, remote registry, generation or multi-component planning. Provider consent is session-scoped and secrets remain host-controlled.

Checks: focused Qt navigation/Settings/read-only components/approval/reactive/pinning/cold-process/error/accessibility tests; unchanged full validate-vslice-004.sh chain; locked root fmt/test/clippy/contracts. New outer validate-ux-001.sh and hosted Linux workflow. Source-bound independent UX/accessibility and security/runtime reviews. Archive logs/screenshots/versions/hashes. Separately supervised real OpenRouter walkthrough; never infer live validation from fixtures.

Acceptance: user-facing default has no component management controls; Settings exposes installed exact versions/pins and registry; request→Ask→validated readable suggestion→one Start; edits/restoration need no LLM; keyboard/focus/status accessibility; preserved regressions. Keep UX001 and VSLICE004 Proposed until required evidence exists. No accepted contract blocker identified; root README and legacy conformance entry remain byte-preserved because retained evidence binds their bytes, with the new ./workbench product entry documented in WORKBENCH.md.
