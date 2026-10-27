# ADR005 bounded Operator — acceptance evidence

Status: ACCEPTED for the scoped Linux offline single-component profile. All
OP-001–024 gates, retained regressions, two independent source-bound reviews and
complete downloaded hosted verification passed on 2026-10-27.
No mandatory gate is waived. Runtime implementation188cbba; evidence packaging
correction443acbc; review receipts0995f78. All runtime/test/UI/schema bytes are
unchanged between the first two commits; the latter corrects archive inventory only.

The complete local wrapper exited0:162 Rust tests (138 retained+24 new),5 Python
tooling tests, original30 structural fixtures plus UI/snapshot/operator corpora,
accepted Qt7/8 entries and14 desktop process cases/25 app-worker pairs,9 Operator
worker cases and6 genuine Operator Qt cases/7 app-worker pairs. Rustfmt and full
locked workspace Clippy with-D warnings passed. No ignored/skipped mandatory case.

Both independent reviews approvef7fe86d with no remaining findings (runtime
tests at188cbba; evidence corrections independently reviewed at443acbc/f7fe86d). Agent-security
binds49 critical source paths; runtime integration binds60 source paths. Each ran
24 native,9 worker and6 genuine Qt cases, recorded commands/exit statuses/log hashes
and independently reviewed the evidence-only delta. All reviewed sources and logs
are included in the final archive. The runtime review also checked71 earlier
accepted-source hash entries unchanged.

Local archive SHA256: f2a9e9e0d6e5891fb3bc950b15c0344f65919098ead97342838dfb1d2c1a092a.
Its recorded source is443acbc (runtime/evidence code); the review receipts were
committed immediately afterward in0995f78. Reproduce from0995f78, which adds only
those reviewed receipts/logs. The separately supplied source hashes verify identical
critical code. See local-verification.json and local-validation-console.log.

The genuine demo typed the explicit request, showed preview0rooms and unchecked
consent, approved once, then displayed actual1461.48. Direct edit of principal to
300000 displayed1753.77 with physical provider count unchanged at1. Accepted Desktop
performs root restoration plus the edit (2 guest calls), preserving its behavior.
Separate cold app/worker processes restored exact pins and committed1461.48 with
provider disabled, IPv4/6 socket creation denied, provider0 and guest0 for display.
Guest trap and filesystem/quota failures retain truthful topology/input/checkpoint
stages; real form repair succeeds without a new provider call. Host-only final
FileStore write failure also preserved the prior topology-only checkpoint after
real1461.48 computation.

Local Qt6.11.2/GCC16.2.1 and Python3.14.7; Rust/Cargo1.99.1,
Wasmtime49.0.1, cargo-component0.21.1. System Qt header SFINAE warnings are retained;
Rust Clippy has no warnings. Shell and embedded Quick canvas are separate offscreen
captures, with actual accessible widget/Quick text and binary/PID evidence.

Earlier successful execution (incomplete artifact; not acceptance):
https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40568646238
PR head0995f7847700e6998c49033d7a340739cb89d32f. Earlier foundation run40538999368
was independently downloaded/verified and remains archived separately; it does not
substitute for this final24-gate acceptance run.

Scope remains one installed pure component in an existing empty native room, initial
empty workspace UI, explicit tested MortgageTerms/GBP adapter and offline fixture.
No general NL claim, remote provider/registry, new ABI/import/privilege, generated QML,
multiple-component graph, automatic approval/repair/apply retry or database/power-loss
recovery is introduced. Namespace permanence (ADR014), general numeric conventions
(ADR008), richer planning and full database recovery (ADR012) remain outside scope.

Hosted run40568646238 passed all jobs on PR head0995f78 (merge7c7f80f),
but its upload omitted exactly `operator/sources/.github/workflows/ci.yml`.
The original ZIP, logs and API provenance are retained as
`hosted-incomplete-upload-*`; server ZIP SHA256 is
`bdede37c4994f89e7cef799d3a458bbbc92766090b1e44b0fe93de14310a60ee`.
Complete offline verification correctly rejected this missing source. Commit
f7fe86d adds only upload-artifact `include-hidden-files: true`; independent
reviews approved that change and the complete hosted rerun passed before acceptance. No result or
source is reconstructed or substituted.

## Complete hosted acceptance

Run: https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40592650858
All three jobs succeeded. PR head `04f0e986a9add82db5081f17796a6c46cefeb052`;
tested PR merge/source `117160896c285b2895127d6c2f55c374b2a2329a`, clean checkout.
Reviewed implementation/packaging `f7fe86d5e00981d05d4d0fee6288f34ff21e8478`.
The final evidence/acceptance commit adds documentation only; tested sources remain
byte-identical to these source-bound reviews.

Artifact [operator-validation-linux, ID12776550140](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40592650858/artifacts/12776550140)
is archived unmodified as `hosted.zip`. API metadata is `hosted-artifacts.json` and
`hosted-run.json`; complete logs are `hosted-logs.zip`. The unchanged offline verifier
passed with hosted provenance, all archived hashes including the hidden workflow,
162 Rust tests/24 native Operator tests, nine worker cases, six genuine Qt cases,
seven distinct app-worker pairs, all prior gate matrices and cold provider0/guest0.
No mandatory gate is ignored or skipped. See `hosted-verification.json`.

Reproduce from PR head04f0e98 with Rust1.99.1, cargo-component0.21.1 and system Qt6:

```sh
bash apps/desktop-qt/validate-operator.sh
python3 docs/implementation/adr-005/evidence/verify.py \
  docs/implementation/adr-005/evidence/hosted.zip \
  --sha256 f86453e7d8181fbe4642a9cc23856a1b9e1460c620610c0d93be2c95c46b5c61 \
  --source 117160896c285b2895127d6c2f55c374b2a2329a --hosted \
  --run-metadata docs/implementation/adr-005/evidence/hosted-run.json \
  --head 04f0e986a9add82db5081f17796a6c46cefeb052
```

Hosted versions: Rust/Cargo1.99.1, cargo-component0.21.1, Wasmtime49.0.1,
Python3.11.17 and Qt6.4.2 (qmake3.1), Ubuntu24.04 x86_64. Full version/platform
receipts are inside `run.json` and `operator/qt-version.txt` in the archive.

SHA256:

| Artifact | SHA256 |
|---|---|
| `hosted.zip` | `f86453e7d8181fbe4642a9cc23856a1b9e1460c620610c0d93be2c95c46b5c61` |
| `hosted-logs.zip` | `4e65c41a07def1abd5a58a94f43270327ed493d28024e79161e0f21539da8a2a` |
| `hosted-run.json` | `882b2663b95e74b05025d351d0d1907a276a12ba984aa46311f485fc771f0cd3` |
| `hosted-verification.json` | `fed80b99e0f8fcf11c4ac1a819afb4bc31b77d9a90a533ca4130ff9549d38d52` |
| `local.zip` | `f2a9e9e0d6e5891fb3bc950b15c0344f65919098ead97342838dfb1d2c1a092a` |
| Hosted native Qt app | `20f378c8ac5896bada3c9cfc40c57819f4e7e97c0a3125d910721e67ac9272f3` |
| Hosted native worker | `b7567c4a1d74e1fc4794f73d4beb19cfb4310afb8607fc9346ecf50280918ff7` |

The runtime-integration reviewer independently reran the complete hosted verifier
(exit0); command, result and verifier/provenance hashes are recorded in
`hosted-independent-verification.json`.
