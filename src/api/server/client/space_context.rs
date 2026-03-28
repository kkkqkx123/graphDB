use parking_lot::RwLock;
use std::sync::Arc;

use super::session::SpaceInfo;

#[derive(Debug)]
pub struct SpaceContext {
    space: Arc<RwLock<Option<SpaceInfo>>>,
}

impl Default for SpaceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SpaceContext {
    pub fn new() -> Self {
        Self {
            space: Arc::new(RwLock::new(None)),
        }
    }

    pub fn space(&self) -> Option<SpaceInfo> {
        self.space.read().clone()
    }

    pub fn set_space(&self, space: SpaceInfo) {
        *self.space.write() = Some(space);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_context() {
        let context = SpaceContext::new();
        assert!(context.space().is_none());

        let space_info = SpaceInfo {
            name: "test_space".to_string(),
            id: 456,
        };
        context.set_space(space_info.clone());

        assert_eq!(context.space().expect("space should exist").id, 456);
        assert_eq!(
            context.space().expect("space should exist").name,
            "test_space"
        );
    }
}
