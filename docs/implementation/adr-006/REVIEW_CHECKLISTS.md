# Independent reviews

## A — Security and privacy
- [ ] Secrets are host-side only, never in room state and never in error output.
- [ ] Network destination cannot be influenced by user prompt/catalog/model output; HTTPS and redirect policy enforced.
- [ ] Consent is explicit, revocable and distinct from composition approval.
- [ ] Catalog excludes unapproved documents, file paths and unnecessary metadata.
- [ ] Injection in catalog or returned explanation cannot add authority or tools.
- [ ] No response/refusal/error variant bypasses strict parsing and validation.
- [ ] Cancellation, timeouts and retry limits close off stale results.
- [ ] Failure and diagnostics are redacted, including nested HTTP errors.
- [ ] Scope of any retained prompts/responses is disclosed and default storage minimised.

## B — Provider and runtime integration
- [ ] Provider-neutral contract matches existing operator implementation.
- [ ] Real provider request format and schema features checked against current docs.
- [ ] Mock response cases include success, refusal, rate limit, auth error, incomplete, tool attempt, malformed and oversized payload.
- [ ] Host still constructs the canonical plan and checks exact pin and room revision.
- [ ] User preview/approval and cancellation work from actual Qt interactions.
- [ ] Reactive edits and offline restore never invoke provider.
- [ ] Existing regression suites and interfaces unmodified.
- [ ] Live smoke run scoped and labelled accurately, no unverified portability claims.

Both reviewers must record findings, resolution commits and verification. No self-attested 'independent' review.
