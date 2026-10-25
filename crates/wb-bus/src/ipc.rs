//! Finite reject-new room mailbox; authority remains in the kernel.
use serde_json::Value;
use std::{
    collections::{BTreeSet, VecDeque},
    sync::{Arc, Mutex},
};
use wb_contracts::ipc::{DispatchContext, Endpoint, ErrorCode, EventEnvelope, HostError, Result};

pub const DEFAULT_ROOM_CAPACITY: usize = 256;

#[derive(Default)]
pub struct Sequence(Mutex<u64>);
impl Sequence {
    pub fn next(&self) -> Result<u64> {
        let mut n = self.0.lock().unwrap();
        *n = n
            .checked_add(1)
            .ok_or_else(|| HostError::new(ErrorCode::ResourceLimit, "bus"))?;
        Ok(*n)
    }
}
#[derive(Debug, Clone)]
pub struct RoutedData {
    pub source: Endpoint,
    pub destination: Endpoint,
    pub schema_id: String,
    pub payload: Value,
    pub generation: u64,
    pub ui_token: Option<u64>,
}
#[derive(Debug, Clone)]
pub struct Queued {
    pub envelope: EventEnvelope,
    pub context: DispatchContext,
    pub ui_token: Option<u64>,
}
pub struct RoomQueue {
    room: String,
    incarnation: u64,
    capacity: usize,
    sequence: Arc<Sequence>,
    events: VecDeque<Queued>,
    accepted: BTreeSet<String>,
}
impl RoomQueue {
    pub fn new(
        room: &str,
        incarnation: u64,
        capacity: usize,
        sequence: Arc<Sequence>,
    ) -> Result<Self> {
        if capacity == 0 {
            return Err(HostError::new(ErrorCode::ResourceLimit, "bus"));
        }
        Ok(Self {
            room: room.into(),
            incarnation,
            capacity,
            sequence,
            events: VecDeque::new(),
            accepted: BTreeSet::new(),
        })
    }
    pub fn len(&self) -> usize {
        self.events.len()
    }
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    pub fn remaining(&self) -> usize {
        self.capacity - self.len()
    }
    pub fn pending(&self) -> Vec<EventEnvelope> {
        self.events.iter().map(|e| e.envelope.clone()).collect()
    }
    /// All routes and identities are host inputs. Request identity is deduplicated
    /// before acceptance; envelope IDs/room/sequences are always created here.
    pub fn publish(
        &mut self,
        request_id: &str,
        correlation: &str,
        context: DispatchContext,
        routes: Vec<RoutedData>,
    ) -> Result<Vec<EventEnvelope>> {
        if request_id.is_empty()
            || request_id.len() > 512
            || correlation.is_empty()
            || correlation.len() > 512
        {
            return Err(HostError::new(ErrorCode::InvalidEnvelope, "bus"));
        }
        if self.accepted.contains(request_id) {
            return Err(HostError::new(ErrorCode::DuplicateId, "bus"));
        }
        if routes.len() > self.remaining() {
            return Err(HostError::new(ErrorCode::QueueFull, "bus"));
        }
        // Validate before allocating any event/sequence or mutating the mailbox.
        for route in &routes {
            EventEnvelope {
                protocol_version: "0.1".into(),
                message_id: "pending".into(),
                room_id: self.room.clone(),
                source: route.source.clone(),
                destination: route.destination.clone(),
                correlation_id: correlation.into(),
                sequence: 1,
                schema_id: route.schema_id.clone(),
                payload: route.payload.clone(),
            }
            .validate()?;
            if route.payload.to_string().len() > 16 * 1024 {
                return Err(HostError::new(ErrorCode::ResourceLimit, "bus"));
            }
        }
        let metadata: Vec<_> = routes.iter().map(|r| (r.generation, r.ui_token)).collect();
        let mut batch = Vec::new();
        for route in routes {
            let sequence = self.sequence.next()?;
            batch.push(EventEnvelope {
                protocol_version: "0.1".into(),
                message_id: format!("{}:{}:{sequence}", self.room, self.incarnation),
                room_id: self.room.clone(),
                source: route.source,
                destination: route.destination,
                correlation_id: correlation.into(),
                sequence,
                schema_id: route.schema_id,
                payload: route.payload,
            });
        }
        self.accepted.insert(request_id.into());
        for (e, (generation, ui_token)) in batch.iter().zip(metadata) {
            let mut metadata = context.clone();
            metadata.execution_generation = generation;
            self.events.push_back(Queued {
                envelope: e.clone(),
                context: metadata,
                ui_token,
            });
        }
        Ok(batch)
    }
    pub fn pop(&mut self) -> Option<Queued> {
        self.events.pop_front()
    }
    /// Execution order is graph order; event sequence still records acceptance.
    pub fn pop_scheduled(&mut self, order: &[String]) -> Option<Queued> {
        let index = self
            .events
            .iter()
            .enumerate()
            .min_by_key(|(_, e)| {
                // Finish a committed propagation wave before committing the next
                // staged UI input. UI acceptance order is FIFO, across targets.
                (
                    u8::from(e.ui_token.is_some()),
                    if e.ui_token.is_some() {
                        0
                    } else {
                        order
                            .iter()
                            .position(|id| id == &e.envelope.destination.instance_id)
                            .unwrap_or(usize::MAX)
                    },
                    e.envelope.sequence,
                )
            })
            .map(|(i, _)| i)?;
        self.events.remove(index)
    }
    pub fn cancel_targets(&mut self, targets: &BTreeSet<String>) -> Vec<Queued> {
        let mut removed = Vec::new();
        let mut kept = VecDeque::new();
        for e in self.events.drain(..) {
            if targets.contains(&e.envelope.destination.instance_id) {
                removed.push(e);
            } else {
                kept.push_back(e);
            }
        }
        self.events = kept;
        removed
    }
}
