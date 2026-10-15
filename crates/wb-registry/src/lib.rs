//! wb-registry: bootstrap placeholder pending versioned public contract.
pub const CONTRACT_VERSION: &str = "0.1";
#[cfg(test)]
mod tests {
    #[test]
    fn version_is_declared() { assert_eq!(super::CONTRACT_VERSION, "0.1"); }
}
