//! Prototype policy boundary. This only rejects operations; it is not an OS sandbox.
pub const CONTRACT_VERSION: &str = "0.1";
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Denial {
    pub operation: String,
    pub reason: &'static str,
}
pub fn authorize_unimplemented(operation: &str) -> Result<(), Denial> {
    Err(Denial {
        operation: operation.to_owned(),
        reason: "CAPABILITY_DENIED",
    })
}
#[cfg(test)]
mod tests {
    #[test]
    fn deny_by_default() {
        assert_eq!(
            super::authorize_unimplemented("filesystem.read")
                .unwrap_err()
                .reason,
            "CAPABILITY_DENIED"
        );
    }
}
