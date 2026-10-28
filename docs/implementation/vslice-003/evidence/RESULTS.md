# VSLICE003 — Linux offline Operator journey

Status: ACCEPTED (Linux deterministic offline Operator profile), 2026-10-28.
All VS3-001–026 gates, two independent reviews, full locked local checks, retained
regressions, clean hosted PR runs and independently verified downloaded evidence pass.
No mandatory gate is ignored, skipped, mocked away or waived.

## Source and hosted provenance

- Accepted base: `cc42020b2db406fdaeb81c3e3fbe6d33fc4a63f9` (merged PR9).
- Implementation: `3de262d`; review-driven fixes: `0ec4200`; final reviewed source:
  `03b6ac6fc8f347cf314ab6c8816c0ea34f21b243`.
- Tested PR head, including review receipts:
  `d3379a8a62972d56871249dafc4a66df783a5eb5`.
- Hosted tested clean PR merge/source:
  `9bac8f276c5fe6b5d97dbfa0647d9798d8f8f0a9`, refs/pull/10/merge.
- [VSLICE003 hosted run40603341850](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40603341850)
  succeeded, attempt1, workflow `.github/workflows/vslice-003.yml`.
- [Retained Bootstrap PR run40603341608](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40603341608)
  also succeeded. Duplicate push run40603313949 was deliberately cancelled; it is not
  acceptance evidence or a test failure.
- Unmodified [artifact12779664042](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40603341850/artifacts/12779664042),
  `vslice003-validation-linux`, is archived as `hosted.zip`; API digest and run metadata
  are `hosted-artifacts.json` and `hosted-run.json`. Complete logs: `hosted-logs.zip`.
- Final acceptance commit adds documentation/evidence only. Reviewed source24-path
  hashes and earlier accepted49/60 source maps remain byte-identical.

## Executed checks

`PATH="$PWD/target/vs2-venv/bin:/tmp/wb-abi-tools/bin:$PATH" bash apps/desktop-qt/validate-vslice-003.sh`
exited0 locally. Hosted CI ran the same wrapper from a clean Linux checkout.
`local-validation-console.log` records the complete local command output.

| Check | Observed result |
|---|---|
| Root locked Rust workspace |162 passed,0 ignored/failed; includes all accepted ABI/IPC/REG/UI/VS1/VS2/OP tests |
| Formatting/Clippy | Root and standalone product manifest pass; warnings denied for Rust |
| Python tooling/contracts |5 retained tooling tests,30 original structural fixtures and all JSON/Turtle/UI/snapshot/Operator corpora pass |
| Retained native Qt entries | UI7 and VS2 8 passed entries,0 failed/skipped |
| Retained VS2 process suite |14 cases/25 app-worker pairs pass |
| Retained OP suites |9 framed-worker and6 genuine Qt cases/7 app-worker pairs pass |
| New named VS3 journey suite |24 pass;22 include genuine Qt journeys,2 exercise native provider boundaries;25 app-worker pairs |
| New native product-worker suite |6 pass, including prior-checkpoint preservation, write failure, CAS reload, stale/future/noncanonical counters and cancellation |
| New provenance suite |6 pass, including substituted workflow/run/attempt/PR/head/status rejection |
| VS3-025 | Both source-bound independent approvals and hashed zero-exit command logs validated |
| VS3-026 | Complete local and hosted archives pass the separate unchanged verifier; exact API run identity is bound |

The standalone product binary has no added Rust unit-test count; its behavior is
covered by the real worker and Qt process tests. Filtered native regression reruns
inside named journey cases are not miscounted as new workspace tests. Exact names
and commands: [conformance map](../CONFORMANCE_MAP.md).

## Real product/process evidence

The center-column explicit request reaches the bounded offline provider, offers only
admitted candidates, and previews a host-validated canonical planVersion0.1/exact pin
with fresh unchecked consent and no saved-room mutation. Keyboard Confirm/Reject,
missing-input clarification and double-confirm fences use real Qt events/accessibility.
Renamed identity/version0.9.7 is actually admitted and executed, proving selection does
not assume the original package identity/version/digest.

Explicit confirmation initializes actual installed pure WASM through typed ingress:
GBP250000, annual5%,300months produces committed1461.48. Editing principal to300000
produces1753.77 with physical provider count unchanged at1. Accepted Desktop lazily
reconstructs roots then applies the first post-handoff edit (2 guest calls); no claim
that a reactive edit uses zero guest calls is made. Second-room requests bring the
provider total to2; room input/output/approval and selected-view ownership stay isolated.
The rapid-switch witness records an actual pending Apply before selecting another room.

Pinned cold restart uses distinct hosted appPIDs19629/19644 and workerPIDs19631/19646.
Both exact component/UI pins, full checkpoints, committed inputs/outputs and pin flags
are identical before/after. Both rooms display offline with provider disabled,
IPv4/IPv6 socket creation denied, provider0 and guest0. Last selected projection is
`{principal:"300000",rate:"5",term:"300",payment:"1753.77"}`. Local cold appPIDs are
766081/766119, workers766084/766122. `process-facts.json` lists all25 pairs per platform
and checksum/equality receipts; raw action/command/accessibility/snapshot/trace data
are in the ZIPs. Screenshots are supplemental: shell and embedded Quick canvas are
separate offscreen captures, not a fabricated combined window.

