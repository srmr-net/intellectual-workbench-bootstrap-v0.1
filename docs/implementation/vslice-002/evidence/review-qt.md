# Independent Review A — Qt, accessibility and UI trust

Reviewer: `/root/review_vs2_qt`, distinct context, no implementation authorship. Date:2026-10-26.
Decision: **APPROVE**, scoped to exact source hashes in `review-qt.json`.
Base commit:c117632f7c50ed0cf729fadd814ac8548c22db0d. Implementation was an uncommitted snapshot during review; exact hashes bind reviewed bytes until the primary author commits identical sources. Later implementation changes require re-review.

Read AGENTS.md, README, contract status, VSLICE002 specification/matrix/assignment/checklists/task, accepted ADR004/protocol/evidence/review, old and new Qt sources/tests and relevant desktop native worker/session mappings. All15 accepted ADR004 review source hashes remain unchanged, including generic QML, original bridge/tests and native UI host. No implementation edits by this reviewer.

The new three-column shell embeds the byte-preserved real generic QQuickWindow. Package strings remain literal metadata; QML is fixed qrc source. No dynamic QML/JS, supplied URL or property expression executes. The attached accessibility description writes a fixed host property as data with the object's correct QQmlContext. QObject/process callbacks remain on GUI thread, protected by guards; admission, guest and persistence run in native worker. Stable delegates retain focus while projections update. Centre status distinguishes pending/unapplied drafts from last committed result. Exact room/instance/UI/revision/generation/sequence checks and local view epochs fence old callbacks. Queues/wire sizes/deadlines are finite; conflict retry is explicit. Normal shutdown drains accepted requests and waits asynchronously for native acknowledged quit and clean process exit.

## Findings and retest

- QT2-A01, medium: error/backpressure while draining could consume final pending operation without sending quit. Resolved by central continueQuit and finite shutdown deadline. Genuine worker shutdown with rejected input and injected backpressure passed independently.
- QT2-A02, high test gap: expected values checked bridge state rather than actual rendered text. Resolved by requiring corresponding real QQuickItem.text. Full independent process suite passed after fix.
- QT2-A03, medium: strict field-specific accessibility needs correct attached-property context; unrelated fields must not inherit global error. Correct object context/checks are present; independent QAccessible principal diagnostic and empty rate description test passed.
- QT2-A04, medium coverage: new bridge needed actual queue/conflict/stale tests, including retaining a live request after stale same-token frame. Six new genuine Qt cases passed independently after validation-before-retirement correction.

## Independent execution

Fresh qmake app build in target/review-vs2-qt-build and new test build in target/review-vs2-qt-tests passed, using Qt6.11.2 and GCC16; only system Qt-header SFINAE warnings. Actual offscreen Qt suite: **8 framework entries passed (6 functional),0 failed/skipped**,83.830s. It covers pending edit+room switch, queue capacity2/visible QUEUE_FULL/explicit Apply, real state conflict/no automatic retry, async rejected/error shutdown drain, field-specific QAccessible descriptions, stale pending preservation and exact counters. Samples are genuine Qt objects with a real admitted component and actual Rust worker; adversarial stale/error frame injection is explicit supplemental boundary coverage.

Complete process suite with independently rebuilt app and actual widget-text assertions: **14 passed,0 failed/skipped**,162.287s. It exercises actual key input/Tab/focus cursor retention, alternate metadata, real financial/trap cases, navigation/pinning, corrupt/unavailable/symlink/oversize recovery and offline cold restore. Archived reports show **25 distinct Qt PIDs and25 worker PIDs**. Independently compared **71 actual rendered readout samples** with committed projections; all match. Cold restart preserves exact pins/inputs/payment with0 guest invocations solely for display.

Logs, versions, commands, process reports/commands/PIDs, snapshots and screenshots are in `independent-qt.zip`; hash is recorded in review-qt.json. Initial pre-fix run logs are retained and distinguished from final runs. This is review approval, not milestone acceptance. Keep VSLICE002 Proposed until separate security/persistence review, full regression and verified successful hosted evidence. Offscreen instrumentation does not certify native GPU or every platform screen reader.

## Final native-only follow-up

Reviewed four changed native files: bounded token/control error echoes, bounded reader backpressure token, fail-closed random temporary nonce, and actual Echo16-room/32-control/510-digit large valid response replacing slow72-package admission stress. The test retains real guest invocation and framed roundtrip assertions (measured73415bytes, above64KiB and below1MiB). No Qt source or accepted ADR004 source changed. Review B independently reran all final native/storage cases. Independently reran genuine app VS2-018 against final native binaries: **1 passed,0failed/skipped**,10.437s; two actual Qt/worker process pairs restored exact committed values with0guest executions solely for display. Approval retained; four source hashes refreshed and archive expanded with follow-up reports.

## Final readable-diagnostics follow-up

2026-10-27 archival verification of independent runs completed before interruption. Reviewed only two Qt production-file changes: fixed host plain-language guidance for incomplete/schema/conflict/guest errors, retaining native diagnostic text, and readable queue-full wait/review/Apply guidance with accessible description. No protocol, control, computation or accepted ADR004 source changed. Independently rebuilt app/new Qt suite. Focused genuine process009/013: **2 passed,0failed/skipped**,22.711s; actual error samples include incomplete/format/retained-result guidance. Genuine Qt011/012/015: **5 framework entries passed,0failed/skipped**,21.285s; queue/full repair, conflict explicit retry and field-specific accessibility remain intact. Approval retained and two Qt hashes refreshed. New native020 test independently passed1 case0ignored in4.98s; test-only native hash refresh awaits Review B final corpus approval.

Review B subsequently approved final strict-loader corpus/new native020 additions after independent storage6case and native020 reruns. Refreshed native test-only hash; all21 final reviewed source hashes match. No further production change.
