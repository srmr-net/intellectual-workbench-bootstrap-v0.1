//! Pure semantic UI validation; never constructs Qt objects or executes strings.
use crate::CheckedMetadata;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use wb_contracts::ui::{strict_json, ControlKind, Direction, Document};
pub const MAX_UI_BYTES: usize = 65536;
pub fn schema_digest() -> String {
    format!(
        "{:x}",
        Sha256::digest(include_bytes!("../../../schemas/ui.schema.json"))
    )
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedUi {
    pub path: String,
    pub sha256: String,
    pub schema_sha256: String,
    pub document: Document,
}
#[derive(Clone, Debug)]
pub struct BoundControl {
    pub field: String,
    pub schema: Value,
}
#[derive(Clone, Debug)]
pub struct ValidatedUi {
    pub document: Document,
    pub controls: BTreeMap<String, BoundControl>,
    pub input_port: String,
}
pub fn field_name(pointer: &str) -> Result<String, &'static str> {
    let raw = pointer.strip_prefix('/').ok_or("UI_BINDING_MISMATCH")?;
    if raw.is_empty() || raw.contains('/') || raw.len() > 256 {
        return Err("UI_BINDING_MISMATCH");
    }
    let mut result = String::new();
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '~' {
            result.push(match chars.next() {
                Some('0') => '~',
                Some('1') => '/',
                _ => return Err("UI_BINDING_MISMATCH"),
            });
        } else {
            result.push(c);
        }
    }
    if ["__proto__", "prototype", "constructor"].contains(&result.as_str()) {
        return Err("UI_BINDING_MISMATCH");
    }
    Ok(result)
}
fn safe_text(text: &str, max: usize) -> bool {
    let lower = text.to_ascii_lowercase();
    !text.trim().is_empty()
        && text.chars().count() <= max
        && !text.chars().any(|c| c.is_control() || "<>{};`".contains(c))
        && !["://", "javascript:", "qrc:", "qt.", "=>", "import qt"]
            .iter()
            .any(|s| lower.contains(s))
}
pub fn validate(bytes: &[u8], metadata: &CheckedMetadata) -> Result<ValidatedUi, &'static str> {
    if bytes.len() > MAX_UI_BYTES {
        return Err("UI_QUOTA_EXCEEDED");
    }
    let value = strict_json(bytes).map_err(|_| "INVALID_UI")?;
    if value["uiVersion"] != "0.1" {
        return Err("UNSUPPORTED_UI_VERSION");
    }
    let schema: Value =
        serde_json::from_slice(include_bytes!("../../../schemas/ui.schema.json")).unwrap();
    jsonschema::validator_for(&schema)
        .map_err(|_| "INVALID_UI")?
        .validate(&value)
        .map_err(|_| "INVALID_UI")?;
    let document: Document = serde_json::from_value(value).map_err(|_| "INVALID_UI")?;
    if !safe_text(&document.root.title, 160) {
        return Err("UNSAFE_UI_CONTENT");
    }
    let mut controls = BTreeMap::new();
    let mut ids = BTreeSet::new();
    let mut input_port = None;
    let mut used = BTreeSet::new();
    for c in &document.root.children {
        if !ids.insert(c.id.clone()) {
            return Err("UI_DUPLICATE_ID");
        }
        if !safe_text(&c.label, 160) || c.unit.as_ref().is_some_and(|s| !safe_text(s, 40)) {
            return Err("UNSAFE_UI_CONTENT");
        }
        let direction = match (&c.kind, &c.bind.direction) {
            (ControlKind::Readout, Direction::Output) => "outputs",
            (ControlKind::DecimalInput | ControlKind::IntegerInput, Direction::Input) => "inputs",
            _ => return Err("UI_BINDING_MISMATCH"),
        };
        let port = metadata.manifest["ports"][direction]
            .as_array()
            .ok_or("UI_BINDING_MISMATCH")?
            .iter()
            .find(|p| p["name"] == c.bind.port)
            .ok_or("UI_BINDING_MISMATCH")?;
        let contract = metadata
            .schemas
            .get(port["schema"].as_str().ok_or("UI_BINDING_MISMATCH")?)
            .ok_or("UI_BINDING_MISMATCH")?;
        if contract["type"] != "object" {
            return Err("UI_BINDING_MISMATCH");
        }
        let field = field_name(&c.bind.field)?;
        let property = contract["properties"]
            .get(&field)
            .ok_or("UI_BINDING_MISMATCH")?;
        let kind = property["type"].as_str().ok_or("UI_BINDING_MISMATCH")?;
        match c.kind {
            ControlKind::DecimalInput
                if kind == "string"
                    && property.get("pattern").and_then(Value::as_str).is_some() => {}
            ControlKind::IntegerInput if kind == "integer" => {}
            ControlKind::Readout if kind == "string" || kind == "integer" => {}
            _ => return Err("UI_BINDING_MISMATCH"),
        }
        if c.bind.direction == Direction::Input {
            if input_port.as_ref().is_some_and(|p| p != &c.bind.port) || !used.insert(field.clone())
            {
                return Err("UI_BINDING_MISMATCH");
            }
            input_port = Some(c.bind.port.clone());
        }
        controls.insert(
            c.id.clone(),
            BoundControl {
                field,
                schema: property.clone(),
            },
        );
    }
    let input_port = input_port.ok_or("UI_BINDING_MISMATCH")?;
    let port = metadata.manifest["ports"]["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == input_port)
        .unwrap();
    let contract = &metadata.schemas[port["schema"].as_str().unwrap()];
    if contract["required"].as_array().is_some_and(|a| {
        a.iter()
            .any(|v| v.as_str().is_none_or(|f| !used.contains(f)))
    }) {
        return Err("UI_BINDING_MISMATCH");
    }
    Ok(ValidatedUi {
        document,
        controls,
        input_port,
    })
}
