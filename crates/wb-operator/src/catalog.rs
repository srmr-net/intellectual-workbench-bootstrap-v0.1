//! Read-only admitted candidate projection. No RDF literals or package paths reach
//! a provider. Exact pins and authoritative schemas remain in the host map.
use crate::{
    provider::Candidate,
    suggestion::{OperatorError, Result, Suggestion},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, io::Read};
use wb_registry::{Registry, ReleasePin};
const CAPABILITY: &str = "https://workbench.example/ns/0.1#CalculateMortgagePayment";
const TERMS: &str = "https://workbench.example/ns/0.1#MortgageTerms";
const GBP: &str = "https://workbench.example/ns/0.1#GBP";
#[derive(Clone)]
pub struct Selection {
    pub pin: ReleasePin,
    pub input_port: String,
    pub output_ports: Vec<String>,
    pub ui_path: String,
    pub ui_sha256: String,
    pub ui_schema_sha256: String,
    schema: Value,
    fields: Vec<String>,
}
pub struct Catalog {
    candidates: Vec<Candidate>,
    selections: BTreeMap<String, Selection>,
}
impl Catalog {
    pub fn discover(registry: &Registry) -> Result<Self> {
        let mut nonce = [0u8; 32];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut nonce))
            .map_err(|_| OperatorError::Unavailable)?;
        let mut candidates = Vec::new();
        let mut selections = BTreeMap::new();
        for summary in registry
            .discover(Some(CAPABILITY))
            .map_err(|_| OperatorError::Unavailable)?
        {
            let Ok(handle) = registry.resolve_exact(&summary.pin) else {
                continue;
            };
            let Ok((metadata, _)) = registry.read_verified_bundle(&handle) else {
                continue;
            };
            let Ok(ui) = registry.read_verified_ui(&handle) else {
                continue;
            };
            let Some(checked) = metadata.ui.as_ref() else {
                continue;
            };
            if metadata.manifest["permissions"] != serde_json::json!([]) {
                continue;
            }
            let Some(ports) = metadata.manifest["ports"]["inputs"].as_array() else {
                continue;
            };
            let [port] = ports.as_slice() else { continue };
            if port["semanticType"] != TERMS || port["unit"] != GBP || port["name"] != ui.input_port
            {
                continue;
            }
            let Some(schema) = port["schema"]
                .as_str()
                .and_then(|p| metadata.schemas.get(p))
            else {
                continue;
            };
            let mut fields: Vec<_> = ui
                .document
                .root
                .children
                .iter()
                .filter(|c| c.bind.direction == wb_contracts::ui::Direction::Input)
                .map(|c| wb_semantic::ui::field_name(&c.bind.field).expect("admitted binding"))
                .collect();
            fields.sort();
            fields.dedup();
            // Only the explicit domain vocabulary is offered. Output bindings,
            // labels and adversarial metadata never enter provider context.
            if fields != ["annual_rate_percent", "principal", "term_months"] {
                continue;
            }
            if candidates.len() == 12 {
                return Err(OperatorError::QueueFull);
            }
            let mut digest = Sha256::new();
            digest.update(nonce);
            digest
                .update(serde_json::to_vec(&summary.pin).map_err(|_| OperatorError::Unavailable)?);
            digest.update(checked.sha256.as_bytes());
            let key = format!("{:x}", digest.finalize());
            candidates.push(Candidate {
                candidate_key: key.clone(),
                capability: CAPABILITY.into(),
                inputs: fields.clone(),
                has_semantic_ui: true,
            });
            selections.insert(
                key,
                Selection {
                    pin: summary.pin,
                    input_port: ui.input_port,
                    output_ports: metadata.manifest["ports"]["outputs"]
                        .as_array()
                        .ok_or(OperatorError::Unavailable)?
                        .iter()
                        .map(|p| {
                            p["name"]
                                .as_str()
                                .map(str::to_owned)
                                .ok_or(OperatorError::Unavailable)
                        })
                        .collect::<Result<Vec<_>>>()?,
                    ui_path: checked.path.clone(),
                    ui_sha256: checked.sha256.clone(),
                    ui_schema_sha256: checked.schema_sha256.clone(),
                    schema: schema.clone(),
                    fields,
                },
            );
        }
        if serde_json::to_vec(&candidates)
            .map_err(|_| OperatorError::Unavailable)?
            .len()
            > 32768
        {
            return Err(OperatorError::QueueFull);
        }
        Ok(Self {
            candidates,
            selections,
        })
    }
    pub fn candidates(&self) -> Vec<Candidate> {
        self.candidates.clone()
    }
    pub fn validate(&self, registry: &Registry, suggestion: &Suggestion) -> Result<Selection> {
        let selection = self
            .selections
            .get(&suggestion.candidate_key)
            .ok_or(OperatorError::InvalidSuggestion)?;
        if suggestion.input_values.keys().cloned().collect::<Vec<_>>() != selection.fields {
            return Err(OperatorError::InvalidSuggestion);
        }
        let value = serde_json::to_value(&suggestion.input_values)
            .map_err(|_| OperatorError::InvalidSuggestion)?;
        if !jsonschema::is_valid(&selection.schema, &value) {
            return Err(OperatorError::InvalidSuggestion);
        }
        let handle = registry
            .resolve_exact(&selection.pin)
            .map_err(|_| OperatorError::Unavailable)?;
        let (metadata, _) = registry
            .read_verified_bundle(&handle)
            .map_err(|_| OperatorError::Unavailable)?;
        let checked = metadata.ui.ok_or(OperatorError::Unavailable)?;
        if checked.path != selection.ui_path
            || checked.sha256 != selection.ui_sha256
            || checked.schema_sha256 != selection.ui_schema_sha256
            || metadata.manifest["permissions"] != serde_json::json!([])
        {
            return Err(OperatorError::Unavailable);
        }
        Ok(selection.clone())
    }
}
