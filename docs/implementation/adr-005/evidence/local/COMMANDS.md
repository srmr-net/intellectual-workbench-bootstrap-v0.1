# Foundation local checks

These checks are not OP001–024 acceptance. Source identities: source-hashes.json;
accepted-byte preservation: contract-hashes.json. Version log's Git SHA is the base
commit because validation ran on the implementation working tree.

```
cargo fmt --all -- --check
REGISTRY_PYTHON="$PWD/target/vs2-venv/bin/python" cargo test --workspace --locked --offline
cargo test -p wb-operator --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
target/vs2-venv/bin/python tools/check_contracts.py
target/vs2-venv/bin/python -m unittest discover -s tools -p 'test_*.py' -v
```

Full workspace must run outside the execution sandbox: REG003 creates a Unix
socket. The sandbox-denied attempt is preserved separately. The earlier failed
attempt before the UI adapter lifetime fix is also preserved; never count it as
passing. Standalone registry rerun passed outside the sandbox.

Qt baseline commands execute existing binaries with QT_QPA_PLATFORM=offscreen,
QT_QUICK_BACKEND=software, QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1, absolute
WB_REPOSITORY_ROOT and REGISTRY_PYTHON, removing desktop platform theme/display
variables. Execute target/qt-tests/ui-conformance and
 target/desktop-tests/vs2-qt-conformance with -o <log>,txt. The first command omitted
WB_UI_EVIDENCE_DIR and failed screenshot capture. The affected accessibility case
was rerun with that variable pointing at this directory's ui/ subdirectory and
passed. Logs preserve both attempts. These are existing Qt regressions, not a new
Operator request/approval UI or OP mandatory process demonstration.
