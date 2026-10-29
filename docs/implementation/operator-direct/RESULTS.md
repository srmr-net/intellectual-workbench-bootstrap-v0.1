# Direct Operator development entry

Implemented following the user's2026-10-29 instruction to stop the scripted consent walkthrough and prioritize direct interaction. This is an exploratory entry, not a new accepted architecture milestone.

User flow: launch with explicit `--live` or enable OpenRouter once in Settings; type a request; Ask Operator; review the tool/inputs; Start this tool. No per-request consent dialog or chat approval. Starting remains a genuine proposal-bound host approval. The native guard, provider limits, exact pins, registry/kernel/bus/WASM, renderer and storage are reused unchanged. No new guest privilege, ABI/schema or model execution authority.

Build succeeded using `apps/desktop-qt/operator-direct/build.sh`. Focused genuine Qt integration passed both data cases: actual adapter with mocked HTTP, and offline without credentials. QtTest totals4 passed (including init/cleanup),0 failed,0 skipped,15.867s. Tests assert no room before Start, real WASM1461.48 after one Start gesture,1753.77 after typed bridge editing, and exactly one mocked request / zero offline requests. Logs under evidence/. Root fmt and Python contract checks passed. No actual remote request is claimed by these tests.

The real Wayland app was opened in normal interactive mode with the production-only worker and the user-provided private source. It does not run validation actions, deadlines or scripted approvals. It remains open for the user to interact directly. Private rooms persist outside build output. Actual user requests/results are not scraped into evidence by default.

Current catalog is limited; mortgage is the working installed example. General component generation and multi-component planning are not implemented. Polished UX, persistent preferences, broader catalogs and formal live-milestone acceptance are deferred. VSLICE004 remains Proposed; its two scripted attempts sent zero inference requests (expired review, then already-checked checkbox treated as a harness error). Those failures do not imply a passing VS4-030.
