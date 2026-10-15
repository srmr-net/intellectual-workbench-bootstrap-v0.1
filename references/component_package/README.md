# Workbench Component Package and Manifest Specification v0.1

- `SPECIFICATION.md` — normative proposed contract
- `schemas/component.schema.json` — JSON Schema 2020-12 for `component.json`
- `examples/mortgage-calc.wbc/` — metadata-only reference package; **not installable** because `logic.wasm` is deliberately absent
- `tools/validate_package.py` — schema, package layout, hash and selected RDF cross-file checks (SHACL, full permission/evidence mapping, and UI-binding validation are pending) (requires `jsonschema` and `rdflib`)

Run:

```bash
python tools/validate_package.py examples/mortgage-calc.wbc schemas/component.schema.json
```

The example is **expected to fail** with `missing file: logic.wasm`. Replace the placeholder with a valid compiled component, recompute the digest in both `component.json` and RDF, and rerun.

This package proposes contracts aligned to Architecture and Interface Specification v0.1 and ontology v0.1. The RDF namespace `https://workbench.example/ns/0.1#` is provisional pending ADR-005. Package/archive publication and ABI pinning remain gated on ADRs.
