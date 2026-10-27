"""Explicit ADR005 verification contract; source changes require renewed review."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
RUST_TESTS=162
WORKER_TESTS=9
GUI_TESTS=6
NATIVE={
 'normalization':'deterministic_fixture_and_explicit_units',
 'missing':'no_defaults_or_injected_instructions',
 'strict':'strict_untrusted_json',
 'cancel':'ambiguity_cancel_unavailable_and_deadline',
 'deadline':'cancellation_and_deadline_rechecked_after_ready',
 'corpus':'positive_and_adversarial_fixture_corpus',
 'factory':'cancelled_and_zero_budget_never_invoke_provider_factory',
 'limits':'op_001_003_007_exact_byte_character_item_and_context_boundaries',
 'discovery':'op_002_003_008_discovery_is_admitted_only_minimal_readonly_and_instruction_free',
 'pins':'op_009_010_offered_keys_actual_fields_schema_and_pin_drift_no_fallback',
 'ambiguity':'op_004_006_semantic_ambiguity_never_selects_arbitrary_release',
 'plan':'op_011_012_invalid_plan_preview_never_grants_or_reserves',
 'compute':'real_admission_preview_approval_ingress_and_replay',
 'partial':'checkpoint_failure_is_honest_partial_completion',
 'consent':'fresh_preview_identity_stale_revision_and_expiry',
 'grant_cleanup':'failed_apply_removes_only_new_execution_consent',
 'policy':'private_session_policy_epoch_revoke_before_grant_and_real_commit',
 'pending':'pending_cancel_queue_release_policy_race_and_room_isolation',
 'rooms':'op_022_concurrent_rooms_keep_intent_approval_and_cancellation_isolated',
 'storage':'committed_guest_then_real_final_write_failure_preserves_topology_checkpoint',
 'queue':'op_007_021_queue_ownership_uses_host_ticket_not_caller_token',
 'wire':'op_007_wire_cannot_supply_host_ticket_or_authority',
 'quota':'op_012_scope_quota_creates_no_guest_or_execution_authority',
 'overflow':'op_018_021_oversized_reply_retains_authoritative_commit_stages',
}
CASES={
 1:['normalization','corpus','limits'],2:['discovery'],3:['discovery','limits'],4:['normalization','ambiguity'],5:['normalization','compute'],6:['missing','ambiguity'],7:['strict','corpus','limits','wire'],8:['discovery','missing','wire'],9:['pins'],10:['pins'],11:['plan','policy'],12:['plan','grant_cleanup','quota'],13:['compute'],14:['consent','compute'],15:['policy','consent','pins'],16:['compute'],17:['compute'],18:['partial','storage','overflow'],19:['compute'],20:['compute'],21:['cancel','deadline','factory','pending','queue','overflow'],22:['rooms','pending'],23:[],24:[],
}
def critical_sources():
 result=[]
 for directory,extensions in [('crates/wb-operator',{'.rs','.toml','.md'}),('apps/desktop-qt/operator',{'.cpp','.h','.pro'}),('apps/desktop-qt/operator-tests',{'.py'}),('docs/implementation/adr-005/schemas',{'.json'}),('fixtures/operator',{'.json'})]:
  result += [str(p.relative_to(ROOT)) for p in (ROOT/directory).rglob('*') if p.is_file() and p.suffix in extensions]
 result += ['Cargo.lock','crates/wb-kernel/src/lib.rs','crates/wb-kernel/src/operator_validation.rs','crates/wb-kernel/src/room.rs','crates/wb-kernel/src/runtime.rs','apps/desktop-qt/validate-operator.sh','apps/desktop-qt/qml/Main.qml','.github/workflows/ci.yml','schemas/component.schema.json','schemas/composition.schema.json','schemas/ui.schema.json','schemas/room.schema.json','ontology/workbench.ttl','wit/component.wit','docs/implementation/adr-005/evidence/verify.py']
 return sorted(set(result))
