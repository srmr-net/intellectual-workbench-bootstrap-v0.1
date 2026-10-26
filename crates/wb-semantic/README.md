# wb-semantic

ADR-003 supplies `CheckedMetadata` and `bounded_worker` for trusted native
admission workers. `tools/registry_validate.py` runs strict manifest checks,
local-only acyclic schema expansion, RDF/Turtle parsing, pinned trusted
`admission-shapes.ttl` through pySHACL 0.30.1, and exact reconciliation of identity,
ports/types/units, artifact and evidence references. Namespace/ontology stay
provisional and unchanged; the new shapes implement existing package invariants.
No graph inference, data-supplied shapes, SPARQL, JavaScript, remote retrieval or
permission grants occur. Assurance assertions never elevate host trust.

Only private staged bytes may be passed to the worker. Registry owns race-safe
source access. Python, worker script and shapes are trusted installation inputs;
missing tools fail admission. Limits: 2 GiB address space, configured CPU/wall time,
4 MiB output, 64 file descriptors, no core dump, null stdin/stderr and cleared
environment. Parent death and timeout terminate workers. This is resource
containment, not an OS filesystem/network security sandbox. Public errors expose
stable codes and bounded package-relative locations, never parser diagnostics.

`bash examples/wasm-components/validate-registry.sh` exercises this path through
REG-001..020 plus accepted ABI and IPC regression tests.

ADR-004 adds pure `ui` validation over admitted bytes and actual expanded local
port schemas. It enforces the restricted canonical UI schema, literal labels,
unique IDs, fixed directions/kinds, single RFC6901 member bindings and complete
aggregate input coverage. `CheckedMetadata.ui` is optional and omitted for None,
so historical headless record bytes remain stable. It carries raw/schema hashes,
manifest UI path and a typed document; it grants no runtime authority.
