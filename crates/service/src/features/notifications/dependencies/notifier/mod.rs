// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

//! Inspired from [valeriansaliou/vigil](https://github.com/valeriansaliou/vigil)'s
//! [Notifiers](https://github.com/valeriansaliou/vigil/tree/master/src/notifier).

mod email;
mod generic;
#[cfg(debug_assertions)]
mod logging;

use std::{fmt::Debug, ops::Deref, sync::Arc};

use crate::app_config::AppConfig;

use self::email::EmailNotifier;
pub use self::generic::*;

#[cfg(debug_assertions)]
use self::logging::LoggingNotifier;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Notifier {
    implem: Arc<dyn GenericNotifier>,
}

impl Notifier {
    pub fn live(config: &AppConfig) -> Result<Self, String> {
        Ok(Self::from(EmailNotifier::new(config)?))
    }

    #[cfg(not(debug_assertions))]
    pub fn from_config(config: &AppConfig) -> Result<Self, String> {
        Self::live(config)
    }

    #[cfg(debug_assertions)]
    pub fn from_config(config: &AppConfig) -> Result<Self, String> {
        use crate::app_config::NotifierDependencyMode;

        match config.debug_only.dependency_modes.notifier {
            NotifierDependencyMode::Live => Self::live(config),
            NotifierDependencyMode::Logging => Ok(Self::from(LoggingNotifier)),
        }
    }
}

impl From<Arc<dyn GenericNotifier>> for Notifier {
    fn from(implem: Arc<dyn GenericNotifier>) -> Self {
        Self { implem }
    }
}

impl<N: GenericNotifier + 'static> From<N> for Notifier {
    fn from(implem: N) -> Self {
        Self {
            implem: Arc::new(implem),
        }
    }
}

impl Deref for Notifier {
    type Target = Arc<dyn GenericNotifier>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}
