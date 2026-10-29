# Direct Operator — development entry

This is the quick exploratory entry requested after the scripted VS4 walkthrough. It reuses the existing native worker and real registry/kernel/WASM/semantic canvas. It does not generate components or code. The installed catalog currently includes the mortgage calculator; broader catalogs/compositions remain future work.

```sh
apps/desktop-qt/operator-direct/build.sh
apps/desktop-qt/operator-direct/run.sh --live --credential-file "$HOME/openrouter.key"
```

`--live` explicitly enables OpenRouter for requests you submit in this session. Without it the deterministic offline provider is selected. Settings lets you switch providers and choose a private credential-file reference; key contents never enter the UI or room. Remote requests send your typed request and bounded local catalog to the accepted NVIDIA free model, with the existing host limits and validation. There are no per-request consent dialogs or chat confirmations in this entry.

Type “I need a mortgage calculator for a £250,000 loan over 25 years at 5%.” and click **Ask Operator**. Review the suggested tool and inputs, then click **Start this tool**. Edit the form on the canvas; calculations run in the real installed WASM with no further model calls. Pin the canvas to keep it. The private profile is outside build output under `~/.local/state/intellectual-workbench-operator` (or XDG_STATE_HOME); use `--profile /absolute/private/path` for another workspace.

Start remains an actual proposal-bound user approval, not model execution authority. The host still performs its internal request-body/pin/revision guard automatically after your Ask gesture and session provider opt-in. No accepted ABI, schema, permissions or runtime protocol changes. Topology commit and typed initialization remain separate; partial failures are surfaced. VSLICE004's exhaustive live/UX acceptance is deferred and remains Proposed; this exploratory entry does not claim that milestone passed.
