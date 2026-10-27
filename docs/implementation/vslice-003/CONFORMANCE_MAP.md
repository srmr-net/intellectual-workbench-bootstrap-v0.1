# VS3 executable conformance map

Run `bash apps/desktop-qt/validate-vslice-003.sh` with the documented Linux Qt tools.
All accepted wrappers, test sources and contract bytes remain unchanged. The full
workspace remains162 Rust regression tests; the separate product entry is explicitly
built/tested/Clippy-checked with its own locked manifest. Its real behavior is tested
through the subprocess/Qt boundaries, not fabricated Rust unit-test counts.

The journey suite contains24 independently named cases:22 include genuine Qt product
processes, two directly exercise accepted native adversarial provider boundaries.
Cases019/022/023 have second processes; total25 app/worker pairs. Six additional
native product-worker tests cover exact prior checkpoint preservation, second-room
write failure, CAS drift, stale/future counters and pending cancellation.

| Gate | Executable journey test | Additional evidence |
|---|---|---|
| VS3-001 | `test_vs3_001_real_isolated_process` | Actual Qt/registry/WASM journey artifacts |
| VS3-002 | `test_vs3_002_keyboard_accessible_three_columns` | Actual Qt/registry/WASM journey artifacts |
| VS3-003 | `test_vs3_003_actual_boundary_provider` | Actual Qt/registry/WASM journey artifacts |
| VS3-004 | `test_vs3_004_admitted_readonly_discovery` | Native admitted-only catalog/injection/metadata bounds |
| VS3-005 | `test_vs3_005_opaque_selection_independent_identity` | Renamed package/version is actually admitted and executed |
| VS3-006 | `test_vs3_006_exact_normalization_and_real_payment` | Actual Qt/registry/WASM journey artifacts |
| VS3-007 | `test_vs3_007_clarification_no_defaults` | Actual Qt/registry/WASM journey artifacts |
| VS3-008 | `test_vs3_008_strict_suggestion_boundaries` | Strict parser/size/type/duplicate/fixture native tests |
| VS3-009 | `test_vs3_009_instruction_injection_powerless` | Actual Qt/registry/WASM journey artifacts |
| VS3-010 | `test_vs3_010_exact_preview_without_mutation` | Actual Qt/registry/WASM journey artifacts |
| VS3-011 | `test_vs3_011_canonical_plan_schema_revision` | Native canonical plan/resource validation |
| VS3-012 | `test_vs3_012_reject_and_cancel_preserve_existing_room` | Compare saved-room identity/pin and unchanged committed canvas |
| VS3-013 | `test_vs3_013_confirm_double_click_and_replay_fences` | Native expiry/replay/cross-room tests; real Qt double-click |
| VS3-014 | `test_vs3_014_workspace_drift_revalidation` | Qt pin drift; native policy/pin and workspace CAS tests |
| VS3-015 | `test_vs3_015_kernel_apply_revision_and_audit` | Actual Qt/registry/WASM journey artifacts |
| VS3-016 | `test_vs3_016_real_installed_wasm_committed_canvas` | Actual Qt/registry/WASM journey artifacts |
| VS3-017 | `test_vs3_017_partial_initialization_truthful_repair` | Real guest trap + explicit form repair; native second topology write failure |
| VS3-018 | `test_vs3_018_reactive_edit_zero_further_provider` | Actual Qt/registry/WASM journey artifacts |
| VS3-019 | `test_vs3_019_invalid_edit_retains_last_commit` | Actual invalid edit/trap/bad output preserve checkpoint; retained ui_016_bounded_transport_and_no_projection_loss and Qt ui_015_016_017_fences_queue_room_switch_and_teardown |
| VS3-020 | `test_vs3_020_room_switch_and_rapid_edits` | Actual pending native QML Apply then switch; native old/future counters |
| VS3-021 | `test_vs3_021_two_operator_rooms_isolated` | Two real Operator-created rooms; native concurrent-room isolation |
| VS3-022 | `test_vs3_022_pinned_cold_process_zero_compute` | Provider disabled + IPv4/IPv6 denied; distinct app/worker PIDs |
| VS3-023 | `test_vs3_023_missing_pin_non_destructive_recovery` | Missing pin preserves durable bytes, no substitute |
| VS3-024 | `test_vs3_024_provider_bounds_cancellation_privacy` | Native timeout/unavailable/cancel/queue-ownership/privacy tests |
| VS3-025 | `test_vs3_025_source_bound_independent_reviews` | Collector checks both independent approvals, exact source hashes and zero-exit hashed command logs |
| VS3-026 | `test_vs3_026_complete_downloaded_hosted_evidence` | Separate verifier requires unmodified server ZIP hash and completed successful PR-run provenance; recursively verifies accepted OP/VS2/UI/VS1/REG/IPC/ABI evidence |

Every process report records actual PID/worker PID, app hash, Qt version, initial
physical provider/guest counts, native accessibility/text and stage/projection samples.
Worker logs include canonical pins/revisions/native audit and persistence stages.
Screenshots supplement these facts; offscreen embedded Quick window and shell are
separate captures. Optional external-model trial: NOT RUN (optional, no adapter).

No gate is self-accepted by the collector or a workflow file. Reviews and successful
hosted execution/download verification are separately required. Keep Proposed until
those gates pass; record exact observed totals rather than predicting success.

Review-driven negative evidence: non-canonical outer epoch lexemes are rejected
before and after handoff; valid externally replaced workspace CAS triggers explicit
refresh with no apply. Six provenance unit tests reject mismatched successful run,
attempt, workflow, PR reference and failed/head-drifted metadata. Final hosted
verification binds its API run id/attempt to archived GITHUB_RUN_ID/ATTEMPT as well
as the VS3 workflow path, repository, PR reference, source/head and successful status.
