# Independent Qt / threading / UI trust review

- Reviewer: `/root/review_ui_qt` (no implementation authorship).
- Date: 2026-10-26.
- Base commit: `d7c5887a3a11c61c9e4e0798b8dc9a300e3c5c9f`.
- Status: APPROVED within Qt/bridge review scope, bound to source hashes in
  review-qt.json. This is not ADR acceptance or hosted evidence.

## Coverage

Read AGENTS.md, ADR-004, validation task/matrix, Qt bridge/renderer/tests, native
worker/session/framing and semantic binding code. Reviewed static trusted QML
construction, immutable metadata-only model, GUI-thread ownership, callback
lifetime fencing, finite command/projection transport, draft/commit separation,
exact counters and Qt accessibility instrumentation.

No package QML/source URLs are loaded. QML contains only static reviewed
components; labels/readouts explicitly use PlainText. No component-ID or mortgage
field switch exists in the renderer. Genuine QAccessible interfaces are inspected
by the test, including actual role/name/focus state. The Rust worker retains the
accepted kernel and bus in one process; the UI pipe is a separate host interface.

## Findings and disposition

| ID | Severity | Finding | Current disposition |
|---|---|---|---|
| QT-001 | High | Readout calls Q_INVOKABLE committedValue without a property dependency, so valuesChanged does not refresh its text. Backend already committed1461.48 while widget stayed blank. | Author added trusted Connections.valuesChanged. Closed: fresh independent rebuild/full Qt suite passed. |
| QT-002 | Medium | Wrong-key/stale Shutdown was rejected by Session but reader/processor still stopped the worker. | Author changed termination to accepted-only, with final-frame flush acknowledgment. Closed: independent real-worker wrong-room rejection, refresh continuity, authorized shutdown flush/exit passed. |
| QT-003 | Medium | Fresh Ready accepted arbitrary room/instance key instead of expected launch ownership. | Author added expected-room/instance check. Closed by source review; room/switch isolation suite passed. No separate forged-Ready retest. |
| QT-004 | Medium | Integer projection values beyond i64::MAX round in QJsonValue/QVariant display; schema currently permits them. Text normalization also parses only i64 despite accepting numericu64. | Closed: canonical text projection and i64/u64 text normalization; focused locked unit test passed. |
| QT-005 | Medium | Allowed long labels/title/readouts have no wrapping/fill-width, which can overflow under large text. | Closed: fill-width/wrapping added; genuine alternate-metadata long-label test at 2x text passed. |
| QT-006 | Low | edit/apply stale-epoch branch mutated counter before GUI-thread assertion. | Closed: entry assertions added and reviewed. |
| QT-007 | Medium | Unknown reply kinds fell through as projection frames. | Closed: explicit kind whitelist added and source reviewed; no separate synthetic unknown-kind retest. |

## Independent execution so far

- Local Qt6.11.2 / qmake6 / GCC16; real offscreen QGuiApplication,
  QtQuick software rendering and QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1.
- Desktop QT_QPA_PLATFORMTHEME=gtk3 initially tried to open an unavailable display.
  Test environment explicitly unsets platform-theme/display/GTK variables; this
  changes the test process only, not desktop configuration.
- Original keyboard/accessibility test reproduced QT-001: all three input and
  readout accessible roles/names/focus checks passed; final widget value failed
  despite valid committed projection. Log: target/review-ui-keyboard.txt.
- Three other genuine Qt functional tests passed (5 framework entries including
  init/cleanup,0 failed/skipped,38.039s): UI012/014, UI015/016/017 and UI018.
  Log: target/review-ui-other-qt.txt.
- Independent Qt numeric probe demonstrates18446744073709551615 becoming
  QVariant double/string18446744073709552000;9007199254740993 remains exact qlonglong.
- Separate clean qmake build under target/review-qt-build completed. GCC16 emitted
  system Qt-header SFINAE warnings, not project source errors.

## Final independent execution and scope

Clean independent qmake build in target/review-qt-build completed, then the
rebuilt binary ran the complete Qt suite: **7 passed, 0 failed, 0 skipped**
(5 functional cases plus init/cleanup),296240ms. Archived full log:
independent-qt/full-suite.txt. This includes actual keyboard/tab/accessibility,
committed readout updates, generic alternate metadata, two live room isolation,
package-script rejection, long-label wrapping at2x, teardown, automatic quit save,
exact-pin restore and missing-artifact refusal.

Focused locked integer losslessness test passed (i64 minimum,u64 maximum,
2^53+1 and negative-zero rejection), archived independent-qt/integer.txt.
Independent raw worker protocol verified rejected wrong-room Shutdown leaves
worker usable, followed by successful Refresh and authorized saved reply flushed
before exit0; archived independent-qt/shutdown.json. Earlier readout regression
confirmation is independent-qt/keyboard-fixed.txt.

No unresolved findings in reviewed Qt/bridge scope. Approval does not certify
platform screen readers/GPU rendering, hosted Linux CI, complete installer
durability, or milestone acceptance. Native dynamic system Qt dependencies and
public Qt6 API usage are explicit. Parent must retain Proposed until all other
mandatory gates, independent registry review and hosted evidence succeed.

Source hashes bind the current implementation/tests reviewed. Any change to
these bytes requires re-review; see machine-readable review-qt.json.

## Follow-up test harness review

Approved the test-only wire-reader receive deadline increase from45s to120s and
stage-specific timeout diagnostics after simultaneous test/reviewer load. The
forged-shutdown rejection, diagnostic, subsequent Refresh, saved response and
successful child-exit assertions are intact. This test is functional ownership/
continuity coverage, not a startup-performance gate; production Qt startup60s
remains unchanged. No other source hashes changed. ui.rs hash refreshed in the
sidecar after source inspection; original independent executions above remain
the execution evidence, and the parent reported isolated original45s test passed
in17.94s.
