# Trusted semantic Qt frontend

ADR-004 implementation, separately gated from accepted headless runtime contracts.
All loaded QML is compiled into qrc resources. Package metadata selects only a
fixed form/control vocabulary; it supplies no executable QML, property names,
resource paths or JavaScript. Text is rendered as PlainText. No mortgage identity
or field name appears in the renderer. The alternate metadata fixture verifies it.

UiBridge owns QProcess and all QObject state on the GUI thread. Heavy admission
and WASM work stays in wb-ui-worker. Length/queue limits, exact textual counters,
room/instance ownership, generation/revision/sequence checks and QPointer/view
callbacks fence teardown and room switches. Editor objects retain focus/cursor
across projections; uncommitted local drafts are separate from committed values.
A visible QUEUE_FULL or STATE_CONFLICT requires explicit Apply; no implicit retry.
Accessibility tests inspect real QAccessible roles/names/focus and use actual key
input/tab order. Large text and light/dark screenshots are archived.

Install Qt6 development libraries for Core/Gui/Quick/QuickControls2/Test/Qml,
qmake6, C++17 compiler, pkg-config and the QML Quick/Controls/Layouts/Window/
Templates/WorkerScript modules. The validation uses public Qt6.4 APIs, dynamically
linked system Qt (LGPLv3/GPLv3 or commercial). Local and hosted actual versions
are recorded rather than claiming cross-version binary portability.

```sh
bash apps/desktop-qt/validate.sh
# After validation, launch the actual generic UI:
target/qt-build/workbench-ui target/debug/wb-ui-worker \
  "$PWD/target/ui-fixtures/mortgage-ui" /absolute/private/registry \
  /absolute/python tools/registry_validate.py \
  target/debug/wb-registry-inspect mortgage-room
```

The registry parent must already exist. Optional last argument `restore` reopens
that room; execution approval is explicit native startup policy, never package
metadata or persisted grants. No Qt plugin, package script or guest service runs.
Linux CI uses offscreen software rendering and real accessibility instrumentation;
this does not certify platform screen readers, mobile packaging or native GPU UX.
