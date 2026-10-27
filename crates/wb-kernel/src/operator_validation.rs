//! Admission-only Operator preview validation. No grants, lifecycle transitions,
//! instance reservations or generation allocation. Apply repeats room validation.
use crate::{runtime::error, Kernel};
use wb_contracts::ipc::*;
impl Kernel {
    /// Observe existing exact execution consent without granting authority.
    pub fn operator_execution_approved(
        &self,
        room: &str,
        instance: &str,
        pin: &wb_contracts::ComponentRef,
    ) -> bool {
        self.services.approved(room, instance, pin).is_ok()
    }

    /// The bounded Operator profile adds one instance to an existing empty room.
    /// This check is advisory; `Room::apply_plan` remains commit authority.
    pub fn validate_operator_plan(
        &self,
        current: &RoomSnapshot,
        plan: &CompositionPlan,
    ) -> Result<()> {
        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../../../schemas/composition.schema.json"))
                .map_err(|_| error(ErrorCode::SchemaViolation))?;
        let value = serde_json::to_value(plan).map_err(|_| error(ErrorCode::SchemaViolation))?;
        if !jsonschema::is_valid(&schema, &value) {
            return Err(error(ErrorCode::SchemaViolation));
        }
        if current.room_id != plan.room_id || current.revision != plan.expected_room_revision {
            return Err(error(ErrorCode::StateConflict));
        }
        if !plan.requested_permissions.is_empty() {
            return Err(error(ErrorCode::CapabilityDenied));
        }
        if !current.components.is_empty()
            || !current.connections.is_empty()
            || !current.state.is_empty()
            || plan.instances.len() != 1
            || !plan.connections.is_empty()
        {
            return Err(error(ErrorCode::IncompatiblePorts));
        }
        let artifact = self.services.resolve(&plan.instances[0].component)?;
        if artifact.inputs.len() != 1 {
            return Err(error(ErrorCode::IncompatiblePorts));
        }
        Ok(())
    }
}
