# ADR-004 implementation evidence

Status: Accepted — 2026-10-26. Clean locked local checks, all mandatory gates,
two independent reviews and verified successful GitHub-hosted Linux artifacts pass.

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


## Verified local and hosted execution

PR: [#7](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/pull/7).
Reviewed final feature commit: `9f7f2248afa6de469ef5f84cda544a17d56a1445`.
[Successful hosted Linux run40401170134](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40401170134)
tested PR checkout `d29ba8e9d865774ead63300e10e96f320d5a84ba`.
GitHub's synthetic merge tree is identical to the reviewed feature tree
`83a1b4d332ecd0cef01a785fcd0f27e1968ae590`; hosted-checkout.json records that proof.
Later changes are acceptance documentation and archived evidence only.

| Check | Local | GitHub-hosted Ubuntu24.04 x86_64 |
|---|---|---|
| Locked workspace | 117 passed,0 failed/ignored | 117 passed,0 failed/ignored |
| UI-001–020 | all executable/external gates verified | all executable/external gates verified |
| Genuine Qt suite | 7 entries passed,0 failed/skipped | 7 entries passed,0 failed/skipped |
| ABI / IPC / REG / VSLICE gates | 10 /15 /20 /20 pass | 10 /15 /20 /20 pass |
| Financial oracle | 33 cases pass,clean rebuild matches | 33 cases pass,clean rebuild matches |
| Contracts / Python harness | original30 fixtures + new UI corpus,5 tests pass | same pass |
| Rustfmt / locked Clippy -D warnings | pass | pass |
| Independent reviews | Qt/security and state/registry APPROVED | matching source hashes checked by collector |

Native Rust/Cargo1.99.1, Wasmtime49.0.1, cargo-component0.21.1 and guest
wit-bindgen-rt0.41.0 retain accepted pins. Local system Qt6.11.2/GCC16.2.1 and
Python3.14.7; hosted Qt6.4.2/GCC13.3.0 and Python3.11.17. Both use genuine
offscreen Qt Quick software rendering and QAccessible instrumentation.
jsonschema4.26.0, rdflib7.6.0, pySHACL0.30.1 are recorded in run.json. Local
GCC16 emits Qt6.11 header SFINAE warnings; no checks are disabled and hosted Qt6.4
build/tests pass. Offscreen testing does not certify every platform screen reader.

Artifacts, complete hosted logs, API metadata, two reviews and offline verification
are retained here. Local and hosted verification both check110 recorded hashes,
frozen accepted contract/test hashes, all test totals, actual receipts/records,
package files, schema/UI digests, numerical results, PNG and accessibility data.
The native worker uses the same actual mortgage WASM as the headless package in
each environment. Same-machine clean rebuild is required; cross-machine binary
identity is not claimed (as in accepted VSLICE001).

| Artifact | SHA-256 |
|---|---|
| local.zip | `dd82fed3c0206b61ed586c5a9e94238151fc69cf68953d69314eafd0d710d849` |
| hosted.zip / GitHub artifact12689651887 | `31729bf8170026dd79962b65830b8c52a0647f4077867eaca05de24bd3ac9ab0` |
| hosted-logs.zip | `b0dad1401c94047b5ab3fb0895aafdb07590b09565c2cb697ef7a5f586a6765b` |
| Canonical UI schema | `f183fe5d5566d6420fbf6c6ab052a7aa7dc86e41e694a5371551ef51b55c5380` |
| Mortgage UI raw JSON | `b2bfe1b7ed2d83a762ca0bf884b4db6c4cee57af4d6876cf089b487ff428c5c6` |
| Local mortgage WASM | `64987f2704fa0d313ec2529e1e0fd9a0a16b8551cc9e42a352c57f950587cb95` |
| Hosted mortgage WASM | `b045130fe32aef76a0bf30cce9072b7b57ea2cb7b6bad37b23dfaf6a6e7573dd` |

Recheck without extraction using verify.py (with reviewed source files present):

```sh
python3 docs/implementation/adr-004/evidence/verify.py \
  docs/implementation/adr-004/evidence/hosted.zip \
  --sha256 31729bf8170026dd79962b65830b8c52a0647f4077867eaca05de24bd3ac9ab0 \
  --source d29ba8e9d865774ead63300e10e96f320d5a84ba --hosted
```

The initial keyboard test exposed a non-notifying readout accessor, corrected with
an explicit trusted projection connection. Reviews found rejected-shutdown
termination, exact integer display, layout and restore/profile issues; all were
resolved and retested. A final audit found unrelated local drafts cleared by Save
or another field's accepted edit; acknowledgements now cover only submitted fields,
with independent focused Qt and final full local/hosted retests. Early45s wire
harness overload timeouts and the collector's Rust-only gate-index error are
recorded rather than counted as passes. The final120s wire harness preserves every
assertion and product deadlines. Superseded/duplicate CI runs were cancelled; the
final reviewed-source run above completed successfully. No unresolved mandatory
finding remains within this accepted profile.
