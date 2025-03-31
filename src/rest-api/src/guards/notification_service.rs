// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::error;

use super::prelude::*;

impl FromRequestParts<AppState> for service::notifications::NotificationService {
    type Rejection = Error;

    async fn from_request_parts(
        _parts: &mut request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(email_notifier) = state.email_notifier.as_ref() else {
            return Err(Error::from(error::MissingConfig {
                config_id: "email_notifier",
            }));
        };
        Ok(Self::new(email_notifier.clone()))
    }
}
