// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use async_trait::async_trait;
use prose_xmpp::{
    stanza::vcard4::{self, Email, Fn_},
    BareJid, ConnectionError, RequestError,
};

use crate::{
    auth::AuthToken,
    members::Nickname,
    models::{Avatar, AvatarDecodeError, EmailAddress},
};

pub use super::live_xmpp_service::LiveXmppService;

#[derive(Debug, Clone)]
pub struct XmppService {
    pub implem: Arc<dyn XmppServiceImpl>,
}

#[derive(Debug, Clone)]
pub struct XmppServiceContext {
    pub bare_jid: BareJid,
    pub auth_token: AuthToken,
}

pub type VCard = prose_xmpp::stanza::VCard4;

#[async_trait]
pub trait XmppServiceImpl: std::fmt::Debug + Send + Sync {
    async fn get_vcard(
        &self,
        ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<Option<VCard>, XmppServiceError>;

    async fn get_own_vcard(
        &self,
        ctx: &XmppServiceContext,
    ) -> Result<Option<VCard>, XmppServiceError> {
        self.get_vcard(ctx, &ctx.bare_jid).await
    }

    async fn set_own_vcard(
        &self,
        ctx: &XmppServiceContext,
        vcard: &VCard,
    ) -> Result<(), XmppServiceError>;

    async fn create_own_vcard(
        &self,
        ctx: &XmppServiceContext,
        name: &Nickname,
        email: Option<EmailAddress>,
    ) -> Result<(), XmppServiceError> {
        tracing::debug!("Creating {}'s vCard with name '{name}'…", ctx.bare_jid);
        let mut vcard = VCard::new();
        vcard.nickname.push(vcard4::Nickname {
            value: name.to_string(),
        });
        if let Some(email) = email {
            vcard.email.push(Email {
                value: email.to_string(),
            });
        }
        self.set_own_vcard(ctx, &vcard).await
    }

    async fn get_own_nickname(
        &self,
        ctx: &XmppServiceContext,
    ) -> Result<Option<String>, XmppServiceError> {
        let vcard = self.get_own_vcard(ctx).await?.unwrap_or_default();
        Ok(vcard.nickname.first().map(|v| v.value.to_owned()))
    }

    async fn set_own_nickname(
        &self,
        ctx: &XmppServiceContext,
        nickname: &Nickname,
    ) -> Result<(), XmppServiceError> {
        tracing::debug!("Setting {}'s nickname to {nickname}…", ctx.bare_jid);
        let mut vcard = self.get_own_vcard(ctx).await?.unwrap_or_default();
        vcard.nickname = vec![
            vcard4::Nickname {
                value: nickname.to_string(),
            },
        ];
        self.set_own_vcard(ctx, &vcard).await
    }

    async fn get_own_formatted_name(
        &self,
        ctx: &XmppServiceContext,
    ) -> Result<Option<String>, XmppServiceError> {
        let vcard = self.get_own_vcard(ctx).await?.unwrap_or_default();
        Ok(vcard.fn_.first().map(|v| v.value.to_owned()))
    }

    async fn set_own_formatted_name(
        &self,
        ctx: &XmppServiceContext,
        formatted_name: &str,
    ) -> Result<(), XmppServiceError> {
        tracing::debug!(
            "Setting {}'s formatted name to {formatted_name}…",
            ctx.bare_jid
        );
        let mut vcard = self.get_own_vcard(ctx).await?.unwrap_or_default();
        vcard.fn_ = vec![Fn_ {
            value: formatted_name.to_owned(),
        }];
        self.set_own_vcard(ctx, &vcard).await
    }

    async fn get_avatar(
        &self,
        ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<Option<Avatar>, XmppServiceError>;

    async fn get_own_avatar(
        &self,
        ctx: &XmppServiceContext,
    ) -> Result<Option<Avatar>, XmppServiceError> {
        self.get_avatar(ctx, &ctx.bare_jid).await
    }

    async fn set_own_avatar(
        &self,
        ctx: &XmppServiceContext,
        avatar: Avatar,
    ) -> Result<(), XmppServiceError>;

    async fn is_connected(
        &self,
        ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<bool, XmppServiceError>;
}

pub type Error = XmppServiceError;

#[derive(Debug, thiserror::Error)]
pub enum XmppServiceError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] ConnectionError),
    #[error("{0}")]
    RequestError(#[from] RequestError),
    #[error("Internal error: {0}")]
    AvatarDecodeError(#[from] AvatarDecodeError),
    #[error("{0}")]
    Other(String),
}

// MARK: - Boilerplate

impl From<anyhow::Error> for XmppServiceError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(format!("{err:#}"))
    }
}

impl std::ops::Deref for XmppService {
    type Target = Arc<dyn XmppServiceImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}
