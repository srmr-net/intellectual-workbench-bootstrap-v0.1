# Intellectual Workbench — Operator chat

The default Linux Qt entry is a conversation with Operator:

```sh
./workbench --build
./workbench
# Or supply the private host credential reference explicitly:
./workbench --credential-file "$HOME/openrouter.key"
```

Open Settings, enable OpenRouter for this application session, and start chatting. Enter sends; Shift+Enter adds a new line. Stop cancels the active request. New chat creates an isolated conversation; the sidebar reopens conversations from this process. The private `~/openrouter.key` source supplied for this project is used if present and no other host credential source is set. Its contents never enter the UI.

This is real contextual text conversation, not repeated structured tool requests. The scrollable transcript remains for this application session; **chat history is not yet persisted across process restart**. Each request sends the new message plus up to twelve recent successful conversation turns. Settings discloses the fixed OpenRouter/NVIDIA free profile, provider logging and request limits. Remote is off until enabled; replies arrive as complete messages, with progress and cancellation while waiting.

Components are secondary. Settings → Tools opens the existing validated component workspace; `./workbench --tools` also opens it directly. Existing pinned tools and room snapshots remain in their original private profile and retain their offline restoration behaviour. Chat does not read those snapshots or the registry and cannot execute anything. Creating tools still requires the existing validated suggestion, exact-pin preview and explicit Start approval. No shell agent, code generation, remote registry or additional guest privileges are introduced.

The prior UX-001 implementation is preserved in [draft PR14](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/pull/14). This chat-first iteration is Proposed under [CHAT-001](docs/implementation/operator-chat/TASK.md) / [ADR016](docs/adr/ADR-016.md); its reviews and validation are separate. See [chat build/API notes](apps/desktop-qt/operator-chat/README.md).
