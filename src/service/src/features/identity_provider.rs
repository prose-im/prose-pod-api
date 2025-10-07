// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use crate::{
    members::MemberService,
    models::{xmpp::BareJid, EmailAddress},
    xmpp::XmppServiceContext,
};

pub use self::vcard::VcardIdentityProvider;

#[derive(Debug, Clone)]
pub struct IdentityProvider {
    implem: Arc<dyn IdentityProviderImpl>,
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
        // TODO: Remove this, as it won’t make sense with other IdPs we might
        //   support in the future. At the moment it’s non-trivial because of
        //   how `XmppService` works and how state is passed around in the API,
        //   but we’ll get there eventually.
        member_service: &MemberService,
        ctx: &XmppServiceContext,
    ) -> anyhow::Result<Option<EmailAddress>>;
}

mod vcard {
    use crate::members::VCardData;

    use super::*;

    #[derive(Debug, Default)]
    pub struct VcardIdentityProvider;

    #[async_trait::async_trait]
    impl IdentityProviderImpl for VcardIdentityProvider {
        async fn get_email_address(
            &self,
            jid: &BareJid,
            member_service: &MemberService,
            ctx: &XmppServiceContext,
        ) -> anyhow::Result<Option<EmailAddress>> {
            use std::str::FromStr as _;

            match member_service.get_vcard(jid, ctx).await {
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
    }
}
