# Headless mortgage vertical slice — PROPOSED

This source/build recipe creates a complete installable local package in
`target/mortgage-packages/mortgage-headless`. Unlike the original metadata-only
mortgage reference, it includes a real compiled import-free Component Model guest,
matching raw digest, canonical manifest, Turtle and local schemas. The adversary
and observation packages are explicitly test fixtures, not financial engines.
No UI definitions, extra manifest fields, permission requests or ontology changes.

Install the repository's pinned Rust/cargo-component and Python requirements, then:

```sh
bash examples/mortgage-headless/validate.sh
REGISTRY_PYTHON=/absolute/path/to/python target/debug/wb-mortgage-headless \
  target/mortgage-packages/mortgage-headless /tmp/mortgage-local-registry \
  '{"principal":"250000.00","annual_rate_percent":"5.00","term_months":300}'
```

The CLI uses actual strict registry admission, exact resolution, verified cache,
explicit host approval for that pure pin/room/instance, lifecycle, sealed native
input, WIT invocation and atomic Room::output commit. JSON includes the committed
result, input envelope, real invocation ID/revision, exact pin and correlated audit.
With no downstream node there is no fabricated output bus event. Tests additionally
route the result through a separately admitted compiled observation component.
Failures emit bounded JSON error codes, not parser diagnostics or host paths.
No Operator, Qt, native grants, remote lookup or guest services are involved.

`HeadlessRoom` is native production glue in wb-kernel: no policy default is changed.
Its UI-named accepted ingress APIs work without any GUI. Required room UI metadata
is an inert host-owned reference and is never resolved or put in the bundle.

Principal is unsigned ASCII decimal, >0 through1000000000, at most two fractional
digits. Annual percentage is0..100 with at most six fractional digits; term is an
integer1..600. Integer-equivalent JSON month numbers such as8.0 are accepted by the
schema and guest. No signs, leading zeros, whitespace, exponents or special numeric
strings. Trailing-zero scales such as5.00 are permitted as the supplied fixtures
require. Exact integer rationals avoid small-rate cancellation and intermediate
rounding. Only the final payment is rounded half-up to two places, including0.00.
Output schema enforces canonical shape, not proof of the financial result.
GBP is the existing illustrative unit; no exchange-rate, fee, APR, tax or advice
semantics are supplied. No shared ADR-008 numeric convention is settled here.

The CLI/product tests use a bounded20M-fuel policy through the existing Limits API;
accepted Host defaults remain2M fuel and all accepted tests remain unchanged.
Memory stays8MiB, epoch ticks100, message16KiB and result-count8. Exact arithmetic
at the maximum tiny-rate600-month boundary exceeds the generic spike fuel default;
calibration, actual results and durations are archived. These are engineering test
limits, not universal financial workload guarantees.

`reference.py` uses an independent Decimal90 discounted cash-flow sum to verify
all supplied vectors, nine boundaries and sixteen seeded cases. `rebuild.py`
builds in a genuinely fresh target directory and compares raw hashes on the SAME
machine. Local/hosted hashes are recorded separately; no cross-machine identity is
assumed. `validate.sh` retains all original ABI/IPC/REG wrappers, locked workspace
checks, guest formatting and complete version/hash/log evidence.

Accepted failure behavior is retained: invalid ingress does not change input or
projection; once a valid input wave starts, its revision/input can commit before
compute. Failed candidate output never replaces the prior value or publishes a
partial batch, but marks diagnostics invalid/error and may fail its instance.
Snapshots contain authoritative inputs/pins/topology; fresh-kernel restore uses
current registry integrity checks, new approvals/generations and recomputes results.
No durable room database or physical power-cut guarantee is introduced.

See docs/implementation/vslice-001 for mandatory gates and independent reviews.
Milestone stays PROPOSED until local checks, reviews, hosted Linux execution and
archived artifact verification all succeed.
