// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Debug, sync::Arc};

#[cfg(debug_assertions)]
use crate::features::app_config::UuidDependencyMode;
use crate::AppConfig;

#[cfg(debug_assertions)]
use self::incrementing::IncrementingUuidGenerator;
use self::live::LiveUuidGenerator;

/// Inspired by <https://github.com/pointfreeco/swift-dependencies/blob/9620f731d92647de19df2dc4b96c791b0a86a816/Sources/Dependencies/DependencyValues/UUID.swift>.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Uuid {
    generator: Arc<dyn UuidGenerator>,
}

impl Uuid {
    #[cfg(not(debug_assertions))]
    pub fn from_config(_config: &AppConfig) -> Self {
        Self {
            generator: Arc::new(LiveUuidGenerator),
        }
    }

    #[cfg(debug_assertions)]
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            generator: match config.debug_only.dependency_modes.uuid {
                UuidDependencyMode::Normal => Arc::new(LiveUuidGenerator),
                UuidDependencyMode::Incrementing => Arc::new(IncrementingUuidGenerator::new()),
            },
        }
    }

    pub fn new_v4(&self) -> uuid::Uuid {
        self.generator.new_v4()
    }
}

trait UuidGenerator: Send + Sync + Debug {
    fn new_v4(&self) -> uuid::Uuid;
}

mod live {
    use super::UuidGenerator;

    #[derive(Debug)]
    pub(super) struct LiveUuidGenerator;

    impl UuidGenerator for LiveUuidGenerator {
        fn new_v4(&self) -> uuid::Uuid {
            uuid::Uuid::new_v4()
        }
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_live_generator() {
        let uuid_gen = LiveUuidGenerator;
        assert_ne!(uuid_gen.new_v4(), uuid_gen.new_v4());
    }
}

#[cfg(debug_assertions)]
mod incrementing {
    use super::UuidGenerator;
    use std::sync::RwLock;

    #[derive(Debug)]
    pub(super) struct IncrementingUuidGenerator {
        value: RwLock<u32>,
    }

    impl IncrementingUuidGenerator {
        pub(super) fn new() -> Self {
            Self {
                value: RwLock::new(0),
            }
        }
    }

    impl UuidGenerator for IncrementingUuidGenerator {
        fn new_v4(&self) -> uuid::Uuid {
            let mut value = self.value.write().unwrap();
            let res = uuid::Uuid::from_u128(0x40000000000000000000u128 + *value as u128);
            *value += 1;
            res
        }
    }

    #[test]
    fn test_incrementing_generator() {
        use uuid::uuid;

        let uuid_gen = IncrementingUuidGenerator::new();
        assert_eq!(
            uuid_gen.new_v4(),
            uuid!("00000000-0000-4000-0000-000000000000")
        );
        assert_eq!(
            uuid_gen.new_v4(),
            uuid!("00000000-0000-4000-0000-000000000001")
        );
        assert_eq!(
            uuid_gen.new_v4(),
            uuid!("00000000-0000-4000-0000-000000000002")
        );
    }
}
