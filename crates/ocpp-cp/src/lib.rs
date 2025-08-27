//! ocpp-cp implementation
pub fn hello() -> String {
    "Hello from ocpp-cp".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert!(!hello().is_empty());
    }
}
