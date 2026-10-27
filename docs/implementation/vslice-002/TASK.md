# VSLICE-002 — Interactive mortgage and persistent canvas

- Owner: primary Codex; distinct Qt/accessibility and persistence/security reviewers.
- Base: master c117632f7c50ed0cf729fadd814ac8548c22db0d, merged accepted PR6/7.
- Dependencies: accepted ADR001–004, VSLICE001 and supplied VSLICE002 kit.
- Owned paths: new wb-desktop-host; narrow wb-storage internal snapshot module;
  new apps/desktop-qt/desktop and desktop-tests; VS2 fixtures, outer validation/CI,
  docs/API/evidence. Accepted renderer, original bridge/session, all original tests,
  guest/domain/schema/ontology/WIT and original collectors remain unchanged.
- Goal: actual three-column Qt process, installed exact-pin picker, typed actual
  WASM calculation, pinning, strict durable committed snapshots, zero-compute cold
  display and non-destructive recovery. No Operator, remote stores or guest rights.
- Checks: bash apps/desktop-qt/validate-vslice-002.sh; retained entire accepted UI
  wrapper; named VS2-001–024 native/genuine Qt/process tests; two independent source
  reviews; clean locked local and actual hosted Linux CI/download verification.
- Status: PROPOSED until every mandatory gate/review/archive check passes.
