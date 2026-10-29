# VSLICE-004 results — Linux/OpenRouter

**Status: PROPOSED. VS4-001–029 passed; VS4-030 awaits separately supervised live consent and approval.** No live request has been sent for this milestone. Earlier ADR006 live evidence is not reused as VS4-030.

[Draft PR12](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/pull/12). Accepted baseline: master/PR11 `57b76ef9835819b46bcc6b2e778df781b1d1c425`. Final implementation: `c796ca7a459bd66f2fb116cbb255d1d7aa3a2bee`. Reviewed critical source inventory: 25 paths. Tested PR head: `92c30471518d86aaffdca5e70a31d59195b12922`; hosted merge source: `84e1ca67fd9a9a922510b2e2f6018bf11f63367a`. Later archive/ledger bookkeeping does not change those reviewed source bytes. Accepted WIT, schemas, ontology, registry scope, lifecycle, bus, approval and persistence contracts remain unchanged.

## Gate ledger

Each executable below passed in the clean final-source local wrapper and GitHub-hosted Linux wrapper, without skips. Expected behavior is the corresponding unchanged `TEST_MATRIX.md` row. Proof for every row: the named result in `vs4/process-tests.txt` and actual `vs4/process/<test-name>/run-*/` artifacts inside both archives; additional framed native adversaries are in `vs4/control/`. Full commands and source bindings follow.

| Gate | Executable / procedure | Observed | Status |
|---|---|---|---|
| VS4-001 | `test_vs4_001_fresh_installed_desktop` | Named check passed in local and hosted logs | PASS |
| VS4-002 | `test_vs4_002_offline_default_absent_key` | Named check passed in local and hosted logs | PASS |
| VS4-003 | `test_vs4_003_private_source_control` | Named check passed in local and hosted logs | PASS |
| VS4-004 | `test_vs4_004_truthful_outbound_data` | Named check passed in local and hosted logs | PASS |
| VS4-005 | `test_vs4_005_withheld_denied_withdrawn` | Named check passed in local and hosted logs | PASS |
| VS4-006 | `test_vs4_006_assistive_consent_state` | Named check passed in local and hosted logs | PASS |
| VS4-007 | `test_vs4_007_actual_adapter_transport` | Named check passed in local and hosted logs | PASS |
| VS4-008 | `test_vs4_008_progress_cancel` | Named check passed in local and hosted logs | PASS |
| VS4-009 | `test_vs4_009_exact_admitted_selection` | Named check passed in local and hosted logs | PASS |
| VS4-010 | `test_vs4_010_adversarial_suggestions` | Named check passed in local and hosted logs | PASS |
| VS4-011 | `test_vs4_011_injection_no_authority` | Named check passed in local and hosted logs | PASS |
| VS4-012 | `test_vs4_012_structured_preview` | Named check passed in local and hosted logs | PASS |
| VS4-013 | `test_vs4_013_separate_genuine_approval` | Named check passed in local and hosted logs | PASS |
| VS4-014 | `test_vs4_014_reject_replay_double_click` | Named check passed in local and hosted logs | PASS |
| VS4-015 | `test_vs4_015_revision_pin_fences` | Named check passed in local and hosted logs | PASS |
| VS4-016 | `test_vs4_016_real_kernel_wasm` | Named check passed in local and hosted logs | PASS |
| VS4-017 | `test_vs4_017_partial_failure_safe_form_repair` | Named check passed in local and hosted logs | PASS |
| VS4-018 | `test_vs4_018_committed_reference_output` | Named check passed in local and hosted logs | PASS |
| VS4-019 | `test_vs4_019_reactive_without_inference` | Named check passed in local and hosted logs | PASS |
| VS4-020 | `test_vs4_020_rapid_edits_and_rollback` | Named check passed in local and hosted logs | PASS |
| VS4-021 | `test_vs4_021_provider_errors_preserve_commit` | Named check passed in local and hosted logs | PASS |
| VS4-022 | `test_vs4_022_switch_fences_and_isolation` | Named check passed in local and hosted logs | PASS |
| VS4-023 | `test_vs4_023_pinned_cold_process_offline` | Named check passed in local and hosted logs | PASS |
| VS4-024 | `test_vs4_024_offline_after_live_disabled` | Named check passed in local and hosted logs | PASS |
| VS4-025 | `test_vs4_025_keyboard_focus_accessible_status` | Named check passed in local and hosted logs | PASS |
| VS4-026 | `test_vs4_026_secret_safe_artifacts_and_negative_scan` | Named check passed in local and hosted logs | PASS |
| VS4-027 | Independent security/privacy review | APPROVED, exact source map and logged retests | PASS |
| VS4-028 | Independent desktop/runtime review | APPROVED, findings fixed and retested | PASS |
| VS4-029 | Successful hosted Linux run + downloaded archive verification | All26 VS4 cases,42 process pairs, retained regression chain; two independent verifier executions | PASS |
| VS4-030 | Supervised real OpenRouter desktop walkthrough | NOT RUN; user readiness requested; fresh outbound consent and separate exact proposal approval required | OPEN |

