// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;

use async_trait::async_trait;
use prosody_http::admin_api::{ProsodyAdminApi, UserInfo};
use time::OffsetDateTime;

use crate::{
    auth::AuthToken,
    members::{Member, MemberRole},
    prosody::ProsodyRoleName,
    util::JidExt,
    xmpp::{jid::BareJid, NonStandardXmppClient},
};

#[async_trait]
impl NonStandardXmppClient for ProsodyAdminApi {
    async fn is_connected(&self, jid: &BareJid, auth: &AuthToken) -> Result<bool, anyhow::Error> {
        let user_opt = self.get_user_by_name(jid.expect_username(), auth).await?;
        match user_opt {
            Some(UserInfo {
                last_active: Some(last_active),
                ..
            }) => {
                // `user.last_active` is set to “now” if the user is connected,
                // but “now” is on the Server, and time passed because of
                // serialization, networking, deserialization, etc. We can
                // consider that if a user was “last active” in the past
                // 5 seconds it means they are connected.
                let considered_active_if_logged_in_after =
                    OffsetDateTime::now_utc() - Duration::from_secs(5);
                Ok(last_active > considered_active_if_logged_in_after)
            }
            None
            | Some(UserInfo {
                last_active: None, ..
            }) => Ok(false),
        }
    }
}

// MARK: - Boilerplate

impl From<&UserInfo> for Member {
    fn from(info: &UserInfo) -> Self {
        let role: ProsodyRoleName = info.role.clone().expect("Members should have roles").into();

        Self {
            role: MemberRole::try_from(&role)
                .inspect_err(|e| {
                    tracing::warn!("Unsupported role for `{jid}`: {e}", jid = &info.jid)
                })
                .ok(),
            jid: info.jid.clone(),
        }
    }
}

impl From<UserInfo> for Member {
    fn from(info: UserInfo) -> Self {
        Self::from(&info)
    }
}
