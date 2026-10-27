# VSLICE003 — Linux offline Operator journey

Status: PROPOSED. Both independent reviews approve03b6ac6; complete local wrapper and successful
hosted run/download verification remain mandatory. No gate is waived.

Base: `cc42020b2db406fdaeb81c3e3fbe6d33fc4a63f9`, merged accepted PR9.
Implementation: `3de262d`; review-driven boundary fixes `0ec4200`; actionable
clarification UX/final frozen source `03b6ac6fc8f347cf314ab6c8816c0ea34f21b243`.
Source-binding receipts must cover that final source. Accepted49/60 ADR005 source
maps remain byte-identical; no accepted WIT/schema/ontology/ABI/lifecycle change.

Primary testing found/fixed two integration gaps: Desktop rematerialization reset
transport view epochs, and identical UI metadata reused editors with the first
room's owner epoch. The new product entry keeps a monotonic outer counter with
strict fail-closed translation and recreates generic view-owned controls during
selection. Original accepted Desktop/bridge/QML sources and all prior gates remain
unchanged. Both product regressions now pass in real Qt/native tests.

Independent reviews identified noncanonical outer-counter acceptance (`01`, `+1`),
missing refresh after reloaded-workspace CAS conflict, and insufficient binding
between successful run API metadata and archived workflow/run identity. Fixes add
lexical checks and negatives, explicit refresh/no-topology reply, and exact run id,
attempt, VS3 workflow path, PR reference, repository, source/head and status binding.
Six provenance tests exercise mismatched successful runs/workflows/attempts and
failed/head-drifted metadata. Both final source-bound reviewers approved03b6ac6 with all24 journey,6native-worker
and6provenance cases passing independently. Review JSONs and hashed logs are archived
next to this ledger. Complete local/hosted wrapper evidence remains pending.

Scope: deterministic offline provider, read-only admitted local catalog and explicit
GBP MortgageTerms normalization; one component in each new empty room, alongside
existing rooms within the accepted16-room snapshot bound. Topology checkpoint and
input initialization/output checkpoint are distinct. A failed latter stage does
not roll topology back; partial status offers explicit direct form repair.
No remote provider/registry, autoapproval/retry, generated code/QML, privileged guest,
new ABI/schema/ontology, populated-room planning or multi-component graph.
Optional external-model trial: NOT RUN (optional; no adapter).

Reproduction command: `bash apps/desktop-qt/validate-vslice-003.sh` with pinned
Rust1.99.1/cargo-component0.21.1, tools/requirements.txt and system Qt6 modules.
The wrapper retains the unchanged OP→VS2→UI→VS1→REG→IPC→ABI chain and separately
checks the locked standalone product manifest. The root Rust regression suite
remains162 tests. New journey suite:24 named cases (22include actual Qt journeys,
25app/worker pairs); native product-worker6 cases; provenance unit6 cases; separate
review gate VS3-025 and downloaded hosted verification VS3-026. Counts here describe
the declared executable inventory; final observed full-wrapper evidence is pending.
