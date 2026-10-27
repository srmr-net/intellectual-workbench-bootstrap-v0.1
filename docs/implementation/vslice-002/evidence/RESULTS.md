# VSLICE-002 evidence

Status: ACCEPTED — 2026-10-27. All24 mandatory gates, both independent reviews,
clean locked local and successful hosted Linux checks and downloaded artifact verification pass.

Base: merged master c117632f7c50ed0cf729fadd814ac8548c22db0d (accepted PR6/7).
Reproduce: `bash apps/desktop-qt/validate-vslice-002.sh` with pinned Rust/guest tools,
Python requirements and Qt6 Core/Widgets/Quick/QuickControls2/Test/QML development
modules. This outer wrapper retains the entire unchanged accepted UI wrapper and
all ABI/IPC/REG/VSLICE001 checks. See IMPLEMENTATION_MAP.md and API READMEs.

| Gate | Executable evidence |
|---|---|
| VS2-001 | process_cases.test_vs2_001: actual product executable, PID/hash/version/exit |
| VS2-002 | process_cases.test_vs2_002: actual three-column widget geometry/visibility |
| VS2-003 | process_cases.test_vs2_003: installed metadata picker/exact selected pin |
| VS2-004 | native vs2_004: tampered exact release, read-only recovery |
| VS2-005 | process_cases.test_vs2_005: alternate admitted metadata through unchanged QML |
| VS2-006 | native vs2_006: actual manifest aggregate and read-only binding |
| VS2-007 | native vs2_007: real mortgage1461.48 and500.00 |
| VS2-008 | native vs2_008: real canonical ingress/audit/committed output |
| VS2-009 | native +process009: invalid/incomplete/schema errors, no compute/revision |
| VS2-010 | native010 actual cancelled invocation; Qt010 pending switch/stale callback |
| VS2-011 | native011:140edit audit bounds,73415byte actualGuest response; Qt011 real queue2 |
| VS2-012 | native +Qt012: actual STATE_CONFLICT, retained draft, explicit Apply |
| VS2-013 | native +process013: real guest trap/bad output, prior coherent checkpoint |
| VS2-014 | Qt014 error-drain quit; process014 switching/teardown actual workers |
| VS2-015 | Qt015 actual field-only QAccessible errors; process015 keys/Tab/focus |
| VS2-016 | native016 ownership; process014 distinct component pins/room values |
| VS2-017 | native +process017 durable pin/unpin and canvas selection |
| VS2-018 | process018 two actual Qt/worker PID pairs, exact restored values,0compute |
| VS2-019 | storage019 atomic/CAS/lock; process019 repeated byte-stable cold restart |
| VS2-020 | native020 duplicate same-value/output-schema checks; process020 visible recovery |
| VS2-021 | storage021 corpus/path/FIFO/size; native021 bounded forgery; Qt021 stale ACK |
| VS2-022 | process022 actual exec-inherited socket denial,0compute offline restore |
| VS2-023 | two independent reports/source hashes/executions checked by collector |
| VS2-024 | full locked wrappers +successful hosted run/download verification required |

Native tests: crates/wb-desktop-host/tests/vs2.rs and crates/wb-storage/tests/vs2.rs.
New genuine Qt tests: apps/desktop-qt/desktop-tests/Vs2Conformance.cpp.
Process tests: apps/desktop-qt/desktop-tests/process_cases.py (same product binary,
explicit whitelisted native test instrumentation, actual QQuickItem text checks).

The first launch test exposed a null-focus dereference in test instrumentation;
fixed and actual embedded-window activation/Tab order tested. Review findings on
audit growth, transport size, pending retirement, quit-error drain and field-only
accessibility were resolved with real retests. The costly72-admission prototype
stress case was superseded, not counted as a pass: its replacement retains the
>64KiB/<1MiB checks with16actual rooms,32controls and a real510-digit guest echo.

Limits: clean-quit single-writer atomic snapshots, not full database or physical
power-loss recovery; same-UID host storage is trusted (checksum is not a signature).
Corrupt/incompatible/missing-pin state stays read-only and original bytes remain.
Cold rendering uses verified committed cache only; first later valid edit lazily
creates an approved kernel incarnation and suppresses restore recomputation.
No saved output/grant is injected as new kernel authority. ADR012 remains open.
Offscreen Qt cannot composite native child windows into QWidget.grab on all plugins;
shell and genuine canvas PNGs are separate supplemental captures, with actual widget
geometry, text, keyboard and accessibility assertions as primary evidence. Platform
screen-reader certification and cross-platform packaging remain deferred.


## Verified final execution

