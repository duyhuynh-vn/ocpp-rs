//! ocpp-simulator implementation
pub fn hello() -> String {
    "Hello from ocpp-simulator".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert!(!hello().is_empty());
    }
}
