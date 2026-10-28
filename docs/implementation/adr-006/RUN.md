# Run the live Operator desktop yourself

Status: Accepted for the Linux/OpenRouter profile; hosted and live evidence is
recorded in `evidence/RESULTS.md`. Run these from a
normal Linux desktop terminal in the repository. Validation processes use separate
registries/workspaces.

The prepared launcher in this checkout is:

```sh
cd /home/scot/Projects/intellectual-workbench-bootstrap-v0.1
./target/run-live-workbench.sh
```

It uses the final production-only `live-http` worker in `target/llm-manual`, the
trusted Qt executable in `target/llm-qt-build`, private host credential file
`~/openrouter.key`, and a separately admitted real mortgage UI package under
`target/manual-workbench/registry`. It starts with the offline provider selected.
It does not read or copy credentials into the launcher, workspace or bundles.

1. Select **OpenRouter NVIDIA** in the centre column.
2. Enter `I need a mortgage calculator for a £250,000 loan over 25 years at 5%.`
3. Tick **Consent to next remote request**, then select **Preview**. Enter the
   request before consenting: editing revokes any already armed consent.
4. Review the actual host-built proposal and exact pin/inputs. Tick its separate
   approval checkbox, then **Confirm**. The initial payment should be1461.48.
5. Edit the canvas fields normally. Principal300000 at the same rate/term should
   commit1753.77 without any further model call.
6. **Pin** the room and close the window normally. Relaunch the same script to
   restore exact committed state without a model or guest call merely to display.

The remote model is exactly `nvidia/nemotron-3-ultra-550b-a55b:free`, NVIDIA-only,
no provider/model fallback, zero-price routing filters. Its endpoint does not offer
provider JSON Schema enforcement: the accepted strict host parser remains
mandatory. The UI discloses NVIDIA logging/data use before each remote consent.
No exact-spend or provider-retention guarantee is made.

This is a development launcher: its saved workspace is
`target/manual-workbench/workspace`. Keep that directory if cleaning build output.
The launcher itself is generated under ignored `target/`; the equivalent explicit
launch command after the documented validation/build/install steps is:

```sh
export WB_LLM_PROVIDER=openrouter-chat
export WB_LLM_MODEL=nvidia/nemotron-3-ultra-550b-a55b:free
export WB_LLM_SECRET_FILE="$HOME/openrouter.key"
export QT_QUICK_BACKEND=software
unset WB_LLM_TEST_TRANSPORT WB_LLM_BUDGET WB_OPERATOR_PROVIDER QT_QPA_PLATFORM

target/llm-qt-build/workbench-live-provider \
  "$PWD/target/llm-manual/debug/wb-live-worker" \
  "$PWD/target/manual-workbench/registry" \
  "$PWD/target/manual-workbench/workspace" \
  "$PWD/target/vs2-venv/bin/python" \
  "$PWD/tools/registry_validate.py" \
  "$PWD/target/debug/wb-registry-inspect"
```

Rebuild the production worker with:

```sh
cargo build --locked --manifest-path apps/desktop-qt/live-provider/host/Cargo.toml \
  --features live-http --target-dir target/llm-manual
```

No validation-action arguments are used for interactive operation. Use the
existing Qt6 prerequisites and full `validate-live-provider.sh` workflow to
regenerate fixtures, installer/inspector, Python environment and Qt build.
