//! Kernel facade: no execution permitted until runtime, capabilities and plans are validated.
pub const CONTRACT_VERSION: &str = "0.1";
pub fn execute_unvalidated_plan() -> Result<(), &'static str> { Err("PLAN_NOT_VALIDATED") }
#[cfg(test)]
mod tests { #[test] fn refuses_unvalidated() { assert!(super::execute_unvalidated_plan().is_err()); } }
