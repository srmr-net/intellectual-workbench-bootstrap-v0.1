# ADR005 unresolved integration requirements

Status: PROPOSED; no acceptance or complete OP gate claims.

The accepted kernel exposes `approve_execution` and `revoke_execution` but no
observable policy revision or transactional/scoped grant. Revoking an absent
approval leaves no observable state. A preview prepared before that revocation
cannot distinguish it from unchanged policy. Unconditionally reapproving at apply
would fail OP015's explicit grant-drift fence. Failed `apply_plan` cleanup now observes preexisting exact approval and removes only a newly issued approval on failure; concurrency/provenance review is still required. Independent runtime
and security reviewers both identified this issue. A policy revision/scoped consent
extension needs explicit design and tests; do not silently reinterpret revocation.

The foundation API also accepts separate Kernel and Room references; controller
provenance must be sealed in a host-owned session before public UI exposure.
Admission-only preview is advisory and does not reserve quota. Actual apply remains
authoritative; preview resource feasibility remains unresolved. Temporary ingress adapters are now revoked after execution. Initialization now
checks every admitted output for a valid committed value at the current revision;
review and adversarial tests remain required.

Still unimplemented: room request coordinator/lease, bounded queue/repair/audit,
Qt request/preview/approval UI, cancellation product path, durable checkpoint handoff,
full OP001–024 mapping and adversarial fixtures, final independent reviews, full
regression/Qt checks and hosted Linux evidence. Existing accepted contracts and
historical evidence have not been amended to hide these missing gates.
