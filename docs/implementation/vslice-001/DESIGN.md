# Headless API and arithmetic mapping

This is product glue over accepted APIs, not a new public protocol or ADR.

1. Registry::admit_directory / resolve_exact issue an opaque handle;
   Kernel::cache_registry_release rehashes that bundle before caching immutable
   verified bytes. Explicit approve_execution names room, instance and exact pin;
   installation never grants execution or native capabilities.
2. New native HeadlessRoom adapter will reuse Room::apply_plan, prepare/start and
   authorize_ui/submit_ui. Those existing UI-named methods are sealed native ingress
   and do not depend on Qt. The CLI actor is headless.host; no Operator is invoked.
   CompositionPlan/RoomSnapshot require UiReference, so use an inert host-owned
   logical reference host.headless/session/ui.json. It is never resolved/rendered
   or included as UI metadata in the package. No registry scope is expanded.
3. Invocation::run exclusively invokes the accepted host; Room::complete validates
   and commits the full candidate. Single-node output is a Room::output projection;
   no fabricated output envelope is emitted when there is no downstream edge.
   Captured input envelope, real dispatch context, correlated audit and exact pin
   provide traceability. Tests may also use a real admitted consumer to inspect
   actual canonical bus fanout.
4. ADR-002 intentionally commits accepted aggregate inputs/revisions before compute
   and invalidates derived projections. Failure preserves the previous output VALUE,
   while valid/error flags and lifecycle may change. VSLICE rollback means no
   candidate/partial output or delivery commit, not byte-identical diagnostic state.
   Snapshot keeps authoritative inputs/pins/topology; restore recomputes outputs
   with a fresh kernel, exact registry reads and fresh explicit native approvals.
5. Arithmetic uses exact bounded BigUint rational cents, not f64 accumulation.
   principal cents <=100000000000; annual percent has <=6 fractional digits and is
   scaled by 1000000. Monthly rate a/b = rate_micro/1200000000, reduced by gcd.
   For n=1..600, payment cents = P*a*(a+b)^n / [b*((a+b)^n-b^n)], or P/n for zero
   rate. Divide once and round half up via 2*remainder >= denominator; only the final
   published cents are rounded. Bounds apply BEFORE exponentiation. The worst tiny-rate/600-month case exceeds the generic 2M spike fuel default.
   The mortgage product caller uses the accepted Limits API with fuel=20,000,000,
   retaining default 8 MiB memory, 100 epoch ticks, message/batch/count limits.
   Host defaults and every accepted ABI limit regression remain unchanged. Both
   reviewers approved workload-specific calibration; all 33 reference cases
   must pass that bounded profile locally and on hosted Linux before adoption.
6. Canonical grammar means unsigned ASCII base-10 with no exponent/leading zero,
   whitespace or plus sign. Fixtures explicitly allow fixed trailing-zero scales
   (5.00,12.0). Principal allows 0..2 fractional digits; rate allows 0..6.
   Range checks are duplicated by schema and guest. Output is exactly two digits
   after '.', permits 0.00 and up to 1083333333.33. Existing wb:MortgageTerms,
   wb:MortgagePayment, wb:CalculateMortgagePayment and wb:GBP provide exact RDF
   meanings/units without adding ontology terms.

No current accepted contract blocks this mapping. Any future discrepancy must be
reported explicitly; accepted tests/semantics will not be weakened for the slice.
