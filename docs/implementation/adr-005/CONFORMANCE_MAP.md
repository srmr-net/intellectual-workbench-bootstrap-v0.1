# OP001–024 executable conformance map

Default command: `bash apps/desktop-qt/validate-operator.sh`. It retains the unchanged accepted VS2 → UI → VS1 → REG → IPC → ABI wrappers, locked workspace tests/Clippy, formatting and Python validators. No mandatory test is ignored/skipped.

Native names below run under `cargo test --locked -p wb-operator`; the wrapper executes them as part of the entire workspace. Product commands are `python3 apps/desktop-qt/operator-tests/worker_cases.py` (9 cases) and `python3 apps/desktop-qt/operator-tests/process_cases.py` (6 genuine Qt cases /7 app-worker pairs).

| Gate | Named native evidence | Additional product/evidence check |
|---|---|---|
| OP-001 | `deterministic_fixture_and_explicit_units`<br>`positive_and_adversarial_fixture_corpus`<br>`op_001_003_007_exact_byte_character_item_and_context_boundaries` | Strict typed schemas and positive/negative JSON corpus |
| OP-002 | `op_002_003_008_discovery_is_admitted_only_minimal_readonly_and_instruction_free` | Only admitted UI-bearing packages; no install from request |
| OP-003 | `op_002_003_008_discovery_is_admitted_only_minimal_readonly_and_instruction_free`<br>`op_001_003_007_exact_byte_character_item_and_context_boundaries` | UTF8/context/item bounds; redacted audit tail |
| OP-004 | `deterministic_fixture_and_explicit_units`<br>`op_004_006_semantic_ambiguity_never_selects_arbitrary_release` | Actual registry pin selected by opaque semantic key |
| OP-005 | `deterministic_fixture_and_explicit_units`<br>`real_admission_preview_approval_ingress_and_replay` | Real committed1461.48; direct1753.77 |
| OP-006 | `no_defaults_or_injected_instructions`<br>`op_004_006_semantic_ambiguity_never_selects_arbitrary_release` | Genuine clarification/decline and semantic ambiguity |
| OP-007 | `strict_untrusted_json`<br>`positive_and_adversarial_fixture_corpus`<br>`op_001_003_007_exact_byte_character_item_and_context_boundaries`<br>`op_007_wire_cannot_supply_host_ticket_or_authority` | Native wire cannot supply authority/ticket; oversized requests |
| OP-008 | `op_002_003_008_discovery_is_admitted_only_minimal_readonly_and_instruction_free`<br>`no_defaults_or_injected_instructions`<br>`op_007_wire_cannot_supply_host_ticket_or_authority` | Actual metadata injection excluded; genuine malicious request has0rooms |
| OP-009 | `op_009_010_offered_keys_actual_fields_schema_and_pin_drift_no_fallback` | Worker exact-pin tamper fails before invocation |
| OP-010 | `op_009_010_offered_keys_actual_fields_schema_and_pin_drift_no_fallback` | Actual schema/UI fields and GBP semantic unit policy |
| OP-011 | `op_011_012_invalid_plan_preview_never_grants_or_reserves`<br>`private_session_policy_epoch_revoke_before_grant_and_real_commit` | Canonical unchanged plan schema, exact UI/pin and revision |
| OP-012 | `op_011_012_invalid_plan_preview_never_grants_or_reserves`<br>`failed_apply_removes_only_new_execution_consent`<br>`op_012_scope_quota_creates_no_guest_or_execution_authority` | Cycles/edges/permissions/missing resources/duplicate topology rejected; bounded32-room scope |
| OP-013 | `real_admission_preview_approval_ingress_and_replay` | Genuine preview0rooms, unchecked consent, accessible plaintext fields |
| OP-014 | `fresh_preview_identity_stale_revision_and_expiry`<br>`real_admission_preview_approval_ingress_and_replay` | Genuine fresh approval, decline, intent-edit invalidation and approve/cancel boundary |
| OP-015 | `private_session_policy_epoch_revoke_before_grant_and_real_commit`<br>`fresh_preview_identity_stale_revision_and_expiry`<br>`op_009_010_offered_keys_actual_fields_schema_and_pin_drift_no_fallback` | Policy epoch conflict, pin drift, corrupt snapshot and deterministic writer-lock collision |
| OP-016 | `real_admission_preview_approval_ingress_and_replay` | Real native kernel audit and topology checkpoint |
| OP-017 | `real_admission_preview_approval_ingress_and_replay` | Real installed pure WASM through typed ingress; no direct renderer shortcut |
| OP-018 | `checkpoint_failure_is_honest_partial_completion`<br>`committed_guest_then_real_final_write_failure_preserves_topology_checkpoint`<br>`op_018_021_oversized_reply_retains_authoritative_commit_stages` | Guest trap, actual FileStore write failure, host quota trap and genuine form repair |
| OP-019 | `real_admission_preview_approval_ingress_and_replay` | Genuine direct edit; physical provider count remains1 |
| OP-020 | `real_admission_preview_approval_ingress_and_replay` | Separate offline cold process, exact saved pins, provider0/guest0 |
| OP-021 | `ambiguity_cancel_unavailable_and_deadline`<br>`cancellation_and_deadline_rechecked_after_ready`<br>`cancelled_and_zero_budget_never_invoke_provider_factory`<br>`pending_cancel_queue_release_policy_race_and_room_isolation`<br>`op_007_021_queue_ownership_uses_host_ticket_not_caller_token`<br>`op_018_021_oversized_reply_retains_authoritative_commit_stages` | Pending cancel/timeout/queue release, reply byte bounds and preserved commit stages |
| OP-022 | `op_022_concurrent_rooms_keep_intent_approval_and_cancellation_isolated`<br>`pending_cancel_queue_release_policy_race_and_room_isolation` | Concurrent rooms, cross-room digest rejection and audit privacy |
| OP-023 |  | Independent review-agent-security.json + review-runtime-integration.json; source hashes and commands checked by collect_evidence.py |
| OP-024 |  | Successful hosted Linux run and downloaded archive verified by evidence/verify.py with run metadata; baseline138 + new24 =162 Rust tests |

Exit statuses, logs, exact tool versions, source/binary hashes, review commit and hosted run URL belong in evidence/RESULTS.md and the generated artifact report. The collector records execution; it cannot self-accept an ADR or turn a workflow declaration into hosted success. OP024 is completed only by the separate downloaded-artifact verifier after successful hosted execution.

Profile: one installed pure component in one existing empty native room, initially empty workspace UI; explicit MortgageTerms/GBP normalization and deterministic offline provider. No general NL inference, populated-room replacement, new ABI/guest privilege, new schema/ontology field, remote registry or power-loss database recovery.
