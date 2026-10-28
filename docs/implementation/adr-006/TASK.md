# LLM-001 — Host-controlled opt-in live Operator provider

- Owner: Codex; independent security/privacy and provider/runtime reviewers.
- Base:08c1e8d9f27db801628ce9cbee42c4c11d35fc3b (merged accepted VS3/PR10).
- Owned: additive apps/desktop-qt/live-provider standalone host/Qt entry, new tests,
  outer wrapper/additive CI, ADR006 kit/docs/evidence. Accepted source bytes frozen.
- Inputs: accepted OperatorProvider/ProviderRequest/Suggestion; read-only Catalog;
  accepted Session/preview/approval/ingress and VS3 desktop handoff.
- No changes: WIT, plan, manifest, UI, RDF/ontology, approval/grants, registry scope.
- Effective limits intersect accepted ADR005: request<=4096bytes, candidates<=12,
  suggestion<=16384bytes, overall provider<=30s even if profile asks for larger caps.
- Production endpoint fixed HTTPS api.openai.com:443/v1/responses; no redirects,
  proxies, arbitrary host/model URLs, tools, implicit model fallback or schema repair.
- Transport: trusted isolated stdlib Python TLS subprocess, fixed source embedded in
  Rust, DNS public-address pinning and verified TLS/SNI, bounded framed pipes, killed
  when request future drops. Secret only in private stdin/auth header; never argv.
- Secret facility: host env or owner-only regular private file, lazy after consent.
- Profile offline by default. Remote opt-in/disclosure/revocation separate from
  existing composition consent. Default no retries, configurable at most1 transient
  retry inside overall deadline/call cap; no automatic apply retry.
- Commands: bash apps/desktop-qt/validate-live-provider.sh; all locked accepted
  regressions plus isolated new manifest fmt/test/Clippy, mock Qt/process/LLM gates.
- Live: separate LIVE001–003, explicit user data/model/budget consent, host secret;
  no live keys in CI. Credential path supplied privately; never copied into artifacts.
- Status: Completed; accepted Linux/OpenRouter scope. Final source23fd97d,
  hosted run40734630115 and separately source-labelled LIVE evidence passed.


User steering after HTTP429: add separate allowlisted OpenRouter Chat adapter for
exact free NVIDIA model, keep original OpenAI implementation and offline default.
Only host remote-profile selection defaults to OpenRouter. Capability verification
and absence of provider JSON Schema mode are explicit in API_VERIFICATION.md.
New origin/data-policy consent is separate; no OpenAI consent/key is reused.
