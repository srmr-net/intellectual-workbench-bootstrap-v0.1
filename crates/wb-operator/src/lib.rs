//! ADR-005 bounded untrusted suggestions. This crate grants no execution authority.
pub mod catalog;
pub mod provider;
pub mod suggestion;
pub const CONTRACT_VERSION: &str = "0.1";

pub mod proposal;
