# VSLICE-004 results — evidence ledger

**Status: PROPOSED — validation in progress.**

Baseline: accepted ADR001–006 and VS1–3, merged PR11/master `57b76ef9835819b46bcc6b2e778df781b1d1c425`. Branch: `vslice-004-live-ux`. Accepted implementations, WIT, schemas and ontology are byte-preserved. Product additions reuse the ADR006 adapter and accepted registry/kernel/guest/renderer/storage.

## Gate ledger

Command for executable cases: `REGISTRY_PYTHON=target/vs2-venv/bin/python target/vs2-venv/bin/python apps/desktop-qt/vslice-004-tests/process_cases.py`; reproducible full chain: `PATH="$PWD/target/vs2-venv/bin:/tmp/wb-abi-tools/bin:$PATH" bash apps/desktop-qt/validate-vslice-004.sh`.

| Gate | Executable/procedure | Status | Evidence |
|---|---|---|---|
| VS4-001 | `test_vs4_001_fresh_installed_desktop` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_001_fresh_installed_desktop/` in archive |
| VS4-002 | `test_vs4_002_offline_default_absent_key` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_002_offline_default_absent_key/` in archive |
| VS4-003 | `test_vs4_003_private_source_control` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_003_private_source_control/` in archive |
| VS4-004 | `test_vs4_004_truthful_outbound_data` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_004_truthful_outbound_data/` in archive |
| VS4-005 | `test_vs4_005_withheld_denied_withdrawn` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_005_withheld_denied_withdrawn/` in archive |
| VS4-006 | `test_vs4_006_assistive_consent_state` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_006_assistive_consent_state/` in archive |
| VS4-007 | `test_vs4_007_actual_adapter_transport` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_007_actual_adapter_transport/` in archive |
| VS4-008 | `test_vs4_008_progress_cancel` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_008_progress_cancel/` in archive |
| VS4-009 | `test_vs4_009_exact_admitted_selection` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_009_exact_admitted_selection/` in archive |
| VS4-010 | `test_vs4_010_adversarial_suggestions` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_010_adversarial_suggestions/` in archive |
| VS4-011 | `test_vs4_011_injection_no_authority` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_011_injection_no_authority/` in archive |
| VS4-012 | `test_vs4_012_structured_preview` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_012_structured_preview/` in archive |
| VS4-013 | `test_vs4_013_separate_genuine_approval` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_013_separate_genuine_approval/` in archive |
| VS4-014 | `test_vs4_014_reject_replay_double_click` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_014_reject_replay_double_click/` in archive |
| VS4-015 | `test_vs4_015_revision_pin_fences` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_015_revision_pin_fences/` in archive |
| VS4-016 | `test_vs4_016_real_kernel_wasm` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_016_real_kernel_wasm/` in archive |
| VS4-017 | `test_vs4_017_partial_failure_safe_form_repair` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_017_partial_failure_safe_form_repair/` in archive |
| VS4-018 | `test_vs4_018_committed_reference_output` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_018_committed_reference_output/` in archive |
| VS4-019 | `test_vs4_019_reactive_without_inference` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_019_reactive_without_inference/` in archive |
| VS4-020 | `test_vs4_020_rapid_edits_and_rollback` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_020_rapid_edits_and_rollback/` in archive |
| VS4-021 | `test_vs4_021_provider_errors_preserve_commit` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_021_provider_errors_preserve_commit/` in archive |
| VS4-022 | `test_vs4_022_switch_fences_and_isolation` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_022_switch_fences_and_isolation/` in archive |
| VS4-023 | `test_vs4_023_pinned_cold_process_offline` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_023_pinned_cold_process_offline/` in archive |
| VS4-024 | `test_vs4_024_offline_after_live_disabled` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_024_offline_after_live_disabled/` in archive |
| VS4-025 | `test_vs4_025_keyboard_focus_accessible_status` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_025_keyboard_focus_accessible_status/` in archive |
| VS4-026 | `test_vs4_026_secret_safe_artifacts_and_negative_scan` | Final-source run pending | `vs4/process-tests.txt`, `vs4/process/test_vs4_026_secret_safe_artifacts_and_negative_scan/` in archive |
| VS4-027 | Independent security/privacy review | Pending source-bound final signoff | `review-security.json` |
| VS4-028 | Independent desktop/runtime review | Pending source-bound final signoff | `review-desktop.json` |
| VS4-029 | Full successful hosted Linux run + downloaded archive verification | Pending | Run/API/source/hash ledger to follow |
| VS4-030 | Separately supervised visible OpenRouter journey | NOT RUN | Requires fresh disclosure consent and separate proposal approval |

## Actual execution so far

Development native tests: 2 passed; all-feature Clippy passed. First complete Qt sweep: 24/26 passed; rapid-edit test expected serial completion but accepted revision fencing retained a draft, and focus traversal instrumentation needed correction. These are not acceptance passes. The edit test now exercises explicit trusted Apply on conflict; separate-room isolation was strengthened following independent review. Final clean source runs are pending.

## Implementation and privacy

Remote disclosure binds exact utterance/public candidate fields/fixed instructions/schema/settings and current durable workspace/exact immutable pin/UI set. Future candidate selectors cannot be known before accepted Session creates its nonce: placeholders explicitly disclose the fresh host-issued digest-derived opaque selectors generated at send. Actual fields/order/multiplicity are rechecked before forwarding. This is not a claim of byte-identical future outbound bodies. Accepted immutable registry checks protect component input schema; the explicit additional fingerprint covers UI/schema identity.

Credentials remain host-controlled; the UI accepts a private file reference, clears it on Apply, and never receives key contents. Metadata availability is not authentication. Real owner-only synthetic key files exercise the accepted loader in mocked tests, including invalid-mode and source-change negatives. No actual credential or provider request runs in CI. Synthetic secret-bearing error fixtures must remain absent from submitted artifacts, including decompressed ZIP entries.

Topology, durable topology checkpoint, typed ingress/calculation and output checkpoint remain separate. Partial initialization failure keeps the installed form available for local repair; no topology auto-retry. Cold display is validated in a new app/worker process with denied network and zero provider/guest display calls.

## Reviews, versions, logs and hashes

Independent reviewers: `/root/vs4_security_review`, `/root/vs4_desktop_review`. Initial source inspection found no contract blocker. Desktop reviewer requested distinct second-room isolation coverage; implemented, retest pending. Complete findings/signoff and source-bound logs will be recorded before final CI. Local toolchain: Rust1.99.1, Qt6.11.2; definitive full-chain `run.json` and archive hashes pending.

## Hosted and live evidence

No new hosted run or VS4 live acceptance is claimed yet. Earlier ADR006 live evidence is not VS4-030. The new supervised live harness preserves the Linux display, waits for a fresh review-digest-bound consent, then waits separately for exact proposal approval, and exercises real WASM/reactivity/offline cold restart. It is forbidden in CI. Exact actual model/version/usage/latency/cost will be recorded where the accepted adapter exposes them; unavailable fields will be stated.

## Limitations

Linux/OpenRouter scope only; no acceptance for other models/providers/platforms. Qt event/focus tests do not substitute for a human assistive-technology assessment. No full database or power-loss recovery. Milestone stays Proposed until every mandatory test, both reviews, final-source hosted and separately supervised live gate pass.
