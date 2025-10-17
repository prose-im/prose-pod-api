// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{identity_provider::IdentityProviderImpl, xmpp::XmppServiceContext};

use super::prelude::*;

#[derive(Debug)]
pub struct MockIdentityProvider {
    pub(crate) state: Arc<RwLock<MockIdentityProviderState>>,
    pub(crate) mock_user_repository_state: Arc<RwLock<MockUserRepositoryState>>,
}

#[derive(Debug, Default)]
pub(crate) struct MockIdentityProviderState {
    pub recovery_emails: HashMap<BareJid, EmailAddress>,
}

#[async_trait::async_trait]
impl IdentityProviderImpl for MockIdentityProvider {
    async fn get_public_email_address(
        &self,
        jid: &BareJid,
        _ctx: &XmppServiceContext,
    ) -> Result<Option<EmailAddress>, anyhow::Error> {
        let state = self.mock_user_repository_state.read().unwrap();

        let email_opt = state
            .users
            .get(jid)
            .map(|data| data.email_address.clone())
            .flatten();

        Ok(email_opt)
    }

    async fn get_recovery_email_address(
        &self,
        jid: &BareJid,
        _ctx: &XmppServiceContext,
    ) -> Result<Option<EmailAddress>, anyhow::Error> {
        let state = self.state.read().unwrap();

        let email_opt = state.recovery_emails.get(jid);

        Ok(email_opt.cloned())
    }

    async fn set_recovery_email_address(
        &self,
        jid: &BareJid,
        email_address: EmailAddress,
        _ctx: &XmppServiceContext,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        let mut state = self.state.write().unwrap();

        state.recovery_emails.insert(jid.clone(), email_address);

        Ok(())
    }
}
