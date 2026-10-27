# Operator entry for the existing Linux Workbench

Build and validate with `bash apps/desktop-qt/validate-operator.sh`. Use the accepted
local installer to admit the real UI package, then launch:

```sh
target/operator-build/workbench-operator "$PWD/target/debug/wb-operator-worker" \
  /absolute/registry /absolute/empty-workspace /absolute/python \
  "$PWD/tools/registry_validate.py" "$PWD/target/debug/wb-registry-inspect"
```

Enter an explicit tool request, review exact pin/inputs/permissions/effects, check
fresh consent, then Approve and initialize. The fixture recognizes the documented
GBP mortgage request; all selection uses admitted semantic metadata and opaque keys.
The center preview is plain native text and the canvas is the existing generic QML
renderer. No calculator-specific QML or model-produced executable content exists.

Only committed, durably validated outputs appear. Direct form edits use the existing
runtime with no provider. Quit and reopen the same workspace to display its exact
committed pin/value offline without calculation. WB_OPERATOR_PROVIDER=disabled is
an explicit provider-unavailable/offline test configuration, not a remote adapter.
Partial initialization offers direct form repair when topology is saved; otherwise
resolve storage diagnostics and preview again. Approval is the commit boundary;
Cancel is available while preparing, not after confirmation.

The first-use profile requires an empty workspace and targets one existing empty
native room. Existing saved workspaces restore/edit normally; adding to a populated
room is outside this single-component planner. No provider call occurs at startup.
The bounded status query reads instrumentation only, never invokes inference.

Optional --validation-actions FILE --validation-report FILE uses whitelisted native
UI events, actual worker processes, QAccessible data and screenshots. It is the same
product executable and never loads package code/QML strings or guest services.
