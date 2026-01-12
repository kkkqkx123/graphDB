use std::time::{SystemTime, UNIX_EPOCH};

/// Utility function to generate unique IDs
pub fn generate_id() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64
}

/// Utility function for validating node/edge IDs
pub fn is_valid_id(id: u64) -> bool {
    id != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();

        // IDs should be different
        assert_ne!(id1, id2);

        // IDs should be valid
        assert!(is_valid_id(id1));
        assert!(is_valid_id(id2));
    }

    #[test]
    fn test_is_valid_id() {
        assert!(is_valid_id(1));
        assert!(is_valid_id(42));
        assert!(is_valid_id(u64::MAX));
        assert!(!is_valid_id(0));
    }
}
