# Codex Implementation Task — VSLICE-001

## Preconditions
- Work on a branch based on merged PR #5 or a pinned commit containing accepted ADR-001 through ADR-003. Record the base SHA and PR status. The kit itself does not assert current GitHub merge status.
- Read `AGENTS.md`, accepted ADR-001/002/003, corresponding evidence, component package specification v0.1, ontology, and this `SPECIFICATION.md` and `TEST_MATRIX.md`.
- Inspect **actual code** and interfaces in `wb-registry`, `wb-wasm-host`, `wb-kernel`, `wb-bus`, `wb-reactive`, `wb-semantic`, `wb-contracts`; do not infer public APIs solely from suggested ADR sketches.

## Work sequence
1. **Inventory & design note:** map existing APIs and identify minimal missing production integration links. State any contract or trust boundary ambiguity before coding.
2. **Guest and package:** implement pure headless mortgage WASM guest in the guest workspace, keep WIT and interface constant; add input/output schemas and RDF using the existing manifest and ontology; automate digest stamping without introducing unchecked package publishing.
3. **Integrated execution:** wire registry resolution to kernel preparation/start; wire host bus ingress to schema validation/room revision and WIT invocation; ensure output batch validates and is committed to room projection; add a stable CLI or integration fixture demonstrating the full pipeline.
4. **Coverage:** implement every gate in `TEST_MATRIX.md`, use actual guest and registry for the key positive tests; negative tests must assert absence of side effects and correct typed errors. Keep all accepted ABI/IPC/REG tests.
5. **Independent reviews:** request one review focusing on mathematical semantics/schema and one on boundary integrity/races/revision fences; reviewers must not author the core feature under review, and must record concrete test evidence and findings.
6. **Evidence and CI:** run locked clean guest build; record raw guest/package SHA-256, version pins, commands, tests, redacted logs; run formatting, Clippy, contracts, all workspace suites; require **GitHub-hosted Linux** green run with logs/artifacts. Capture run URL and commit SHA. Add evidence file from template.
7. **Acceptance:** update milestone status to Accepted only *after* all gates/review/hosted CI succeed, document exceptions explicitly, and submit PR. Do not change accepted ADR status.

## Mandatory negative behaviours
- Wrong digest, incompatible interface, bad RDF, invalid input, undeclared output, denial/absent pin, attempted imports, stale completions, full queue and interrupted install must reject safely.
- If inducing an invalid guest output needs a separate adversarial fixture, compile a second guest for that fixture rather than inventing host mock output.
- No new WASI imports, guest host services, UI definitions, Qt integration, remote registry, AI/Operator or implicit privileges.

## Suggested test commands (adapt to actual repo layout; record exact executed commands)
```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
python -m pytest   # only if existing repo uses pytest; otherwise run repo's actual contract scripts
# existing guest/package build script and hosted CI workflow
```
Do not claim a test passed unless its command actually ran. Record absent/unavailable tooling as gaps, not passes.
