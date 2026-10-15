# wb-wasm-host — ADR-001 validation adapter

Contract version: **0.1 proposed**. This crate is an executable compatibility spike,
not a kernel-integrated component runtime or package installer. The authoritative
WIT is `../../wit/component.wit`; Wasmtime host and cargo-component guest bindings
are generated from that one source. Manifest discriminator remains
`org.workbench.component@0.1` (not the WIT package identifier).

`Host::new(Limits)` configures Wasmtime 49.0.1 with fuel and epoch interruption.
`Host::register(manifest, raw_bytes, schema_documents)` validates the unchanged
canonical manifest schema, requires empty permissions, verifies raw SHA-256,
rejects all component imports and checks generated world types. Schema documents
are supplied by a trusted bundle resolver, keyed by manifest schema paths; remote
retrieval is disabled. Duplicate port names and missing/invalid schemas fail.
This method never opens a component-nominated filesystem path or grants access.
It does not install a bundle, verify RDF, authorize requests or compare semantic
ports; those remain separate trusted-host responsibilities.

`Host::call(&Registered, &Message)` checks the registered input port and Draft
2020-12 JSON schema before creating a fresh Store. It instantiates with an empty
linker and calls typed `compute`. It validates every returned port and payload
before returning any result batch. Each pure event gets its own store, including
after errors. A registered artifact from a different Host engine is rejected.

`Failure` distinguishes manifest, digest, world, denied import, schema, undeclared
port, guest-reported error, engine trap, resource budget and host configuration
failures. These are spike errors; integration with shared kernel error DTOs is
not implemented. No permissions, WASI, filesystem, network or process APIs are
linked for guests.

Default execution budgets (policy candidates): 8 MiB per linear memory, one
memory, two tables of at most 1,024 entries, eight core instances, 2,000,000 fuel
units, 100 engine ticks at 10 ms each, 16 KiB input payload, eight output messages
and 16 KiB combined output payload/port bytes. Failed memory/table growth traps;
resource errors remain separate from guest errors/traps. Wasmtime 49 count-cap
errors are identified by its pinned error-chain prefix, guarded by memory/table/
instance-count regression fixtures. An engine-owned timer
advances epochs and is signaled/joined when Host drops. Epoch interruption is a
cooperative Wasmtime deadline, not a hard real-time guarantee. Fuel exhaustion
is deterministic; an additional epoch test gives the loop effectively unlimited
fuel to exercise interruption independently.

The 2 MiB raw artifact cap is checked before compilation. These execution limits
do not bound native compilation time/memory or host JSON/schema processing.
Canonical lifting allocates results before result-count/byte validation; linear
memory, table and fuel budgets provide a coarse bound, not a complete native
allocation budget. Production admission needs separate process/resource policy,
schema-complexity bounds and a reviewed output-lifting allocation strategy.
The caller must validate raw JSON duplicate keys and bundle paths/symlinks,
metadata consistency and approval policy before registration; accepting parsed
`Value` is a deliberately narrow trusted-caller API.

Run `bash examples/wasm-components/validate.sh` from the repository root after
installing the pinned prerequisites described in that directory. Tests fail with
an explicit build instruction when guest artifacts are absent; none silently
skip. ABI-010's local test checks reproduction declarations, while successful
hosted Linux CI remains a separate acceptance requirement. See
`docs/adr/evidence/ADR-001-results.md` for actual results and remaining gates.
