//! ADR-005 powerless providers and trusted host composition. Only the privately
//! owned Session may turn explicit user consent into accepted kernel execution.
pub mod catalog;
pub mod provider;
pub mod suggestion;
pub const CONTRACT_VERSION: &str = "0.1";

pub mod proposal;

pub mod session;

#[cfg(test)]
mod proposal_tests;
