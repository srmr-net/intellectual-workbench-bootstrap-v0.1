# Baseline and bounded implementation choices

PR11 is merged at57b76ef. Accepted code is source-hash guarded by earlier evidence;
the new product entry reuses its libraries/resources and preserves their bytes.
Existing ADR006 adapter already handles fixed HTTPS/model/routing, private sources,
strict untrusted suggestions, budgets, cancellation and bounded telemetry. Existing
Session/worker own canonical plan/approval, exact pins and distinct topology/ingress;
Desktop/renderer own typed edits, validated committed output and cold snapshots.

Product gaps: current launcher is generated under ignored target; credential setup
requires environment knowledge; outbound disclosure is general; preview is mostly
raw JSON; progress/error/accessibility presentation needs explicit states. Existing
ADR006 mock factory cannot prove a genuinely missing credential (it supplies one),
and lacks the full VS4 failure/room/partial-repair matrix.

## Truthful pre-send disclosure with unchanged accepted Catalog

Session generates fresh SHA256(nonce+pin+UIhash) opaque selectors internally. A
preflight cannot promise those exact future bytes without changing accepted code.
The review therefore shows exact user text, every dynamic semantic catalog field
and count, and explicitly describes candidateKey as a fresh host-issued opaque
selector generated at send. It is a data disclosure, not a raw wire-byte preview.
Fixed instructions/schema, destination/model and budgets are also disclosed.
No prospective token is presented as an actual value. Both early independent
reviewers accepted this design subject to executable enforcement.

A single-use nonce/digest/expiry binds reviewed text, complete ordered public
projection, exact local pin/UI/schema set, durable workspace CAS and provider/source
generation. The actual ProviderRequest is compared before forwarding to the
accepted LiveProvider; only the explicitly disclosed opaque selectors may differ.
Drift fails before credentials/transport; apply-time accepted revalidation remains.
Human data review is outside the accepted30s provider await deadline. No rewritten
suggestions, keys, schemas, or alternative apply/invocation path is introduced.

## Product-only control plane

New internal UX commands carry review/consent and credential-file references (never
raw keys). A thin factory uses accepted Control/Profile/HostCredential/LiveProvider/
TlsTransport; it does not duplicate API serialization/parsing or egress policy.
Source availability means metadata/source availability, not authenticated validity.
Changing the source revokes generation and outstanding work before it is usable.
Every remote entry, including raw operator.1 prepare, requires a spent bound review;
provider.1 consent alone cannot bypass it. Offline path stays operational.

Structured rows derive identity/pins and admitted labels/units from verified data.
Labels/units are plain-text hints; accepted semantic validation remains authority.
User selection switching immediately invalidates/cancels old work; the app's own
post-approval room selection is treated separately. Partial topology/ingress flags
remain distinct and form edits are the explicit repair path. No multi-component,
remote installation, new guest privilege or arbitrary QML is added.
