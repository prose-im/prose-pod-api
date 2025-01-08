// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

//! Inspired from [valeriansaliou/vigil](https://github.com/valeriansaliou/vigil)'s
//! [Notifiers](https://github.com/valeriansaliou/vigil/tree/master/src/notifier).

pub mod email;
mod generic;
#[cfg(debug_assertions)]
mod logging;

use std::{fmt::Debug, ops::Deref, sync::Arc};

use crate::app_config::AppConfig;

pub use self::generic::*;
#[cfg(debug_assertions)]
use self::logging::LoggingNotifier;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Notifier<N: NotificationTrait> {
    implem: Arc<dyn GenericNotifier<Notification = N>>,
}

impl<N: NotificationTrait + 'static> Notifier<N> {
    #[cfg(not(debug_assertions))]
    pub fn from_config<Notifier, Error>(app_config: &AppConfig) -> Result<Self, Error>
    where
        Notifier: GenericNotifier<Notification = N>
            + for<'a> TryFrom<&'a AppConfig, Error = Error>
            + 'static,
    {
        Ok(Self::from(Notifier::try_from(app_config)?))
    }

    #[cfg(debug_assertions)]
    pub fn from_config<Notifier, Error>(app_config: &AppConfig) -> Result<Self, Error>
    where
        Notifier: GenericNotifier<Notification = N>
            + for<'a> TryFrom<&'a AppConfig, Error = Error>
            + 'static,
    {
        use crate::app_config::NotifierDependencyMode;

        Ok(match app_config.debug_only.dependency_modes.notifier {
            NotifierDependencyMode::Live => Self::from(Notifier::try_from(app_config)?),
            NotifierDependencyMode::Logging => Self::from(LoggingNotifier::default()),
        })
    }
}

impl<N: NotificationTrait> From<Arc<dyn GenericNotifier<Notification = N>>> for Notifier<N> {
    fn from(implem: Arc<dyn GenericNotifier<Notification = N>>) -> Self {
        Self { implem }
    }
}

impl<N, Generic> From<Generic> for Notifier<N>
where
    N: NotificationTrait,
    Generic: GenericNotifier<Notification = N> + 'static,
{
    fn from(implem: Generic) -> Self {
        Self {
            implem: Arc::new(implem),
        }
    }
}

impl<N: NotificationTrait> Deref for Notifier<N> {
    type Target = Arc<dyn GenericNotifier<Notification = N>>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}
