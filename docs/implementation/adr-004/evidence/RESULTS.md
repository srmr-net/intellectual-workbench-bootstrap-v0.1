# ADR-004 implementation evidence

Status: PROPOSED until clean locked local checks, all mandatory gates, independent
reviews and verified successful GitHub-hosted Linux artifacts are archived.

Base: merged accepted VSLICE001 PR6, d7c5887a3a11c61c9e4e0798b8dc9a300e3c5c9f.
Reproduce with `bash apps/desktop-qt/validate.sh` and the toolchain/dependencies in
apps/desktop-qt/README.md. The outer wrapper builds real WASM/UI fixtures before
retaining unchanged VSLICE/REG/IPC/ABI wrappers. It adds native UI and genuine Qt
offscreen tests, screenshots, QAccessible instrumentation, metadata, receipts,
immutable records, hashes and source-bound review gates. No skipped mandatory test
is accepted. Hosted acceptance is checked outside the collector.

## Indexed executable coverage

All native tests: crates/wb-ui-host/tests/ui.rs (and its integer unit test in src/lib.rs).
All genuine GUI tests: apps/desktop-qt/tests/UiConformance.cpp.

| Gate | Native case / actual Qt method |
|---|---|
| UI-001 | ui_001 strict duplicate-safe schema/version; Python positive/malicious corpus |
| UI-002 | ui_002_010 executable/unknown/resources; Qt malicious admission produces no controls |
| UI-003 | ui_003 bounds/IDs/labels; Qt large-text long-label metadata |
| UI-004 | ui_004 directions and actual manifest authority |
| UI-005 | ui_005 fields/kinds/escapes/refs; lossless i64/u64 display unit test |
| UI-006 | ui_006 exact RDF/interface reconciliation; ui.json basename profile |
| UI-007 | ui_007 actual opt-in immutable admission/overwrite/tamper; raw record archive |
| UI-008 | ui_008 headless None/idempotence/mixed store; full unchanged headless suite executes real package |
| UI-009 | Qt ui_009_011 + ui_009_010_017 alternate IDs/labels from real installed metadata |
| UI-010 | native executable corpus + Qt malicious metadata rejected before controls |
| UI-011 | Qt ui_009_011 keyboard/Tab/QAccessible/focus/theme/scale/screenshots |
| UI-012 | Qt ui_012_014 + teardown tests; real framed native worker test |
| UI-013 | ui_013 invalid/incomplete draft no compute/revision; full typed ingress |
| UI-014 | ui_014 + Qt ui_012_014: real registry/kernel/bus/WASM committed1461.48 and500.00 |
| UI-015 | ui_015 expected revision/generation; Qt ui_015_016_017 stale projection/view epoch; unchanged IPC/VSLICE late completion gates |
| UI-016 | native bounded framing; actual Qt queue capacity2/reject/explicit Apply, committed replies retained |
| UI-017 | native ownership/forged fields/shutdown continuity; Qt two live rooms distinct values and cross-room projection rejection |
| UI-018 | native exact UI/pin restore; Qt saved then newer unsaved edit quit/reopen, missing object visibly unavailable |
| UI-019 | Independent review-qt and review-state reports/JSON, reviewed source hashes and independent actual executions |
| UI-020 | Full outer locked wrapper + hosted Linux success/download verification; cannot be self-proved by test name |

## Scope and unresolved decisions

Linux trusted native host with local private registry directories and dynamic system
Qt, not a privileged guest UI service. One form, 1–32 controls, one aggregate input,
ui.json basename, direct scalar schema types. Units are literal hints, no currency
conversion. u64 counters and integer displays use exact text. A same-generation
revision conflict retains drafts until explicit Apply; no automatic retry.

The supplied kit repurposes the original unresolved ADR004 storage number. ADR012
preserves that open database/recovery decision. Other unresolved decisions remain
namespace permanence, richer UI/locale semantics, multi-platform distribution and
platform screen-reader certification. Offscreen roles/keyboard/screen captures are
real tests but not certification of every desktop assistive technology. Snapshot
file fsync+rename is bounded logical reopen support, not physical power-cut proof.
Accepted WIT, manifest schema, ontology and financial computation remain unchanged.
