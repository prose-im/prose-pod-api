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
    async fn get_public_email_address(
        &self,
        jid: &BareJid,
        ctx: &XmppServiceContext,
    ) -> Result<Option<EmailAddress>, anyhow::Error>;

    async fn get_recovery_email_address(
        &self,
        jid: &BareJid,
        ctx: &XmppServiceContext,
    ) -> Result<Option<EmailAddress>, anyhow::Error>;

    async fn get_recovery_email_address_with_fallback(
        &self,
        jid: &BareJid,
        ctx: &XmppServiceContext,
    ) -> Result<Option<EmailAddress>, anyhow::Error> {
        // Look for a dedicated password recovery email address.
        if let recovery_email @ Some(_) = self.get_recovery_email_address(jid, ctx).await? {
            return Ok(recovery_email);
        }

        // Fallback to public email address (e.g. in vCard).
        self.get_public_email_address(jid, ctx).await
    }

    async fn set_recovery_email_address(
        &self,
        jid: &BareJid,
        email_address: EmailAddress,
        ctx: &XmppServiceContext,
    ) -> Result<(), Either<Forbidden, anyhow::Error>>;
}

mod live {
    use crate::{
        members::VCardData, models::DatabaseRwConnectionPools, prosody::ProsodyHttpAdminApi,
    };

    use super::*;

    #[derive(Debug)]
    pub struct LiveIdentityProvider {
        pub db: DatabaseRwConnectionPools,
        pub member_service: MemberService,
        pub admin_api: Arc<ProsodyHttpAdminApi>,
    }

    #[async_trait::async_trait]
    impl IdentityProviderImpl for LiveIdentityProvider {
        async fn get_public_email_address(
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

        async fn get_recovery_email_address(
            &self,
            jid: &BareJid,
            _ctx: &XmppServiceContext,
        ) -> Result<Option<EmailAddress>, anyhow::Error> {
            use crate::auth::recovery_emails_store as store;

            let todo = "Migrate data from previous table to KV store";
            store::get_typed(&self.db.read, jid.as_str()).await
        }

        async fn set_recovery_email_address(
            &self,
            jid: &BareJid,
            email_address: EmailAddress,
            _ctx: &XmppServiceContext,
        ) -> Result<(), Either<Forbidden, anyhow::Error>> {
            crate::auth::recovery_emails_store::set_typed(
                &self.db.write,
                jid.as_str(),
                email_address,
            )
            .await?;

            Ok(())
        }
    }
}
