# Live Operator Linux desktop — VSLICE-004 (Proposed)

This additive product shell reuses the accepted declarative renderer and desktop bridge, registry, kernel, mortgage guest and ADR-006 provider library. The native worker adds a read-only outbound review and a one-use binding to the actual future request. Opaque candidate selectors are generated only at send; the disclosure explicitly shows placeholders. Every other dynamic field, candidate ordering/multiplicity, exact package/UI/schema pin and durable workspace revision must still match before the accepted adapter can send. No accepted schema changes. Request progress/Cancel and reviewed data/proposal sections scroll into view without changing focus or activating consent.

## Build and run

Linux requires Rust 1.99.1, cargo-component 0.21.1, Python dependencies in `tools/requirements.txt`, Qt 6 Widgets/Quick/Controls and the QML modules listed in `.github/workflows/vslice-004.yml`. The full reproducible build/test command is:

```sh
PATH="$PWD/target/vs2-venv/bin:$PATH" bash apps/desktop-qt/validate-vslice-004.sh
# Build the production worker separately: no mock feature.
cargo build --locked --manifest-path apps/desktop-qt/vslice-004/host/Cargo.toml --features live-http --target-dir target
apps/desktop-qt/vslice-004/run.sh --credential-file "$HOME/openrouter.key"
```

The launcher defaults to a private persistent `${XDG_STATE_HOME:-$HOME/.local/state}/intellectual-workbench` profile, outside build output. `--profile /absolute/private/path` selects an independent profile. Keep this directory to preserve installed exact pins and rooms. The default local package is the real compiled fixture produced by the validation build; `--package /absolute/local/package` selects another package for actual registry admission. Installation does not run the guest. Repeated launch resolves the same immutable package.

Offline is the initial provider. The installed-tool picker works without remote inference. For a remote request, select **OpenRouter**, choose an existing owner-only private credential file (path only, never paste the key), and type an explicit request, for example “I need a mortgage calculator for a £250,000 loan over 25 years at 5%.” Click **Review request / outbound data**. Inspect the exact request, bounded catalog projection, generated-selector policy, fixed instructions/schema, destination, model and budgets. Check the inference-consent checkbox, then **Send reviewed data to OpenRouter**. This does not approve execution.

Inspect the structured host-built proposal. Check the separate execution confirmation and approve, or reject. Topology commits first; typed initialization and validated output persistence are subsequent operations. On an initialization failure the installed tool remains; repair it through its typed form, without resubmitting topology. Only committed output appears on the canvas. Edit the principal to 300000 to obtain the reactive result without another provider call. Pin the room and close normally. Reopening restores committed state without inference or guest calls solely for display; Offline remains available.

Only the accepted `nvidia/nemotron-3-ultra-550b-a55b:free` OpenRouter/NVIDIA route is configured. NVIDIA's free endpoint logs requests for security/product improvement. It does not enforce JSON Schema; strict local suggestion/plan validation is mandatory. Availability is not authentication: private-file metadata is checked in the UI, and the accepted credential loader validates again at send. Source references are transient and never saved in room snapshots. To reuse a source after restart use the CLI flag or approved `OPENROUTER_API_KEY` environment source. Request review/consent expires, is single-use and is invalidated by source/provider/request/workspace changes. There are no automatic retries or model tools.

## Native API and validation

`operator.1`, `desktop.1` and `provider.1` retain accepted semantics. A local product-only `ux.1` channel adds strict bounded `review`, `send(reviewId, reviewDigest)`, `withdraw`, `credential(path)` and read-only `status`. Raw remote `operator.1 prepare` is rejected without the native one-use review guard; raw `provider.1 consent=true` cannot arm remote inference. The `mock` operation requires the separately compiled test transport feature and is rejected in production. No guest sees this control channel.

The test worker enables `test-transport` only and cannot make live HTTP calls. It uses actual owner-only synthetic credential files and the accepted OpenRouter adapter/parser, replacing only transport. Production is built with `live-http` only; a test-mode environment variable disables remote configuration rather than falling back to a real endpoint. See `docs/implementation/vslice-004/evidence/RESULTS.md` for the actual gate status and evidence.

## Supervised live validation

`vslice-004-tests/live_smoke.py` is explicit validation instrumentation, forbidden in CI. Build a production-only worker separately:

```sh
cargo build --locked --manifest-path apps/desktop-qt/vslice-004/host/Cargo.toml --features live-http --target-dir target/vs4-live
python3 apps/desktop-qt/vslice-004-tests/live_smoke.py --credential-file "$HOME/openrouter.key" --consented --output /absolute/new/private/session
```

Use an active Linux display; this launcher preserves DISPLAY/Wayland and opens the real visible shell. It first publishes `report.json.disclosure.json` and waits for a fresh, matching `{ "reviewDigest": "…", "decision": "consent" }` in `report.json.consent.json`. This is supervised validation of the displayed data review, not permission to send on a timer. The driver then activates the real inference checkbox/Send controls. After a valid provider response it publishes `report.json.preview.json`, waits for a separate `{ "proposalDigest": "…", "decision": "approve" }` in `report.json.approval.json`, then activates genuine Qt confirmation/Approve controls. Rejection, mismatch or expiry stops the run. No decision files may preexist. Ordinary users use the visible UI directly and need no files.

The script sends only the documented synthetic request and bounded catalog, at most one request, 512 output tokens and 20 seconds, with no retries. It validates real outputs, reactive editing, pinning and a second offline OS process with denied IPv4/IPv6. Keep its private registry/workspace and reports local until sanitizing/hashing evidence. Do not publish credential references, confidential text, machine environment or keys. The returned model ID/usage/latency is recorded by the accepted adapter audit; unknown provider billing/version fields must be reported as unavailable rather than invented.

The launcher also forwards an explicitly paired `--validation-actions` and `--validation-report` to the whitelisted native Qt test driver. These flags are never enabled by default and do not accept scripts or guest commands. VS4-001 exercises this documented launcher with a private test profile.