Topology, durable topology checkpoint, typed initialization and final committed-output
checkpoint are separate stages. Guest trap and actual filesystem-budget failure tests
report these booleans truthfully, preserve earlier room checkpoints and never fabricate
payment or claim cross-stage rollback. When topology is durable, explicit direct-form
repair works without a new provider call. Missing pins preserve snapshot bytes and
show recovery rather than substituting another release.

## Independent reviews and fixed findings

Agent-security reviewer `/root/review_vs3_security` and Qt/kernel reviewer
`/root/review_vs3_qt_kernel` approve03b6ac6, each independently executing24 journey,
6 native worker and6 provenance cases. Their exact24 source hashes, commands,
exit codes, findings/fixes and hashed logs are in `review-agent-security.json`,
`review-qt-kernel.json` and `reviewer-logs/`. Earlier pre-freeze runs interrupted by
binary relinking are retained honestly as historical non-acceptance logs.

Primary testing fixed rematerialized Desktop counter reset and identical-metadata
editor ownership. Independent reviews reproduced/fixed noncanonical counter lexemes,
missing refresh after reloaded-workspace CAS conflict, and cross-workflow hosted-run
metadata substitution. Both independently reverified final downloaded evidence:
`hosted-independent-verification.json` and `hosted-security-verification.json`.
The latter also rejects actual successful Bootstrap-run metadata on the same head,
wrong run id, attempt and workflow. No mandatory finding remains.

## Versions, hashes and reproduction

Both: Rust/Cargo1.99.1, Wasmtime49.0.1, cargo-component0.21.1, jsonschema4.26.0,
rdflib7.6.0 and pySHACL0.30.1. Hosted: Ubuntu24.04/x86_64, Python3.11.17,
Qt6.4.2/qmake3.1, GCC13.3.0. Local: Linux/x86_64, Python3.14.7,
Qt6.11.2/qmake3.1, GCC16.2.1. Full version/platform/lock/source receipts are in each
archive. Local system Qt-header SFINAE warnings are retained; Rust Clippy passes.
Different platform binaries have separately recorded hashes; bit-identical builds
across machines are not claimed.

Reproduce product checks from d3379a8 with prerequisites above:

```sh
bash apps/desktop-qt/validate-vslice-003.sh
python3 docs/implementation/vslice-003/evidence/verify.py \
  docs/implementation/vslice-003/evidence/hosted.zip \
  --sha256 66f69f1e7660b7ea9424e584233ce953941c1c756b70c5db33b5d631e19bbedb \
  --source 9bac8f276c5fe6b5d97dbfa0647d9798d8f8f0a9 --hosted \
  --run-metadata docs/implementation/vslice-003/evidence/hosted-run.json \
  --head d3379a8a62972d56871249dafc4a66df783a5eb5
```

The final evidence commit adds these archived files to reproduce offline verification.
`local.zip` verifies with source d3379a8 and the local archive digest below, omitting
`--hosted`/run metadata/head arguments. Verifier outputs are archived as
`local-verification.json` and `hosted-verification.json`.

| Artifact | SHA256 |
|---|---|
| `hosted.zip` | `66f69f1e7660b7ea9424e584233ce953941c1c756b70c5db33b5d631e19bbedb` |
| `hosted-logs.zip` | `0e3a1d661d4d8044d24b980e2a16c4de1331ef03e934e5588b8e52b8256c1ace` |
| `hosted-run.json` | `80c753bd4fc4a54f33d01b25e28ea9252875921901e49a95b14cbf29d3e56c8b` |
| `hosted-verification.json` | `b1312eb739d3e13f71bf562583af0763711c86839f5c2e8659012d9baaa7e1b4` |
| `local.zip` | `ba86aa3125a24cc00871f598f79369b9f6873feb74774ac9ab3f7acba86b650b` |
| `local-validation-console.log` | `909122186c0bebb1126ae015e36da4d410e777dff51aeffa6fea5e25e9943ec1` |
| `local-verification.json` | `06a2273ee7c70ee719258729fb1e5f782b160c905d7bcba999c2140919e06d53` |
| `process-facts.json` | `648dd3355f421d5528ce48465a2cdcae01f5efd0e0728890a64e064d591b623f` |
| Hosted Qt app | `9e8edadc7f2761cf6085594278ff3c1525a6cddf6a7fc52812f5e4f4b2dd235a` |
| Hosted Rust worker | `0777622aac7c83168892fda789244ca5ed09aa28a3d985097ebead31db5e13cb` |
| Local Qt app | `99efbfd782751dbfe3f0224f37045a905cfcd143295aaa847a105c3267a33291` |
| Local Rust worker | `fdf20e427cea50daac2a6b162872bcf1a7daf7e4c3eba9ddd21a3df4918ccd16` |

## Scope and unresolved architecture

Accepted only for the Linux deterministic offline profile: one installed pure component
per fresh empty room, beside existing rooms within the accepted16-room snapshot limit.
The accepted planner, WIT/ABI, manifest/ontology/UI/snapshot contracts, lifecycle and
bus semantics are unchanged. Generic trusted QML has a view-lifetime fix; no package
QML/code is loaded. No remote provider/registry, new guest privilege, code generation,
populated-room or multi-component planning, automatic approval/repair/apply retry is
introduced. Optional external-model trial: NOT RUN (optional; no adapter).

No architectural blocker remains for this scope. Permanent namespace (ADR014), shared
numeric conventions (ADR008), full database/power-loss recovery (ADR012), broader NL
and planning, external providers and cross-platform packaging remain unresolved or
deferred. Topology-then-ingress remains deliberately non-atomic across stages.
