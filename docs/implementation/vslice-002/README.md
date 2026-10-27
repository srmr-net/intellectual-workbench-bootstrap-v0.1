# VSLICE-002 — Interactive Mortgage Calculator and Persistent Canvas

**Status:** PROPOSED. This package is a specification and validation kit, not implementation evidence.

## Repository integration

Copy the content to `docs/implementation/vslice-002/` (and the fixture to its test fixture directory). Read `AGENTS.md` and the repository's **accepted** ADR-001–004 and VSLICE-001 implementations/evidence before editing code. If example paths or shapes differ from actual accepted exports, record the mapping; do not overwrite accepted contracts.

Start with `SPECIFICATION.md`, follow `VALIDATION_TASK.md`, implement `TEST_MATRIX.md`, fill `evidence/RESULTS.template.md`, and request the two independent reviews in `REVIEW_CHECKLISTS.md`.

This kit intentionally defines **durable snapshot v0.1**, not fully transactional database recovery. Its acceptance requires demonstrable durability and safe failure handling across a genuine app-process restart; more complex crash-consistency and storage-engine design remain deferred.

No accepted status until genuine Linux desktop/offscreen integration tests and GitHub-hosted CI have passed, reviews are closed, and evidence is archived.