## Commands and actual execution

```sh
PATH="$PWD/target/vs2-venv/bin:/tmp/wb-abi-tools/bin:$PATH" bash apps/desktop-qt/validate-vslice-004.sh
python3 docs/implementation/vslice-004/evidence/verify.py docs/implementation/vslice-004/evidence/local.zip --sha256 b926aef869a044b5a05f6f6e872adc2174b60e65659e164ad287da8f349bd7ff --source 92c30471518d86aaffdca5e70a31d59195b12922
python3 docs/implementation/vslice-004/evidence/verify.py docs/implementation/vslice-004/evidence/hosted.zip --sha256 9734df51adfa6bc1463a52455c1a040176b98b5bcedd7c02de859e79a9c2f328 --source 84e1ca67fd9a9a922510b2e2f6018bf11f63367a --hosted --run-metadata docs/implementation/vslice-004/evidence/hosted-run.json --head 92c30471518d86aaffdca5e70a31d59195b12922
```

Full wrapper and both archive verifiers exited0. Local execution required permission outside the agent sandbox because unchanged REG-003 creates a Unix socket; the initial sandbox denial was not worked around by weakening the test. Final wrapper retains root fmt,162 Rust regression tests,Clippy with warnings denied,Python contract checks,all ABI/IPC/REG/UI/OP/LLM/VS1–3 gates,plus2 default and2 mock native tests,2 negative provenance tests and26 VS4 tests. There are42 distinct actual app/worker process pairs. Hosted VS4 tests took287.643s. Each process records PIDs,binary hash,counters,Qt events and committed projections. The cold-process test records exact restored pins/state,denied IPv4/IPv6 and zero provider/guest calls solely for display.

## Hosted identity, versions and immutable archives

Successful [run40857580528](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40857580528),attempt1,workflow `.github/workflows/vslice-004.yml`,GitHub-hosted Ubuntu24.04/Linux/X64,pull-request12 merge. See `hosted-run.json`,`hosted-jobs.json`,`hosted-artifacts.json` and `hosted-logs.zip`. Native root and all retained workflows also completed successfully on the tested head.

| Artifact | Bytes | SHA256 |
|---|---:|---|
| [local.zip](local.zip) | 95939413 | `b926aef869a044b5a05f6f6e872adc2174b60e65659e164ad287da8f349bd7ff` |
| [hosted.zip](hosted.zip),GitHub artifact12891632354 | 92597646 | `9734df51adfa6bc1463a52455c1a040176b98b5bcedd7c02de859e79a9c2f328` |

The hosted hash matches GitHub's artifact API digest. `local-verification.json`,`hosted-verification.json` and independently produced `hosted-independent-verification.json` verify all evidence-file hashes,exact Git source inventories,review receipts/logs,run/head/merge identity and retained regression archives. Binary and fixture hashes are in each archive's `run.json` and module reports.

Hosted: Rust/cargo1.99.1,cargo-component0.21.1,Wasmtime49.0.1,Qt6.4.2,Python3.11.17,jsonschema4.26.0,rdflib7.6.0. Local: same Rust/Wasmtime/component tooling,Qt6.11.2,Python3.14.7. Toolchain commands and complete version strings are archived.

## Independent reviews and unsuccessful attempts

Security reviewer `/root/vs4_security_review`: APPROVED c796ca7; final independent2 native,2 provenance and strengthened025 tests passed; earlier six-case security run retained. Desktop reviewer `/root/vs4_desktop_review`: APPROVED c796ca7; final008/012/023/025 passed36.615s,zero skips. Receipts bind all25 critical hashes and executed log hashes; historical logs and actual screenshots/process reports are retained under `reviewer-logs/`.

