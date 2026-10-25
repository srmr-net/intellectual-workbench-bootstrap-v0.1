# wb-bus

Version 0.1, ADR-002 accepted profile. `ipc::RoomQueue` is a finite process-local room
mailbox; suggested default DEFAULT_ROOM_CAPACITY is 256, with an explicit positive
capacity required. `publish` rejects whole batches with QUEUE_FULL before changing
the queue; it bounds payloads at 16 KiB and deduplicates host request identities.
It assigns canonical message IDs, room identity and monotonic sequence numbers;
callers provide only trusted routes/payloads and internal generation metadata.
The kernel owns authorization, schema checking and destination-edge selection.

`Sequence` is shared by all event/control allocations for a live room. Native
callers serialize control operations with `&mut Room` or their own room mutex;
concurrent producers are ordered by acceptance under that serialization. Restore
in the same Kernel retains the room sequence clock. Durable sequencing across
process restarts is not promised (ADR-004).

UI events are staged/uncommitted (ui_token metadata), retaining FIFO acceptance
order across destinations. `pop_scheduled` finishes committed derived work in
stable DAG order before taking the next staged UI event. Within a propagation
wave routed batches are accepted in graph order; same-source events retain their
order while independent sources may interleave. Sequence orders acceptance, not
wall-clock execution. The token/generation metadata is not serialized in envelopes.
No committed input is silently coalesced, discarded or implicitly retried.

Stop/cancel is a separate synchronous kernel control path, not an enqueue competing
for data capacity. Cancelled queued events are returned for explicit audit failures.
The capacity bounds pending data, not total native process memory. Deduplication
identities and audit history are retained for the controller lifetime; production
retention/resource policy remains future work. No durability, distributed delivery
or exactly-once effects are claimed. Existing prototype validate_envelope is retained.
