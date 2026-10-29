# UX-001 independent security/runtime preliminary review

Reviewer: /root/ux001_security_review. Implementation still evolving; no source-bound approval is claimed here. No private credential contents or live inference accessed.

SEC-UX-001 (medium): generic !ok handling appended 'workspace unchanged' to conversation before recognizing topologyCommitted. That is false after non-atomic topology success followed by initialization/output persistence failure. Require topology/checkpoint/ingress-specific messages in status, conversation and activity, with actual failing-guest retest.

SEC-UX-002 (medium): normal workspace status claimed all changes saved whenever tool controls existed, including pending or invalid/unapplied drafts. The legacy accurate state message had moved into Settings. Require busy/draft/error-aware normal status and invalid-draft retest.

Implementer reports both fixed; final source inspection and independent retests pending.

Checked bounds: Settings relocates the existing bridge-backed registry list; old double-click opener is disconnected and hidden. Navigation only selects existing room IDs. Provider model/endpoint fields are read-only, enablement is session-scoped, and the same native actual-body guard and proposal approval route remain. Conversation is bounded plain text and only current request editor text is sent. No runtime/schema/permission API edits observed at preliminary review.

SEC-UX-003 (medium): !ok response handling ignored the accepted native refreshDesktop instruction after durable/revision drift. The proposal remains rejected safely, but UI cache/epoch can stay stale; current-room sidebar selection intentionally skips a redundant selection, making the displayed Ask-again recovery potentially ineffective. Honor refreshDesktop by explicitly selecting the current room through the accepted bridge, without auto-apply/retry; add a stale-approval/refresh retest where feasible.
