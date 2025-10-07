// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    identity_provider::{IdentityProviderImpl, VcardIdentityProvider},
    members::MemberService,
    xmpp::XmppServiceContext,
};

use super::prelude::*;

#[derive(Debug)]
pub struct MockIdentityProvider {
    pub(crate) implem: VcardIdentityProvider,
}

#[async_trait::async_trait]
impl IdentityProviderImpl for MockIdentityProvider {
    async fn get_email_address(
        &self,
        jid: &BareJid,
        member_service: &MemberService,
        ctx: &XmppServiceContext,
    ) -> anyhow::Result<Option<EmailAddress>> {
        self.implem
            .get_email_address(jid, member_service, ctx)
            .await
    }
}
