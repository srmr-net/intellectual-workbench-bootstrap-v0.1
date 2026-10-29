# VS4-001 — Live Operator Linux product experience

- **Goal:** Implement VS4-001–026 executable journeys and record VS4-027–030
  reviews, hosted and separately supervised live evidence; remain Proposed until all pass.
- **Owner:** Codex; independent security/privacy and desktop/runtime reviewers.
- **Baseline:** merged PR11, master57b76ef9835819b46bcc6b2e778df781b1d1c425.
- **Contract versions:** accepted ADR001–006; VS1–3; existing WIT, manifest,
  ontology, composition/UI/room and Operator suggestion/approval unchanged.
- **Owned paths:** additive apps/desktop-qt/vslice-004/, vslice-004-tests/,
  validate-vslice-004.sh, new workflow, this kit/evidence, status/backlog bookkeeping.
- **Forbidden paths:** accepted implementation/source-bound receipts, schemas,
  ontology, WIT, references, guest and kernel functionality. Reuse libraries.
- **Dependencies:** existing accepted registry/kernel/renderer/storage and real
  compiled UI-bearing mortgage package; accepted ADR006 LiveProvider/transport.
- **Inputs:** supplied kit/scenarios; private isolated test profiles; fake owner-only
  credential source and injected raw API transport in CI; never live keys.
- **Changes:** product bootstrap/launcher, secure host source configuration,
  bound outbound-data review before consent, structured approval, progress/alerts,
  typed error/partial repair, executable real-process witnesses.
- **Acceptance commands:** bash apps/desktop-qt/validate-vslice-004.sh (outer full
  unchanged ADR006→VS3→OP→VS2→UI→VS1→REG→IPC→ABI chain plus new locked checks).
- **Threats:** disclosure/source/workspace/catalog drift; raw remote-entry bypass;
  opaque selector regeneration; key/log leakage; malformed/injected model text;
  stale response/approval, cancelled/switch/expiry/replay/double-click; partial ingress.
- **Reviews:** /root/vs4_security_review and /root/vs4_desktop_review, source-bound
  executable retests after freeze. Early design review found no contract blocker.
- **Deliverables:** code/launcher/tests, conformance map, reviewable PR,
  evidence/RESULTS.md, final-source hosted evidence and supervised VS4-030.
- **Status:** Proposed; no new gate is claimed passed before execution.
