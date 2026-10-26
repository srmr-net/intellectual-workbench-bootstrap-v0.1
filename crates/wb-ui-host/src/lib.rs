//! Host-owned semantic UI session. No Qt objects, guest imports or new bus semantics.
use serde_json::Value;
use std::collections::BTreeMap;
use wb_contracts::{
    ipc::{CompositionPlan, Endpoint, InstanceSpec, RoomSnapshot, UiReference},
    ui::*,
    ComponentRef,
};
use wb_kernel::{
    room::{Room, UiAdapter},
    Kernel,
};
use wb_registry::{Registry, ValidatedHandle};
use wb_semantic::{ui::ValidatedUi, CheckedMetadata};
pub const CONTRACT_VERSION: &str = "0.1";
pub mod framing;

pub struct Session {
    room: Room,
    adapter: UiAdapter,
    pub key: UiKey,
    pub ui: ValidatedUi,
    drafts: BTreeMap<String, Value>,
    sequence: u64,
    next_request: u64,
}
fn code(error: wb_contracts::ipc::HostError) -> String {
    serde_json::to_value(error.code)
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned()
}
impl Session {
    /// Caller has explicitly approved the selected exact pin. No implicit grant.
    pub fn start(
        kernel: &Kernel,
        registry: &Registry,
        handle: &ValidatedHandle,
        room_id: &str,
        instance: &str,
    ) -> Result<Self, String> {
        let ui = registry.read_verified_ui(handle).map_err(|e| e.code)?;
        let (checked, _) = registry.read_verified_bundle(handle).map_err(|e| e.code)?;
        let pin = kernel
            .cache_registry_release(registry, handle)
            .map_err(code)?;
        let reference = Self::reference(&checked)?;
        let mut room = kernel
            .create_room(room_id, "Semantic workbench", reference.clone(), 128)
            .map_err(code)?;
        room.apply_plan(
            CompositionPlan {
                plan_version: "0.1".into(),
                room_id: room_id.into(),
                expected_room_revision: 0,
                instances: vec![InstanceSpec {
                    instance_id: instance.into(),
                    component: pin,
                }],
                connections: vec![],
                ui: reference,
                requested_permissions: vec![],
            },
            "ui.host",
            "ui.create",
        )
        .map_err(code)?;
        room.prepare(instance, "ui.host", "ui.prepare")
            .map_err(code)?;
        room.start(instance, "ui.host", "ui.start").map_err(code)?;
        Self::attach(room, ui, checked, instance)
    }
    fn reference(checked: &CheckedMetadata) -> Result<UiReference, String> {
        let ui = checked.ui.as_ref().ok_or("UI_UNAVAILABLE")?;
        if std::path::Path::new(&ui.path)
            .file_name()
            .and_then(|s| s.to_str())
            != Some("ui.json")
        {
            return Err("UI_BINDING_MISMATCH".into());
        }
        Ok(UiReference {
            definition_ref: format!("{}/{}", checked.manifest["id"].as_str().unwrap(), ui.path),
        })
    }
    /// Fresh exact registry reads and fresh current approval, never saved grants.
    pub fn restore(
        kernel: &Kernel,
        registry: &Registry,
        handle: &ValidatedHandle,
        snapshot: RoomSnapshot,
        room_id: &str,
        instance: &str,
    ) -> Result<Self, String> {
        let ui = registry.read_verified_ui(handle).map_err(|e| e.code)?;
        let (checked, _) = registry.read_verified_bundle(handle).map_err(|e| e.code)?;
        let expected = handle.pin();
        if snapshot.room_id != room_id
            || snapshot.components.len() != 1
            || snapshot.components[0].instance_id != instance
            || snapshot.components[0].id != expected.id
            || snapshot.components[0].version != expected.version
            || snapshot.components[0].sha256 != expected.sha256
            || snapshot.ui != Self::reference(&checked)?
        {
            return Err("UI_UNAVAILABLE".into());
        }
        kernel
            .cache_registry_release(registry, handle)
            .map_err(code)?;
        let mut room = kernel.restore_room(snapshot, 128).map_err(code)?;
        if let Some(error) = room.run_until_idle().into_iter().next() {
            return Err(code(error));
        }
        Self::attach(room, ui, checked, instance)
    }
    fn attach(
        mut room: Room,
        ui: ValidatedUi,
        checked: CheckedMetadata,
        instance: &str,
    ) -> Result<Self, String> {
        let adapter = room
            .authorize_ui(
                "ui.host",
                [Endpoint::new(instance, &ui.input_port)]
                    .into_iter()
                    .collect(),
            )
            .map_err(code)?;
        let descriptor = checked.ui.as_ref().ok_or("UI_UNAVAILABLE")?;
        let key = UiKey {
            room_id: room.id().into(),
            instance_id: instance.into(),
            ui_id: format!("{}#{}", descriptor.path, descriptor.sha256),
        };
        let mut drafts = BTreeMap::new();
        for c in &ui.document.root.children {
            if c.bind.direction == Direction::Input {
                if let Some(value) = room
                    .input(instance, &c.bind.port)
                    .and_then(|v| v.get(&ui.controls[&c.id].field))
                {
                    drafts.insert(c.id.clone(), value.clone());
                }
            }
        }
        Ok(Self {
            room,
            adapter,
            key,
            ui,
            drafts,
            sequence: 0,
            next_request: 0,
        })
    }
    pub fn room(&self) -> &Room {
        &self.room
    }
    pub fn room_mut(&mut self) -> &mut Room {
        &mut self.room
    }
    pub fn snapshot(&self) -> RoomSnapshot {
        self.room.snapshot()
    }
    pub fn drafts(&self) -> &BTreeMap<String, Value> {
        &self.drafts
    }
    fn projection(&mut self, mut diagnostics: Vec<Diagnostic>) -> Projection {
        self.sequence = self
            .sequence
            .checked_add(1)
            .expect("UI session sequence exhausted");
        let mut values = BTreeMap::new();
        for c in &self.ui.document.root.children {
            let field = &self.ui.controls[&c.id].field;
            let value = if c.bind.direction == Direction::Input {
                self.room
                    .input(&self.key.instance_id, &c.bind.port)
                    .and_then(|v| v.get(field))
            } else {
                let output = self.room.output(&self.key.instance_id, &c.bind.port);
                if let Some(p) = output {
                    if !p.valid {
                        diagnostics.push(Diagnostic::new(
                            p.error
                                .map(|e| {
                                    serde_json::to_value(e)
                                        .unwrap()
                                        .as_str()
                                        .unwrap()
                                        .to_owned()
                                })
                                .as_deref()
                                .unwrap_or("STALE_RESULT"),
                            Some(&c.id),
                        ));
                    }
                }
                output
                    .and_then(|p| p.value.as_ref())
                    .and_then(|v| v.get(field))
            };
            if let Some(value) = value {
                if value.to_string().len() <= 512 {
                    values.insert(c.id.clone(), display_scalar(value));
                } else {
                    diagnostics.push(Diagnostic::new("UI_QUOTA_EXCEEDED", Some(&c.id)));
                }
            }
        }
        Projection {
            key: self.key.clone(),
            room_revision: self.room.revision(),
            execution_generation: self.room.generation(&self.key.instance_id).unwrap_or(0),
            projection_sequence: self.sequence,
            values,
            diagnostics,
        }
    }
    pub fn ready(&mut self) -> Reply {
        Reply {
            protocol: "0.1".into(),
            kind: "ready".into(),
            local_token: String::new(),
            document: Some(self.ui.document.clone()),
            projection: self.projection(vec![]),
            accepted: true,
        }
    }
    pub fn rejected(&mut self, token: &str, diagnostic: Diagnostic) -> Reply {
        Reply {
            protocol: "0.1".into(),
            kind: "projection".into(),
            local_token: token.chars().take(64).collect(),
            document: None,
            projection: self.projection(vec![diagnostic]),
            accepted: false,
        }
    }
    pub fn request(&mut self, request: Request) -> Reply {
        let token = request.local_token.clone();
        if request.protocol != "0.1"
            || token.is_empty()
            || token.len() > 64
            || request.key != self.key
        {
            return self.rejected(&token, Diagnostic::new("UI_OWNERSHIP_DENIED", None));
        }
        if request.expected_generation != self.room.generation(&self.key.instance_id).unwrap_or(0) {
            return self.rejected(&token, Diagnostic::new("STALE_GENERATION", None));
        }
        let edits: Vec<(&String, &Value)> = match &request.command {
            Command::Edit {
                control_id,
                proposed_value,
            } => vec![(control_id, proposed_value)],
            Command::Apply { proposed_values } => proposed_values.iter().collect(),
            _ => vec![],
        };
        if edits.len() > 32 {
            return self.rejected(&token, Diagnostic::new("UI_QUOTA_EXCEEDED", None));
        }
        for (id, value) in &edits {
            let Some(control) = self
                .ui
                .document
                .root
                .children
                .iter()
                .find(|c| c.id == ***id)
            else {
                return self.rejected(&token, Diagnostic::new("UI_BINDING_MISMATCH", None));
            };
            if control.bind.direction != Direction::Input || value.to_string().len() > 512 {
                return self.rejected(&token, Diagnostic::new("UI_BINDING_MISMATCH", Some(id)));
            }
        }
        for (id, value) in edits {
            self.drafts.insert(id.clone(), value.clone());
        }
        if request.expected_room_revision != self.room.revision() {
            return self.rejected(&token, Diagnostic::new("STATE_CONFLICT", None));
        }
        match request.command {
            Command::Edit { .. } | Command::Apply { .. } => {
                let mut object = self
                    .room
                    .input(&self.key.instance_id, &self.ui.input_port)
                    .and_then(Value::as_object)
                    .cloned()
                    .unwrap_or_default();
                for control in self.ui.document.root.children.clone() {
                    if control.bind.direction == Direction::Input {
                        let Some(raw) = self.drafts.get(&control.id) else {
                            return self.rejected(
                                &token,
                                Diagnostic::new("INCOMPLETE_INPUT", Some(&control.id)),
                            );
                        };
                        let Some(value) = normalize(&control.kind, raw) else {
                            return self.rejected(
                                &token,
                                Diagnostic::new("SCHEMA_VIOLATION", Some(&control.id)),
                            );
                        };
                        let bound = &self.ui.controls[&control.id];
                        if !jsonschema::validator_for(&bound.schema)
                            .unwrap()
                            .is_valid(&value)
                        {
                            return self.rejected(
                                &token,
                                Diagnostic::new("SCHEMA_VIOLATION", Some(&control.id)),
                            );
                        }
                        object.insert(bound.field.clone(), value);
                    }
                }
                let value = Value::Object(object);
                // Whole input is checked by accepted submit_ui before queueing.
                self.next_request = self
                    .next_request
                    .checked_add(1)
                    .expect("UI request counter exhausted");
                let correlation = format!("ui:{}", self.next_request);
                if let Err(error) = self.room.submit_ui(
                    &self.adapter,
                    &correlation,
                    Endpoint::new(&self.key.instance_id, &self.ui.input_port),
                    value,
                    &correlation,
                ) {
                    return self.rejected(&token, Diagnostic::new(&code(error), None));
                }
                if let Some(error) = self.room.run_until_idle().into_iter().next() {
                    return self.rejected(&token, Diagnostic::new(&code(error), None));
                }
                Reply {
                    protocol: "0.1".into(),
                    kind: "projection".into(),
                    local_token: token,
                    document: None,
                    projection: self.projection(vec![]),
                    accepted: true,
                }
            }
            _ => Reply {
                protocol: "0.1".into(),
                kind: "projection".into(),
                local_token: token,
                document: None,
                projection: self.projection(vec![]),
                accepted: true,
            },
        }
    }
}
fn normalize(kind: &ControlKind, value: &Value) -> Option<Value> {
    match kind {
        ControlKind::DecimalInput => {
            let s = value.as_str()?;
            let mut parts = s.split('.');
            let first = parts.next()?;
            let fraction = parts.next();
            if first.is_empty()
                || (first.len() > 1 && first.starts_with('0'))
                || !first.bytes().all(|c| c.is_ascii_digit())
                || parts.next().is_some()
                || fraction.is_some_and(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()))
            {
                None
            } else {
                Some(value.clone())
            }
        }
        ControlKind::IntegerInput => {
            if value.as_i64().is_some() || value.as_u64().is_some() {
                return Some(value.clone());
            }
            let text = value.as_str()?;
            let unsigned = text.strip_prefix('-').unwrap_or(text);
            if unsigned.is_empty()
                || (unsigned.len() > 1 && unsigned.starts_with('0'))
                || !unsigned.bytes().all(|b| b.is_ascii_digit())
            {
                return None;
            }
            if text == "-0" {
                return None;
            }
            text.parse::<i64>()
                .ok()
                .map(Value::from)
                .or_else(|| text.parse::<u64>().ok().map(Value::from))
        }
        ControlKind::Readout => None,
    }
}

/// Explicit native host policy helper used by the executable, not package metadata.
pub fn selected_pin(
    registry: &Registry,
    handle: &ValidatedHandle,
    kernel: &Kernel,
) -> Result<ComponentRef, String> {
    kernel
        .cache_registry_release(registry, handle)
        .map_err(code)
}

// Display transport is text even for JSON integer payloads: Qt never rounds u64.
fn display_scalar(value: &Value) -> Value {
    Value::String(
        value
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| value.to_string()),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ui_005_integer_text_and_projection_are_lossless() {
        for text in [
            "18446744073709551615",
            "9007199254740993",
            "-9223372036854775808",
        ] {
            let value = normalize(&ControlKind::IntegerInput, &Value::String(text.into())).unwrap();
            assert_eq!(display_scalar(&value), Value::String(text.into()));
        }
        assert!(normalize(&ControlKind::IntegerInput, &Value::String("-0".into())).is_none());
    }
}
