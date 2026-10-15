# JSON contracts v0.1

All schemas use Draft 2020-12. `component.schema.json` is preserved byte for byte; it remains the canonical package manifest shape authority.

- `composition.schema.json`: complete proposed topology, expected unsigned 64-bit room revision, immutable `{id, version, sha256}` pins, directed endpoints, UI reference and permission requests.
- `ui.schema.json`: recursive closed control variants for all eleven minimum controls. Container children are UI nodes; leaf attributes depend on control type. Labels and optional `accessibleName` are plain display text. Buttons bind to a declared state/property; event code is never permitted. Selector options contain scalar JSON values; tables declare labeled property columns.
- `room.schema.json`: exact component pins, revision, topology, addressed JSON state and saved UI reference. Grants/tokens are not snapshot fields. Missing artifacts are a host restore outcome, not a mutable version replacement.

Proposed detail decisions and limitations are recorded in ADR-011. These schemas are G0 structural contracts, not authorization or runtime validation. Unknown keys are rejected except for permission scope and state payload data, which require capability/port-specific validation. Plain labels are not evaluated as markup or code.

Binding/state keys follow `instance.port[.property...]`: instance names begin with a letter and contain letters, digits, underscore or hyphen; port/property names begin with a lowercase letter and contain ASCII letters/digits. There are no expressions, calls, array indices or escaping. UI references are relative component paths ending in `/ui.json`, with no empty or dot segments, scheme, query or backslash. Resolve references against verified pinned packages; never fetch arbitrary URLs.

Run `python3 tools/check_contracts.py` with `tools/requirements.txt` installed. It registers schemas locally and disables network retrieval. Fixtures are discovered by their first filename segment; every contract needs both positive and negative fixtures. Duplicate JSON keys are rejected. Exit 0 means success, 1 fixture failure, 2 missing optional dependencies. `python3 -m unittest discover -s tools -p 'test_*.py'` checks the harness.

Host validation still must enforce unique instance/control/column identifiers, endpoint and binding existence (including existing room instances), manifests and artifact identity, port types/units, cycles, numeric constraints, permission policy, current revision and atomic commits. JSON Schema cannot compare identifiers across objects, resolve a port schema, or prevent an old computation from committing. `uniqueItems` only rejects fully identical array entries. Permission requests never constitute grants. Room state is input data requiring per-port validation and recomputation of derived outputs. No CT-01..CT-15 end-to-end claim is made.
