# UX-001 executable validation

Status: Proposed. The nine focused Qt cases use the real admitted component package, native worker, kernel and trusted semantic renderer. Remote cases use the accepted deterministic transport; they do not establish live-provider acceptance.

| Case | Gate |
| --- | --- |
| UX-001 | Default three-column navigation, hidden administration, keyboard Settings, read-only Components, focus return |
| UX-002 | Provider Settings, session opt-in, cancelled settings, no inference on enablement |
| UX-003 | Keyboard Ask, visible material parameters, explicit Start, real WASM and local reactive edit |
| UX-004 | Conversation is local presentation; only current request enters model context; secrets excluded |
| UX-005 | Missing credentials, malformed/forged suggestion, authentication and refusal fail safely |
| UX-006 | Truthful topology/initialization failure and unapplied draft status |
| UX-007 | Pinned navigation focus and separate offline cold process with zero model/guest display calls |
| UX-008 | Stale approval refresh, no automatic retry and fresh explicit request/approval |
| UX-009 | Optional passive trace, private new-only file, bounded allowlisted records, forbidden fields excluded |

Run `bash apps/desktop-qt/validate-ux-001.sh` in the documented dependency environment. This invokes the unchanged VSLICE-004 validation wrapper and all retained regressions before the focused tests. Product launch is `./workbench`; build with `./workbench --build`. See WORKBENCH.md.

Two source-bound independent reviews must approve the same implementation source inventory. Hosted evidence must identify the exact PR merge source, runner/run/attempt, successful executed cases, retained regression archive and hashes. No skipped case counts as a pass.

A separate human-operated OpenRouter walkthrough is required: use Settings to enable the supported profile for this session, Ask with the synthetic mortgage request, inspect the validated suggestion, explicitly Start, edit locally, pin, close and reopen offline. Record observed model/usage/latency where supplied, process IDs, call counters and committed values. Never infer this gate from fixtures.

`--trace /absolute/private/new-file.jsonl` is an optional validation-only passive observer, disabled by default. Its final parent must be owned by the current user with mode 0700; every ancestor and the new mode-0600 file are opened without following symlinks. It excludes prompts, conversation, credentials, credential paths, raw responses and arbitrary future fields. It includes component pins, approved synthetic parameters, committed canvas values, native audit counters, timing and process IDs; share only reviewed synthetic traces. Records stop on write failure or the 10 MiB limit. A clean walkthrough must include `trace.closed` with `accepted: true`. The trace cannot issue commands or approve proposals.

The product-owned `operator-direct/Main.qml` derives from the accepted `vslice-003/Main.qml`. Its only semantic presentation change is a `bridge.hasDraft(id)` guard on editingFinished: focus-only transitions do not create typed input. Actual text edits still create drafts through the accepted ownership-checked bridge and use the same edit ingress. Legacy source and its conformance suite remain byte-preserved. UX-003 checks both unchanged focus navigation and real QML keystrokes; UX-008 covers the stale-handoff failure this prevents. Keep this narrow maintained copy aligned with future trusted renderer fixes; it contains no package executable QML.
