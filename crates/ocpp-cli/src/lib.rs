//! ocpp-cli implementation
pub fn hello() -> String {
    "Hello from ocpp-cli".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert!(!hello().is_empty());
    }
}
