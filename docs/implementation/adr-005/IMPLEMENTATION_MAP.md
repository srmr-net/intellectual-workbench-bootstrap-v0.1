# ADR005 intended implementation profile and reconciliations

Design notes, not a completion claim. See BLOCKERS.md for current implemented scope
and missing gates. Qt/RPC/durable handoff described below are not implemented.

The kit replaces the old unaccepted namespace ADR005 placeholder. Namespace/SHACL
permanence stays unresolved (ADR014); accepted ontology bytes are untouched. ADR009
historical Operator placeholder is superseded only for this scoped decision.

Accepted VS2 persistence supports one component per room. This profile targets a
host-created EXISTING EMPTY kernel room; a populated room returns UNSUPPORTED_REQUEST
without replacing/appending topology. No multi-component planning or snapshot format
extension. Real apply uses Room.apply_plan; initialization is separate submit_ui /
run_until_idle. A topology-only checkpoint is saved before initialization; failure
reports partial workflow honestly, retains topology and offers ordinary form repair.

New Kernel admission-only validation checks canonical plan/schema/registered resources/
ports/graph without reserving instances, advancing generations or granting execution.
The unchanged apply_plan repeats authoritative stage/approval checks at commit.
Existing accepted renderer, DesktopShell/Bridge, Desktop worker, storage, tests and
collectors remain byte-preserved. A new worker is desktop.1 compatible and uses its
separate stderr channel for bounded operator.1 RPC; a small trusted panel owns that
channel before event processing. Normal stdout remains accepted desktop projections.
No socket/remote endpoint, shell command, model tool or guest service is exposed.

After Operator application, the worker hands the unchanged format1 checkpoint to
accepted Desktop. Ordinary edits and cold restore delegate directly to accepted
VS2 code, with no provider call. Runtime approval/proposals/prompts are never saved.

Provider is async/replaceable, but only deterministic fixture/disabled adapters ship.
The explicit tested MortgageTerms/GBP parameter adapter handles grouping, percent and
year-to-month arithmetic; it is not a kernel/Qt/component-ID special case. No general
NL accuracy claim. Catalog uses allowlisted semantic IRIs and manifest/schema keys,
never arbitrary RDF/README literals. Ambiguity never chooses a similar release.

Supplemental schemas remain agent-plane-only. Enforce4096 UTF8 bytes separately
from character limits and reduce explanation maxLength2048 in kit to normative512.
