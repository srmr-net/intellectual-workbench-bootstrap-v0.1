# Linux Workbench desktop — VSLICE002

Status PROPOSED until all mandatory reviews and local/hosted gates pass.
This three-column shell embeds the byte-preserved accepted Main.qml renderer:
installed components/pinned rooms at left, local status/context in the centre,
semantic form at right. No mortgage-specific QML or view calculation exists.
The actual manifest has no display name, so the picker uses admitted component id;
room title comes from validated UI metadata. Selection resolves exact immutable pins.

Development prerequisites match the accepted Qt frontend: Rust1.99.1,
cargo-component0.21.1, Wasmtime49.0.1, Python tools/requirements.txt and system Qt6
Core/Gui/Widgets/Quick/QuickControls2/Test/QML development libraries and qmake6.
Dynamic system Qt linkage uses LGPLv3/GPLv3 or commercial licensing. Qt Test provides
explicit native conformance instrumentation; ordinary launch does not execute it.

```sh
bash apps/desktop-qt/validate-vslice-002.sh
# Choose private absolute locations; their parent directories must exist.
target/debug/wb-desktop-install /absolute/registry /absolute/python \
  "$PWD/tools/registry_validate.py" "$PWD/target/debug/wb-registry-inspect" \
  "$PWD/target/ui-fixtures/mortgage-ui"
target/desktop-build/workbench-desktop "$PWD/target/debug/wb-desktop-worker" \
  /absolute/registry /absolute/workspace /absolute/python \
  "$PWD/tools/registry_validate.py" "$PWD/target/debug/wb-registry-inspect"
```

Select the installed component and Open new canvas; edit the three declared fields.
Only committed output is displayed. Invalid/incomplete edits identify their field
without running the guest. Pending/draft/conflict messages retain prior committed
output; explicit Apply retries the retained draft. Pin/unpin and select local rooms.
Quit normally; reopening the same workspace restores its selected/pinned exact UI,
inputs and payment with no guest execution or model call for display. No source
bundle, remote registry, inference service or silent version substitution is used.

Workspace storage is private workspace.json format1 with checksum, strict closed
schema, CAS and fsync/atomic rename. A second writer fails; corruption or missing
pins produces read-only recovery without unrequested repair/overwrite. Do not delete
the file to hide an error. Old ADR004/headless snapshots are not silently migrated.
Clean shutdown is tested; database/power-cut recovery remains outside this slice.

All QObject updates stay on GUI thread; native admission/compute/storage live in the
worker. Native widgets and Quick window have generic admitted-order keyboard focus
handling. New desktop transport is independently bounded1MiB/32frames; accepted
WIT/UI/IPC contracts and all original tests remain unchanged.

Optional --validation-actions FILE --validation-report FILE drives only whitelisted
native UI actions for genuine process tests, records actual PID/childPID/binary hash,
QAccessible/widget text and screenshots. It never evaluates package scripts or adds
guest services. Production operation uses the six positional arguments above.
