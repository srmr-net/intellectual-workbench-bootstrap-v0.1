//! Native headless ingress over the accepted room controller. No new protocol,
//! execution grant or UI package: the legacy UI reference is inert host metadata.
use crate::{
    room::{Room, UiAdapter},
    Kernel,
};
use serde_json::Value;
use std::collections::BTreeSet;
use wb_contracts::{ipc::*, ComponentRef};

pub struct HeadlessRoom {
    room: Room,
    adapter: UiAdapter,
    target: Endpoint,
}
impl HeadlessRoom {
    /// Exact component must already be registry-cached and explicitly approved
    /// for this room/instance. Failure never grants authority or leaves a live room.
    pub fn start(
        kernel: &Kernel,
        room_id: &str,
        name: &str,
        target: Endpoint,
        component: ComponentRef,
        capacity: usize,
    ) -> Result<Self> {
        let ui = UiReference {
            definition_ref: "host.headless/session/ui.json".into(),
        };
        let mut room = kernel.create_room(room_id, name, ui.clone(), capacity)?;
        room.apply_plan(
            CompositionPlan {
                plan_version: "0.1".into(),
                room_id: room_id.into(),
                expected_room_revision: 0,
                instances: vec![InstanceSpec {
                    instance_id: target.instance_id.clone(),
                    component,
                }],
                connections: vec![],
                ui,
                requested_permissions: vec![],
            },
            "headless.host",
            "headless.create",
        )?;
        room.prepare(&target.instance_id, "headless.host", "headless.prepare")?;
        room.start(&target.instance_id, "headless.host", "headless.start")?;
        Self::attach(room, target)
    }
    /// Attach a fresh native adapter to a live/restored controller. Adapters and
    /// approvals are never serialized. This does not start/approve an instance.
    pub fn attach(mut room: Room, target: Endpoint) -> Result<Self> {
        let adapter = room.authorize_ui("headless.host", BTreeSet::from([target.clone()]))?;
        Ok(Self {
            room,
            adapter,
            target,
        })
    }
    pub fn submit(
        &mut self,
        request: &str,
        payload: Value,
        correlation: &str,
    ) -> Result<EventEnvelope> {
        self.room.submit_ui(
            &self.adapter,
            request,
            self.target.clone(),
            payload,
            correlation,
        )
    }
    pub fn room(&self) -> &Room {
        &self.room
    }
    pub fn room_mut(&mut self) -> &mut Room {
        &mut self.room
    }
}
