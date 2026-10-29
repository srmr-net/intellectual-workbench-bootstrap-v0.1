# CHAT-001 results

Status: **Proposed**. ADR016 and VSLICE004 remain **Proposed**. This iteration follows the user's request to set component work aside and prioritize a normal long-running chat experience, explicitly including real contextual OpenRouter chat.

Implemented: default `./workbench` chat entry, growing scrollable transcript, bottom composer, Enter/Shift+Enter, Stop, New/select conversations, isolated drafts, readable inert text, accessible status updates and session provider Settings. Existing tools are secondary under Settings → Tools or `./workbench --tools`; they retain the prior validated Operator execution path. Chat has no registry/kernel handle or execution authority.

History is process-local and bounded; it is not durable across restart. Requests include up to twelve successful turns within the disclosed context budget. Complete replies, not token streaming, are implemented. Credential policy and HTTPS transport are compiled from accepted source unchanged; no new network implementation or guest imports were introduced.

Executed local evidence (development source, final source-bound rerun/reviews pending):

- 16 native tests passed, including strict response parsing, multi-turn context, secret rejection, cancellation, invalid-disable revocation, escaped transcript quota, request serialization failure and New-at-capacity cancellation.
- 7 worker-backed Qt cases passed (9 including setup/cleanup), zero failures/skips; actual accessibility alerts and interfaces checked. Retry isolation, draft switching fences and visible input-limit feedback are covered.
- 3 real-process framing/production/fixture tests passed, with no network or actual credentials. Unknown fields on unit-like commands initially exposed Serde's unit-variant behaviour; empty struct variants now reject extras, and negative process tests pass.
- All 162 root workspace tests passed with `RUST_TEST_THREADS=1 cargo test --workspace --locked`. Root formatting, locked Clippy and contract checks passed. Logs in local/. Prior concurrent root failure remains archived with UX001; no quota was weakened.

Independent review identified and fixed serialized transcript overflow, invalid-disable cancellation, cross-chat retry, New-at-capacity cancellation, dynamic accessibility status and draft ownership during control acknowledgments. Final receipts will bind the frozen source; preliminary findings alone are not approval.

Supervised live result: the real production Qt app was opened with the user-provided private credential reference, remote initially disabled. The user enabled the session provider, tried ordinary chat and a contextual follow-up, and reported **“Chat and follow-up work.”** No message text, credential content or raw response was collected. `live-manual.json` records the limited observation. Exact request count, usage, returned model/version, latency and live process IDs were not captured; they must not be invented or inferred from fixtures. This is a successful manual usability report, not completion of the old VSLICE004 formal live gates.

Preserved baseline: UX001 draft [PR14](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/pull/14) has successful [hosted Linux CI](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40921750651). Its original live attempt sent zero requests and remains incomplete. The chat iteration needs its own reviews/hosted evidence and does not inherit acceptance from PR14.

Remaining: final source-bound reviews, clean final local wrapper, hosted chat/regression CI and artifact verification. No acceptance label is advanced. The current new protocol/interaction decision remains proposed, and future persistent chat, streaming and integrated tool orchestration remain separate work.
