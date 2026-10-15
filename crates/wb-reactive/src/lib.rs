//! Structural DAG validation and monotonic revision fence, not a scheduler.
use std::collections::{HashMap, HashSet};
use wb_contracts::Edge;
pub const CONTRACT_VERSION: &str = "0.1";
pub fn validate_acyclic(edges: &[Edge]) -> Result<(), &'static str> {
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for e in edges { graph.entry(&e.source).or_default().push(&e.target); graph.entry(&e.target).or_default(); }
    fn visit<'a>(n: &'a str, g: &HashMap<&'a str, Vec<&'a str>>, done: &mut HashSet<&'a str>, active: &mut HashSet<&'a str>) -> bool {
        if active.contains(n) { return false; }
        if done.contains(n) { return true; }
        active.insert(n);
        for &next in g.get(n).map(Vec::as_slice).unwrap_or(&[]) { if !visit(next, g, done, active) { return false; } }
        active.remove(n); done.insert(n); true
    }
    let (mut done, mut active) = (HashSet::new(), HashSet::new());
    for &node in graph.keys() { if !visit(node, &graph, &mut done, &mut active) { return Err("CYCLIC_DEPENDENCY"); } }
    Ok(())
}
pub fn accept_revision(latest: u64, candidate: u64) -> Result<u64, &'static str> {
    if candidate < latest { Err("STALE_REVISION") } else { Ok(candidate) }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_cycles() { let edges = vec![Edge{source:"a".into(),target:"b".into()}, Edge{source:"b".into(),target:"a".into()}]; assert_eq!(validate_acyclic(&edges), Err("CYCLIC_DEPENDENCY")); }
    #[test]
    fn accepts_dag() { let edges = vec![Edge{source:"a".into(),target:"b".into()}]; assert_eq!(validate_acyclic(&edges), Ok(())); }
    #[test]
    fn rejects_stale() { assert_eq!(accept_revision(10, 9), Err("STALE_REVISION")); }
}
