# G0 schema fixtures

The checker discovers every `valid/*.json` and `invalid/*.json`; the first filename segment selects component, composition, ui or room schema. Positive means structurally valid only. Digests are synthetic and no fixture claims a usable package, approved permission, valid graph or rendered UI.

Positive coverage: preserved component manifest; mortgage composition and room; permission request and endpoint shape; mortgage UI and every minimum UI control.

Negative coverage: missing manifest version and unknown manifest field; unsupported versions and unknown fields for all new contracts; missing pins, mutable versions, negative revisions; malformed endpoints, grant fields and unsafe UI references; stored grants and expression state addresses; unsupported controls, executable bindings/handlers, wrong widget attributes and empty selector options.

Runtime semantic cases (unknown ports, cycles, stale commits, incompatible units, missing artifacts and authorization) require host validators in later tasks. Shape fixtures do not claim CT conformance.
