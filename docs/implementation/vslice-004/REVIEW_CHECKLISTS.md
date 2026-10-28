# Independent review protocols — VSLICE-004

Each review must be performed by a reviewer distinct from the implementing agent, reference commit and changed files, record findings and severity, then cite resolution/retest. Reviewer approval is not an alternative to executable tests.

## Review A: Privacy, agent security, provider boundary
- [ ] Remote mode explicit, offline remains functional; no background inference without consent.
- [ ] Outbound disclosure matches actual transport body; no hidden workspace documents, paths, secrets or privileged metadata.
- [ ] Host-approved credential source; no token in URL, prompt, UI history, snapshot, telemetry, CI, evidence or logs.
- [ ] HTTPS and destination restrictions, redirect controls and budgets remain per ADR-006.
- [ ] Untrusted catalog text and provider suggestions cannot inject instructions, forge keys/ports/pins or gain privileges.
- [ ] Approvals distinct from consent, exact proposal/revision-bound, expiring and single-use.
- [ ] Cancellation, timeout, provider switches and late responses do not mutate room state.
- [ ] Redaction inspection covers negative error cases and archived artifacts.
- [ ] Findings, fixes and independent retests documented.

## Review B: Desktop UX, kernel integration, restore
- [ ] Genuine Qt process and user-event path; no mock canvas pretending to be the app.
- [ ] Progress, status, error, consent and approval are distinct, accessible and keyboard navigable.
- [ ] Correct exact immutable registry pin shown, no version/digest hardcoding.
- [ ] Actual canonical plan applied by kernel, input ingress separate and partial failure recoverable.
- [ ] Numerical results computed by admitted WASM through accepted bus and projection.
- [ ] Reactive edit does not trigger provider, stale results fenced.
- [ ] Multi-room and repeat request isolation, replay/double-click rejection.
- [ ] New-process offline restore uses committed projections without provider/guest display calls.
- [ ] No regression in existing deterministic offline profile.
- [ ] Findings, fixes and independent retests documented.

## Sign-off template
Reviewer identifier: TBD  
Reviewed commit: TBD  
Evidence/checks: TBD  
Findings (severity, file, reproduction): TBD  
Fix and retest references: TBD  
Final outcome: PENDING / APPROVED / CHANGES REQUIRED
