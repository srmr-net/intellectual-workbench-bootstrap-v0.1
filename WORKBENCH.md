# Intellectual Workbench — Linux desktop

The default product entry is the simplified Operator workspace:

```sh
./workbench --build
./workbench
# Optional explicit session opt-in with an existing private host credential:
./workbench --live --credential-file "$HOME/openrouter.key"
```

Use Settings → Provider to enable OpenRouter for the current process or change the private file reference. The supported model and approved endpoint are shown there; this profile does not accept arbitrary model/endpoints. Credential contents never enter the UI or saved workspace. No provider is needed for ordinary form edits or reopening saved tools.

Ask for a tool and its inputs in the centre column, review the suggestion, then choose **Start this tool**. The host validates the exact installed package and proposal before starting. The current installed example supports “I need a mortgage calculator for a £250,000 loan over 25 years at 5%.” Other applications require corresponding admitted local components; no code generation or multi-component planning is introduced.

The left sidebar contains workspaces, pinned tools and session activity. The right canvas displays the selected semantic tool. **Pin tool** preserves a shortcut; all committed workspace state uses the existing durable snapshot. The default private profile remains `${XDG_STATE_HOME:-$HOME/.local/state}/intellectual-workbench-operator`; `--profile /absolute/private/directory` selects another profile without altering existing data.

Settings → Components provides read-only installed versions, exact pins and local registry information. Technical diagnostics are under Settings. Legacy conformance/demo entry points remain available for reproducible regression tests, but are not the product launcher.

UX-001 and its parent VSLICE-004 remain Proposed until their required evidence, including a separately documented live walkthrough, is complete. See [UX-001](docs/implementation/ux-001/TASK.md) and [default app](apps/desktop-qt/operator-direct/README.md).
