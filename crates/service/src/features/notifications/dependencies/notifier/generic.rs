// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display};

use tracing::instrument;

pub(super) const DISPATCH_TIMEOUT_SECONDS: u64 = 10;

pub trait NotificationTrait: Display + Debug + Sync + Send {}

pub trait GenericNotifier: Debug + Sync + Send {
    type Notification: NotificationTrait;

    fn name(&self) -> &'static str;
    fn attempt(&self, notification: &Self::Notification) -> Result<(), NotifierError>;

    #[instrument(
        level = "trace",
        skip_all, fields(notifier = self.name()),
        err,
    )]
    fn dispatch(&self, notification: &Self::Notification) -> Result<(), NotifierError> {
        // Attempt notification dispatch
        self.attempt(notification)?;

        // TODO: Implement retries. See <https://github.com/valeriansaliou/vigil/blob/master/src/notifier/generic.rs>.

        Ok(())
    }
}

#[repr(transparent)]
#[derive(Debug, Clone)]
#[derive(thiserror::Error)]
#[error("{0}")]
pub struct NotifierError(pub String);
