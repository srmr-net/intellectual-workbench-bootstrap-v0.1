# GRAPH-001 — ADR-003 local registry

- **Owner:** Codex; independent filesystem/durability and ABI reviewers.
- **Goal:** strict local directory admission, immutable exact pins, crash recovery.
- **Dependencies:** accepted ADR-001/002; proposed ADR-003; package specification v0.1.
- **Owned paths:** crates/wb-registry, crates/wb-semantic, narrow kernel adapter,
  tools/registry*, examples/wasm-components registry fixtures, CI, evidence and docs.
- **Forbidden changes:** accepted WIT, ABI/IPC tests and semantics, manifest schema,
  ontology, references/, remote resolution, signatures, Qt, capability grants.
- **Inputs:** real cargo-component compiled pure echo guest; adversarial mutations.
- **Acceptance:** `bash examples/wasm-components/validate-registry.sh` runs guest
  builds, `cargo test --workspace`, Clippy with warnings denied, fmt, Python
  contracts/tests, REG-001..020 and unchanged ABI/IPC suites. Hosted Linux workflow
  and verified uploaded artifacts required before acceptance.
- **Threats:** traversal, aliases/races, duplicate JSON, RDF disagreement, compiler
  resources, partial commits, concurrent installers/readers, exact-pin corruption.
- **Review:** independent filesystem, durability/concurrency and ABI admission.
- **Known blockers:** UI package binding ownership and nonempty permissions remain
  unsupported; reject explicitly. Linux local-filesystem slice only; no power-cut
  or hostile administrator guarantee. ADR remains Proposed during validation.
