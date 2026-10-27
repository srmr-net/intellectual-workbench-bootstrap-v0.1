# ADR005 implementation profile

The supplied kit reuses an unaccepted namespace placeholder. ADR014 retains that
open decision; original ontology bytes are unchanged. ADR009's historical Operator
placeholder is superseded only for this scoped implementation.

Provider DTOs, parser, domain normalization and admitted catalog are in wb-operator.
No component identity/version/digest/path is hardcoded. The explicit tested GBP/
MortgageTerms adapter is separate from the unchanged kernel and generic Qt renderer.
Only fixed semantic vocabulary and actual admitted binding fields enter context;
no RDF literal, package instructions, workspace history, credentials or path does.

Session privately owns Kernel/rooms. Host policy revision changes even for absent
revokes. It fences provider completion and consent against room/policy revisions,
uses identity-bound bounded request leases and consumes exact stored proposals.
The lower-level Proposal/ApplyContext are crate-private. This resolves the initial
provenance/policy-observation findings without changing accepted runtime source.

The scoped planner adds one component to an existing EMPTY native room; populated
rooms are rejected rather than replaced. The UI's first-use profile requires an
initial empty workspace because accepted snapshot1 requires one component per saved
room. No new empty-room snapshot format or multi-component planning is introduced.
Fixed32-room/32-instance ownership and instance='component' bound preview resources;
the unchanged authoritative apply repeats full stage checks and reserves resources.
The new Kernel preview helper is explicitly advisory outside this scoped Session.

The new product entry reuses byte-preserved DesktopShell/Bridge/Main.qml and adds a
native widget panel to the context column. The new worker is desktop.1-compatible;
stdout remains ordinary projections, stderr is bounded operator.1. Providers never
receive either IPC handle. Preview/explanation are plain text. Confirmation is a
fresh checkbox+button gesture; pending approval cannot be cancelled or overwritten
by another UI request, and the watchdog preserves the authoritative outcome.

On confirmation, the worker drops Desktop's writer, reacquires FileStore and checks
workspace revision/checksum/empty scope before mutation. Lock/corruption conflicts
return correlated typed outcomes and never overwrite. The trusted callback persists
topology before accepted typed ingress/real WASM; finalization persists validated
committed state. Partial failure flags distinguish volatile topology, durable topology,
computation and final checkpoint. Direct form repair is offered only for durable
topology; otherwise storage/repreview guidance is shown. No callback reentry is allowed.

The same FileStore then moves into accepted Desktop with its lock intact. Ordinary
edit/restore paths remain unchanged and provider-free. All OP flags, prompts, epochs,
consent, request queues and provider data stay out of snapshots. Cold boot creates no
Session/Kernel/provider; disabled/offline cold display has zero guest/model calls.

Supplemental schemas enforce character/item shapes; Rust additionally enforces4096
UTF8 request bytes,16384 response bytes, unique keys and canonical integer lexemes.
The kit's explanation2048 limit was reconciled to its normative512 Unicode scalars.
Transport audit is a bounded16-event tail of128 host events, and shrinking a reply
never erases authoritative commit-stage fields. No remote adapter or repair retry ships.
