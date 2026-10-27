# wb-operator — bounded local Operator

The opt-in `workbench-operator` shell adds request, canonical preview and explicit
consent to the existing Linux desktop. Provider code receives only bounded typed
request/candidate data and returns powerless suggestions. Only the trusted Session
owns a Kernel and mutable rooms; neither escapes. Registry installation, arbitrary
model tools, executable QML, remote providers and guest services are absent.

`OperatorProvider` is a replaceable async interface. `OfflineFixture` recognizes an
explicit tested MortgageTerms/GBP grammar; `DisabledProvider` returns unavailable.
It does not claim general natural-language understanding. Host normalization checks
explicit amounts/rates/years without floats or inferred defaults. Read-only catalog
projection uses admitted semantic capability, actual manifest/schema/UI input fields
and opaque keys; package instructions/labels/paths/history are never provider context.

Suggestions require strict duplicate-free UTF8 JSON, known properties, lexical
integer/string/bool scalars and bounded bytes/items/characters. Supplemental schemas
are agent-plane only. Exact manifest, composition, semantic UI, room, ontology and
WIT contracts remain unchanged. Host constructs the canonical single-instance plan;
input values are distinct typed ingress after accepted `Room::apply_plan`.

Session serializes policy epochs and approval under one host lock. Every revoke,
including absent grants, invalidates previews; fresh consent binds nonce, action,
plan, input/UI digests, room/revision, policy revision and expiry. Wrong/expired/
replayed attempts consume the proposal. Provider polling releases the control-plane
lock while an identity-bound per-room lease remains active. Cancellation drops the
future; adapters must yield and inspect Cancellation before transport initiation.
No remote adapter ships. The default makes zero repair attempts and never retries
apply. Fixed profile limits:32 rooms/requests, one component per empty room,32 kernel
instance/invocation slots; no graph synthesis or resource reservations at preview.

Approval is the commit boundary. Later cancel/revoke serializes after it and never
claims to undo an approved apply. The bounded persistence callback runs under
Session locks: it must perform FileStore I/O only and MUST NOT reenter Session or
wait for a thread using it. The product worker saves topology before initialization,
then saves committed input/output. Failed initialization preserves durable topology
for direct form repair; failed topology persistence offers storage/repreview recovery.
Persistence flags mean acknowledged FileStore commits; an I/O error does not prove
rollback after rename. Refresh and validate visible state rather than infer absence.
No cross-stage atomicity is claimed. Failed apply removes only newly issued consent.

The worker lazily creates Session only for an explicit request in an initial empty
workspace (one existing empty native room). It hands the unchanged version1 durable
checkpoint back to accepted Desktop. Normal edits and cold restore call no provider;
cold display invokes no guest. The first complete edit lazily restores accepted
runtime roots and then applies the edit, so it performs two native guest calls.

`wb-operator-worker` speaks unchanged desktop.1 on stdout and a bounded operator.1
channel on stderr (64KiB replies,20KiB requests,32 queued frames). Qt owns the latter
before event processing; preview/explanation are plain text, fresh consent defaults
off, expiry is host-enforced, and Cancel is disabled after confirmation. Audit is
bounded128 events, replies expose the latest16, and overflow preserves commit stages.
No prompts, provider responses, secrets or approvals enter snapshots/default audit.

Build/check with `bash apps/desktop-qt/validate-operator.sh`. Prerequisites match the
accepted desktop. The wrapper retains every existing gate and adds native, real
worker, genuine Qt process and independent review evidence. See the ADR005 task and
evidence for acceptance status; CI metadata alone never accepts an ADR.
