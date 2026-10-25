# wb-reactive

Version 0.1, ADR-002 accepted profile. `scheduler::Graph` validates endpoint instance
existence and cycles, computes stable topological order with lexical instance-ID
tie breaks, and returns the affected root/descendant set. There is no Operator,
filesystem or module execution in this crate.

`check_fence` requires exact room revision and execution generation equality;
older and future contexts both fail STALE_RESULT. It is separate from the retained
legacy accept_revision prototype. The kernel freezes aggregate input snapshots,
uses graph order within each propagation wave, atomically validates outputs, and
activates descendants only after commit. UI waves are committed FIFO, without
coalescing or retry. Separate rooms may compute concurrently.

The initial kernel profile requires exactly one aggregate input port and one
upstream producer per connected input. Port-level JSON schemas, semantic identifiers
and units must match exactly; more general compatibility is ADR-005 work. Aggregate
JSON can contain multiple fields, all frozen coherently in one accepted WIT message.
This does not add batch/multi-port WIT semantics or prove arbitrary guest determinism.
