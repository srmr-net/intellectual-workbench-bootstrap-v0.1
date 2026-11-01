# CHAT-001 — Chat-first Operator with real OpenRouter conversation

Status: Proposed. Owner: Codex. User-directed 2026-10-29 after supervised UX-001 feedback. Parent reviewed baseline: PR14 / 77351e4. User explicitly selected real contextual OpenRouter chat, not fixtures alone. Existing component work is preserved and secondary.

Goal: a long-running scrollable conversation, bottom composer, Enter/Shift+Enter, genuine multi-turn model context, understandable progress/errors/cancel, session provider settings and new/select chat. No requirement for complete mortgage parameters merely to converse. Tools remain available through the existing validated tool workspace, without execution authority from chat text.

Contract versions: accepted WIT, manifest/UI/plan/ontology/room and ADR001–006 unchanged. New ADR016 documents host-only conversation protocol; this is not a change to the accepted bounded suggestion contract. No code generation, shell tools, model tool calls, registry/file discovery, background agent execution, permissions or new guest services.

Owned paths: apps/desktop-qt/operator-chat/, root operator-chat launcher, default product wrappers/docs when integrated, docs/implementation/operator-chat/, docs/adr/ADR-016.md, backlog, scoped new CI. Frozen legacy runtime/provider/renderer and all accepted evidence remain unchanged. Reuse accepted credential policy and fixed HTTPS transport source, not a new networking implementation.

Delivery: native Rust bounded conversation worker with mocked transport tests and strict text response parsing; trusted Qt chat UI and focused tests; separate in-memory transcript and bounded request context; no chat/secret data in existing room snapshots or logs. Initial transcript/history lasts for this application process; disk conversation persistence is explicitly deferred rather than silently claiming it.

Bounds: one active request; explicit send per call; no retries/tools/fallback; fixed OpenRouter/NVIDIA free profile; session opt-in disclosure includes recent chat context; latest12 complete turns within48KiB context; max4KiB user message, max16KiB assistant, max1024 output tokens and30s deadline; bounded session/transcript quotas. Credential contents remain host-only. Cancellation drops/reaps the accepted transport and late results cannot enter another chat.

Acceptance commands: cargo fmt/test/clippy --locked --manifest-path apps/desktop-qt/operator-chat/host/Cargo.toml; qmake6 apps/desktop-qt/operator-chat/tests.pro and real worker-backed Qt tests; root required checks; unchanged retained regressions; source-bound independent UX/security review for new scope; hosted Linux deterministic tests without credentials. Live chat only through deliberate user sends after in-app session enablement, not scripted hidden inference. Keep Proposed until evidence exists.

Failure cases: no consent/key, authentication/rate/timeout/refusal/truncation, malicious tool calls/markup/links/secret echo, invalid frames/duplicate keys, excessive input/context/output, cancel/switch/disable/quit, failed messages excluded from context, no history leakage between chats, no model calls at startup or opening tools/canvas state. No automatic component creation from prose.