PR [#8](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/pull/8).
Reviewed implementation `615d76b54ac415bc1d9dd85b7237bfba3f035c68`;
validated branch `e16d6bc670eeb88fd879b67df325fb43590c1cbe` adds review bindings only.
[Successful hosted run40516418294](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40516418294)
tested synthetic PR checkout `0034953273c6da21b7c39807a5b2858f0716e7b6`.
GitHub commit API confirms identical tree `374d12e83058b384b08031ba1b9bb96c3df8d769`.
Later changes record acceptance/docs/evidence only. See decision.json and archived
workflow/job/artifact/checkout API metadata for runner and digest proof.

| Check | Clean local | GitHub-hosted Ubuntu24.04 x86_64 |
|---|---|---|
| Locked Rust workspace | 138 pass,0 failed/ignored | 138 pass,0 failed/ignored |
| New actual desktop process suite | 14 pass,25 Qt+worker PID pairs | same pass |
| New genuine Qt suite | 8 entries pass,0 failed/skipped | same pass |
| Retained actual Qt suite | 7 entries pass,0 failed/skipped | same pass |
| ABI / IPC / REG / UI / VSLICE001 | 10 /15 /20 /20 /20 gates pass | same pass |
| Original financial oracle | 33 cases and clean rebuild pass | same pass |
| Python/contracts | 5 harness tests, original30 + UI corpus,10 new snapshot fixtures pass | same pass |
| Rustfmt/locked Clippy -D warnings | pass | pass |
| Independent reviews | A/B APPROVED,21/22 source hashes | same exact reviewed bytes checked |

Rust/Cargo1.99.1, Wasmtime49.0.1, cargo-component0.21.1 and guest runtime0.41.0
retain accepted pins. Local Qt6.11.2/GCC16.2.1/Python3.14.7; hosted Qt6.4.2/
GCC13.3.0/Python3.11.17. Detailed versions, lockfiles, logs and actual packages are
inside archives. Local Qt headers emit GCC16 SFINAE warnings; no checks disabled.
Offscreen plugin native-parenting warnings are documented; actual canvas text,
geometry, focus, keyboard and accessible roles are instrumented, with separate
shell/canvas screenshots supplemental rather than a claimed composed screen capture.

Cold restart process evidence:

| Environment | First app/worker PID | New app/worker PID | Guest executions for restored display |
|---|---|---|---|
| Local | 662121 /662124 | 662140 /662143 | 0 |
| Hosted | 17551 /17553 | 17566 /17568 | 0 |

Both sessions display exact principal250000.00/rate5.00/term300/payment1461.48
and identical component/UI pins. Additional scenarios yield500.00 and repeated
quit/reopen keeps snapshot bytes stable. The separate offline exec probe verifies
ordinary IPv4/IPv6 sockets return EPERM before the real app/worker launch. Native
and GUI failure cases preserve the last coherent result and diagnostic bytes.
The actual product binary is archived, not a mocked window or local math substitute.

| Artifact | SHA-256 |
|---|---|
| local.zip | `5c913e04a0f62b81410558e53c717ea52828aedbf461aeb3a4d5d432afd8431c` |
| hosted.zip / artifact12740915566 | `7a8bb230d57912459c884f96f87a9a752e9216dc86cc4e2c265644fd50ddca89` |
| hosted-logs.zip | `6fe878d6bfd14b9f7d70299be7df49607698c67644524ddf3ee050e324b60455` |
| Local app | `c14b6c6d86b3081b7e3651f7b9eaa004d6b6aa06c86f86dd18864fb6d7dc1ae3` |
| Hosted app | `06d25cffa1ea208f5d49a8a963e5087e0aa1ff70494e2ce77dd9d66037655ccd` |
| Mortgage UI JSON | `b2bfe1b7ed2d83a762ca0bf884b4db6c4cee57af4d6876cf089b487ff428c5c6` |
| Hosted mortgage WASM | `b045130fe32aef76a0bf30cce9072b7b57ea2cb7b6bad37b23dfaf6a6e7573dd` |

Each archive verifies399 recorded file hashes,138 Rust passes, all retained gates,
25 separate app/worker boundaries,0 cold-display guest calls, exact pins, actual
binary hashes, two source-bound independent reviews and frozen accepted contracts.
Same-machine rebuild identity is checked; cross-machine binaries are recorded
separately rather than assumed equal. Superseded/duplicate workflow runs were
cancelled; the final reviewed-source run above completed successfully.

```sh
python3 docs/implementation/vslice-002/evidence/verify.py \
  docs/implementation/vslice-002/evidence/hosted.zip \
  --sha256 7a8bb230d57912459c884f96f87a9a752e9216dc86cc4e2c265644fd50ddca89 \
  --source 0034953273c6da21b7c39807a5b2858f0716e7b6 --hosted
```

[ADR013](../../../adr/ADR-013.md) records the internal checkpoint/coordination
choice; accepted ADR001–004/WIT/manifest/domain/UI/ontology bytes and original
regression suites are preserved. No unresolved mandatory finding remains in scope.
ADR012 database/power-loss recovery, richer/multi-instance canvas, cross-platform
packaging, namespace permanence and platform screen-reader certification remain open.
