# VSLICE-004 — Live Operator: End-to-End User Experience

**Status:** PROPOSED  
**Version:** 0.1  
**Target:** Linux desktop and the ADR-006 accepted OpenRouter adapter.  
**Prerequisites:** ADR-001–006, VSLICE-001–003 accepted in their documented scopes.

## 1. Objective
Demonstrate a discoverable, comprehensible and reliable live-model path from a natural-language request to an approved, reactive application assembled from an admitted local component. The focus is the *user experience* and its observable safety properties, rather than rebuilding the provider infrastructure already accepted under ADR-006.

## 2. Mandatory reference journey
1. On a Linux desktop, launch the actual application from a clean, isolated user profile using the documented launcher. An admitted mortgage component is installed through the real local registry. No hardcoded artifact version/digest or mortgage-specific QML.
2. Present provider choice clearly (Offline / OpenRouter). Offline remains the default unless the user explicitly selects remote mode. A missing credential must produce a safe actionable state without outbound traffic.
3. Configure the host-managed OpenRouter credential using an approved secure entry/source. Never persist raw secrets in rooms, logs, screenshots, crash reports or evidence bundles.
4. User enters: **“I need a mortgage calculator for a £250,000 loan over 25 years at 5%.”** Display what will be sent remotely, including request text and bounded component catalog fields, and obtain fresh, explicit consent before the remote request. Do not silently transmit prior room documents or workspace filesystem paths.
5. Show bounded progress and a clear cancellable state. The request uses the real ADR-006 provider adapter; capture model identifier, request correlation and redacted outcome without treating provider text as authoritative.
6. If a valid suggestion is returned, host code verifies candidate key, current admitted exact pin, input schemas, units, room revision, relevant permissions and proposal constraints. Any unrecognised candidate/port or malformed output fails closed.
7. Present a **separate approval step** with readable component identity/version, pin/digest details, submitted values, affected room, capabilities, planned effect, remote provider disclosure and what will happen on confirmation. Consent to inference is not approval to execute.
8. A genuine user approval action triggers only the existing canonical `planVersion: "0.1"` topology path. Apply via the accepted kernel, then perform initial typed ingress as a **separate** operation. A partially successful topology followed by failed ingress must be truthfully reported with explicit safe retry options.
9. Real installed mortgage WASM computes the expected reference payment **GBP 1461.48** for principal 250000 GBP, nominal annual interest 5%, 25 years, monthly payments; the trusted semantic Qt renderer displays the committed result.
10. User edits principal to 300000 GBP; observe **GBP 1753.77** from reactive WASM recomputation. Provider call counter remains unchanged. Rapid edits and stale results must not overwrite later committed values.
11. Pin the assembled canvas, close the process cleanly, disconnect provider/network, start a fresh OS process and display the pinned exact component identity, inputs, layout and last committed result with **zero provider and zero guest calls solely to paint restored state**.
12. Confirm the offline provider still works, and test a second room to prove no cross-room suggestion/approval/state leakage.

## 3. Boundaries and authority
- Qt owns interface presentation, progress, actions, accessibility and error messages; it does NOT execute provider output or package-supplied QML/JavaScript.
- Operator/provider can only suggest from a bounded read-only local admitted catalog. Output is untrusted and parsed strictly by the host.
- Host owns schema/unit validation, exact pins, revision fences, user approvals, plans, network consent, budgets and privileges.
- Registry remains local and immutable. No remote component fetching or installation from model-provided URLs.
- Kernel and WASM host own lifecycle, typed ingress, execution and atomic validated output. UI changes never trigger inference once topology is assembled.
- Persist only safe room/app state; never credentials or raw private prompts by default. Remote inference is opt-in and bounded.
- Preserve the existing OpenRouter-specific accepted scope. Do not claim that OpenAI, Anthropic or additional models are validated under this slice.

## 4. UX and errors
- Clearly distinguish provider not configured, missing consent, requesting, cancelled, rejected by provider, malformed suggestion, preview ready, approval declined, stale proposal, plan committed, ingress failed, calculation failed and restored offline.
- Confirmation is disabled until strict host validation completes; rejection/replay/expired preview cannot mutate room. A double-click cannot duplicate an instance.
- No automatic unsafe retry after ambiguous remote response, cancellation or conflict. Respect existing retry and budget rules; surface rate limit/timeout/refusal as non-mutating states.
- Provider switching mid-flight invalidates old responses and previews. Never allow a late result to overwrite a newer room state.
- Keyboard-only operation, accessible names/focus, announcement of progress and error states, and visible high-contrast consent/approval distinctions are mandatory.
- Network disconnection must not remove previously committed canvas contents. An ordinary local edit must not require a provider reconnect.
- Diagnostic/evidence artifacts MUST be redacted and should retain enough correlation, versions, commands and hashes for audit.

## 5. Test profiles and evidence
**Deterministic CI profile:** local fake OpenRouter HTTP endpoint/transport reproducing success, refusal, invalid JSON, 429, 5xx, timeout, cancellation, disclosure, stale response and blocked egress. Test the actual desktop/provider boundary without using billable inference or credentials. The fake may not replace the actual kernel/WASM/registry execution path.

**Live acceptance profile:** supervised, opt-in interactive Linux run against real OpenRouter with a user-configured model. Record sanitized provider/model identifiers, date, latency, available usage/cost, exact reference outputs, approval clicks and evidence of zero repeat provider calls. Live evidence MUST NOT include API keys or raw confidential documents. If the live provider is unavailable, keep status Proposed (or explicitly only deterministic-profile validated), not Accepted for live.

**Hosted CI:** pinned dependencies, real Qt process tests (headless display as necessary), Rust/Python tests, build/lint/format, accepted gates, archived logs, artifact hashes and link to successful run. Fix final documentation-only CI separately if needed and record the final workflow status accurately.

## 6. Out of scope
Multi-component planning; agent-generated components; automatic component publishing; new WASM ABI; expanded guest privileges; arbitrary generated QML; remote registries; general semantic inference; additional providers as accepted targets; power-loss/database recovery; mobile/macOS/Windows builds; translation/localisation; production billing controls beyond ADR-006 limits.

## 7. Acceptance rule
All VS4-001–030 gates pass with proof and two independent reviews, accepted regressions retained and final hosted Linux CI successful. Then mark **Accepted — Linux/OpenRouter live Operator experience**, citing the tested model and run. Any absent evidence must remain explicitly open.
