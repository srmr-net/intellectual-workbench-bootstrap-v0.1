//! Stable DAG planning and exact revision/generation fences.
use std::collections::{BTreeMap, BTreeSet};
use wb_contracts::ipc::{Connection, DispatchContext, ErrorCode, HostError, Result};
#[derive(Clone, Debug)]
pub struct Graph {
    order: Vec<String>,
    children: BTreeMap<String, BTreeSet<String>>,
}
impl Graph {
    pub fn new(ids: impl IntoIterator<Item = String>, edges: &[Connection]) -> Result<Self> {
        let mut children: BTreeMap<_, BTreeSet<String>> =
            ids.into_iter().map(|id| (id, BTreeSet::new())).collect();
        let mut incoming: BTreeMap<_, usize> = children.keys().map(|id| (id.clone(), 0)).collect();
        for e in edges {
            if !children.contains_key(&e.from.instance_id)
                || !children.contains_key(&e.to.instance_id)
            {
                return Err(HostError::new(ErrorCode::IncompatiblePorts, "reactive"));
            }
            if children
                .get_mut(&e.from.instance_id)
                .unwrap()
                .insert(e.to.instance_id.clone())
            {
                *incoming.get_mut(&e.to.instance_id).unwrap() += 1;
            }
        }
        let mut ready: BTreeSet<_> = incoming
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(id, _)| id.clone())
            .collect();
        let mut order = Vec::new();
        while let Some(id) = ready.pop_first() {
            order.push(id.clone());
            for child in &children[&id] {
                let n = incoming.get_mut(child).unwrap();
                *n -= 1;
                if *n == 0 {
                    ready.insert(child.clone());
                }
            }
        }
        if order.len() != children.len() {
            return Err(HostError::new(ErrorCode::CyclicDependency, "reactive"));
        }
        Ok(Self { order, children })
    }
    pub fn order(&self) -> &[String] {
        &self.order
    }
    pub fn affected(&self, root: &str) -> BTreeSet<String> {
        let mut found = BTreeSet::new();
        let mut todo = vec![root.to_string()];
        while let Some(id) = todo.pop() {
            if found.insert(id.clone()) {
                if let Some(next) = self.children.get(&id) {
                    todo.extend(next.iter().cloned());
                }
            }
        }
        found
    }
}
pub fn check_fence(revision: u64, generation: u64, context: &DispatchContext) -> Result<()> {
    if revision != context.room_revision || generation != context.execution_generation {
        Err(HostError::new(ErrorCode::StaleResult, "reactive"))
    } else {
        Ok(())
    }
}
