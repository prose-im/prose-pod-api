// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::members::{member_service::prelude::*, Nickname};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MockUserService {
    pub server: Arc<MockServerService>,
    pub mock_user_repository: Arc<MockUserRepository>,
    pub mock_auth_service: Arc<MockAuthService>,
    pub server_domain: JidDomain,
}

#[async_trait]
impl UserApplicationServiceImpl for MockUserService {
    async fn create_first_acount(
        &self,
        username: &NodeRef,
        password: &Password,
    ) -> Result<CreateAccountResponse, Either<FirstAccountAlreadyCreated, anyhow::Error>> {
        self.server.check_online()?;

        let jid = BareJid::from_parts(Some(username), &self.server_domain);
        let email_address = EmailAddress::from(&jid);
        let role = MemberRole::Admin.as_prosody();
        self.mock_user_repository
            .add_user(
                &Nickname::from_string_unsafe(username.to_string()),
                UserAccount {
                    jid,
                    password: password.to_owned(),
                    role: role.clone(),
                    joined_at: Utc::now(),
                    email_address: Some(email_address),
                },
                &self.mock_auth_service,
            )
            .await?;

        Ok(CreateAccountResponse {
            username: JidNode::from(username),
            role,
        })
    }
}
