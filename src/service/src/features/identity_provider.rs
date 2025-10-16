// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use crate::{
    errors::Forbidden,
    members::MemberService,
    models::{xmpp::BareJid, EmailAddress},
    util::either::Either,
    xmpp::XmppServiceContext,
};

pub use self::live::LiveIdentityProvider;

#[derive(Debug, Clone)]
pub struct IdentityProvider {
    pub implem: Arc<dyn IdentityProviderImpl>,
}

impl IdentityProvider {
    pub fn new(implem: Arc<dyn IdentityProviderImpl>) -> Self {
        Self { implem }
    }
}

impl std::ops::Deref for IdentityProvider {
    type Target = Arc<dyn IdentityProviderImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}

#[async_trait::async_trait]
pub trait IdentityProviderImpl: std::fmt::Debug + Sync + Send {
    async fn get_email_address(
        &self,
        jid: &BareJid,
        ctx: &XmppServiceContext,
    ) -> Result<Option<EmailAddress>, anyhow::Error>;

    async fn set_email_address(
        &self,
        jid: &BareJid,
        email_address: EmailAddress,
        ctx: &XmppServiceContext,
    ) -> Result<(), Either<Forbidden, anyhow::Error>>;
}

mod live {
    use crate::{members::VCardData, prosody::ProsodyHttpAdminApi, util::JidExt as _};

    use super::*;

    #[derive(Debug)]
    pub struct LiveIdentityProvider {
        pub member_service: MemberService,
        pub admin_api: Arc<ProsodyHttpAdminApi>,
    }

    #[async_trait::async_trait]
    impl IdentityProviderImpl for LiveIdentityProvider {
        async fn get_email_address(
            &self,
            jid: &BareJid,
            ctx: &XmppServiceContext,
        ) -> Result<Option<EmailAddress>, anyhow::Error> {
            use std::str::FromStr as _;

            match self.member_service.get_vcard(jid, ctx).await {
                Some(VCardData {
                    email: Some(email_address),
                    ..
                }) => match EmailAddress::from_str(&email_address) {
                    Ok(address) => Ok(Some(address)),
                    Err(err) => Err(anyhow::Error::new(err)
                        .context(format!("Email address in `{jid}` vCard is invalid."))),
                },

                Some(VCardData { email: None, .. }) => {
                    tracing::warn!("vCard for `{jid}` contains no email address.");
                    Ok(None)
                }

                None => {
                    tracing::warn!("`{jid}` has no vCard.");
                    Ok(None)
                }
            }
        }

        async fn set_email_address(
            &self,
            jid: &BareJid,
            email_address: EmailAddress,
            ctx: &XmppServiceContext,
        ) -> Result<(), Either<Forbidden, anyhow::Error>> {
            use crate::prosody::prosody_http_admin_api::UpdateUserInfoRequest;

            self.admin_api
                .update_user(
                    jid.expect_username(),
                    &UpdateUserInfoRequest {
                        email: Some(email_address),
                        ..Default::default()
                    },
                    &ctx.auth_token,
                )
                .await
        }
    }
}