Resolved desktop findings: distinct second-room isolation coverage; bringing progress/Cancel/proposal into the actual viewport without focus/consent activation; reserving status layout so elapsed updates do not push progress offscreen. Rapid edits preserve accepted conflict/draft behavior and require explicit Apply recovery. Initial initialization traps truthfully retain installed topology and offer typed form repair without another proposal or provider call. An early cold-edit timeout during concurrent compilation passed an unchanged stable rerun; no timeout was weakened.

[Hosted attempt40847524106](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40847524106) passed25/26 new cases but failed025 because Qt6.4 offscreen accessibility was inactive. Its logs/API metadata are archived as `hosted-attempt1-*`. [Qt6.4 source](https://github.com/qt/qtbase/blob/v6.4.2/src/gui/accessible/qaccessible.cpp) shows public setActive only notifies observers. Explicit validation now activates the actual QPA backend through matching QtGui private headers;025 additionally asserts backend activation and still requires genuine delivered Alert events. No notification/counter is fabricated and ordinary application startup does not enter this test-client setup. The final successful hosted run confirms the fix on Qt6.4.2. Private headers must match the Qt build; event/focus tests are not a claim of human screen-reader certification.

## Authority, privacy and boundaries

Outbound review binds exact utterance/public candidate fields,fixed instructions/schema/settings,current durable workspace and exact immutable pin/UI set. Future selectors cannot be known before accepted Session creates its nonce: explicit placeholders disclose host-issued digest-derived opaque selectors generated at send. Actual fields,order and multiplicity are rechecked before forwarding. This is not a byte-identical future-request claim. Immutable registry checks protect component input schemas; the additional explicit fingerprint covers UI/schema identity.

Credentials remain in the approved private host source. Qt accepts and clears a file reference,never key contents; metadata availability is not authentication. Mocked tests use actual owner-only synthetic credential files,invalid-mode/source-change cases and secret-bearing error negatives. No live credential or inference runs in CI. The complete decompressed evidence is scanned for the private synthetic marker by the verifier. The guest receives no new privileges.

Topology,durable topology checkpoint,typed ingress/calculation and output checkpoint remain separate. No automatic topology retry or WASM-to-Qt shortcut is introduced. Only validated committed results display. Exact-pin cold restoration requires no provider or guest call solely to paint saved state.

## Live gate and limitations

**VS4-030 is unrun.** A separate readiness question is pending. The actual visible Linux app must first show its current outbound review; user inference consent must be fresh and review-digest-bound,then a successful suggestion needs separate exact proposal approval. The supervised harness is forbidden in CI and sends at most one synthetic request,512 output tokens,20 seconds,no retries/tools/fallback. It then tests real1461.48 output,reactive1753.77 with unchanged provider count,pinning and a separate offline process. No prior ADR006 approval or live result substitutes for this gate.

Configured model: `nvidia/nemotron-3-ultra-550b-a55b:free`,OpenRouter/NVIDIA only. [Current public provider metadata](https://openrouter.ai/nvidia/nemotron-3-ultra-550b-a55b:free),checked2026-10-29,continues to disclose request logging and no response_format enforcement;strict host validation remains mandatory. `public-provider-check.json` is metadata evidence,not inference. Actual returned model/version,usage,latency/cost are **not available for VS4-030 until it runs**; unavailable fields will not be invented. No other provider/model/platform is accepted by this milestone.

No accepted-contract blocker was found. Deferred scope remains multi-component planning,remote stores/registries,guest permissions,arbitrary QML,generation and full database/power-loss recovery. Milestone and PR remain Proposed/draft until VS4-030 is evidenced; no final acceptance is claimed.

## User-directed deferral2026-10-29

The user stopped the confusing scripted walkthrough and deferred further UX/consent ceremony. The first live-stage review expired; the second harness rejected an already-checked consent box. Both recorded zero provider attempts and no topology apply. VS4-030 remains unpassed and this milestone remains Proposed. Work continues through the simpler [direct Operator entry](../../operator-direct/RESULTS.md): one session provider opt-in, Ask, and one actual Start action, preserving host validation and accepted runtime contracts. No further scripted acceptance prompts are required for ordinary exploratory use.
