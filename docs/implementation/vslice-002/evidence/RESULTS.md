# VSLICE-002 evidence

Status: PROPOSED. Independent reviews and local development tests pass; final clean
locked validation and downloaded hosted Linux evidence remain required.

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
