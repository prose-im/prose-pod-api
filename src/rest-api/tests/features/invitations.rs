// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_pod_api::features::invitations::*;
use service::{
    invitations::{prelude::MemberRole, *},
    members::Nickname,
};

use super::prelude::*;

// MARK: - Given

#[given(expr = "<{email}> has been invited via email")]
async fn given_invited(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) -> Result<(), Error> {
    let email_address = email_address.0;
    let username = JidNode::from(&email_address);

    // Create invitation
    let invitation = world
        .mock_invitation_repository()
        .create_account_invitation(
            CreateAccountInvitationCommand {
                username,
                role: MemberRole::Member,
                email_address: email_address.clone(),
                ttl: None,
            },
            &BYPASS_TOKEN,
        )
        .await?;

    // Store current invitation data
    world.scenario_workspace_invitation = Some((email_address, invitation));

    Ok(())
}

#[given(expr = "<{email}> is pre-assigned the {member_role} role")]
fn given_pre_assigned_role(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
    role: parameters::MemberRole,
) {
    world
        .mock_invitation_repository()
        .state_mut()
        .invitations
        .get_mut(&email_address)
        .expect(&format!("<{email_address}> should have been invited"))
        .pre_assigned_role = role.0;
}

#[given(expr = "{int} people have been invited via email")]
async fn given_n_invited(world: &mut TestWorld, n: u32) -> Result<(), anyhow::Error> {
    let domain = world.server_config().await?.domain;

    for i in 0..n {
        let jid = BareJid::new(&format!("person.{i}@{domain}")).unwrap();
        let email_address = service::models::EmailAddress::from(&jid);
        world
            .mock_invitation_repository()
            .create_account_invitation(
                CreateAccountInvitationCommand {
                    username: JidNode::from(jid.expect_username()),
                    role: MemberRole::Member,
                    email_address,
                    ttl: None,
                },
                &BYPASS_TOKEN,
            )
            .await?;
    }

    Ok(())
}

#[given("an admin resent the invitation")]
async fn given_invitation_resent(world: &mut TestWorld) -> Result<(), anyhow::Error> {
    let (_, invitation) = world.scenario_workspace_invitation();

    // Resend invitation
    world
        .invitation_service()
        .resend_account_invitation(&invitation.id, &BYPASS_TOKEN)
        .await?;

    Ok(())
}

#[given("the invitation has already expired")]
fn given_invitation_expired(world: &mut TestWorld) -> Result<(), MutationError> {
    let (email_address, _) = world.scenario_workspace_invitation();

    // Update invitation.
    let invitation = {
        let mut mock_invitation_repository_state = world.mock_invitation_repository().state_mut();
        let invitation = mock_invitation_repository_state
            .invitations
            .get_mut(&email_address)
            .unwrap();
        invitation.accept_token_expires_at =
            Utc::now().checked_sub_signed(TimeDelta::days(1)).unwrap();
        invitation.clone()
    };

    // Store current invitation data.
    world.scenario_workspace_invitation = Some((email_address, invitation.clone()));

    Ok(())
}

// MARK: - When

async fn invite_member(
    api: &TestServer,
    auth: &AuthToken,
    username: &JidNode,
    pre_assigned_role: parameters::MemberRole,
    contact: InvitationContact,
) -> TestResponse {
    api.post(&"/v1/invitations")
        .add_header(CONTENT_TYPE, "application/json")
        .add_header(AUTHORIZATION, format!("Bearer {}", auth.expose_secret()))
        .json(&json!(InviteMemberRequest {
            username: username.to_owned(),
            pre_assigned_role: pre_assigned_role.0,
            contact,
        }))
        .await
}

async fn list_workspace_invitations(api: &TestServer, auth: &AuthToken) -> TestResponse {
    api.get(&"/v1/invitations")
        .add_header(ACCEPT, "application/json")
        .add_header(AUTHORIZATION, format!("Bearer {}", auth.expose_secret()))
        .await
}

async fn list_workspace_invitations_paged(
    api: &TestServer,
    auth: &AuthToken,
    page_number: u64,
    page_size: u64,
) -> TestResponse {
    api.get(&format!(
        "/v1/invitations?page_number={page_number}&page_size={page_size}"
    ))
    .add_header(ACCEPT, "application/json")
    .add_header(AUTHORIZATION, format!("Bearer {}", auth.expose_secret()))
    .await
}

api_call_fn!(
    get_workspace_invitation_by_token,
    unauthenticated: GET,
    "/v1/invitation-tokens/{token}/details"; token=&str,
);

pub(super) async fn accept_workspace_invitation(
    api: &TestServer,
    token: InvitationToken,
    nickname: Nickname,
    password: Option<Password>,
) -> TestResponse {
    api.put(&format!(
        "/v1/invitation-tokens/{token}/accept",
        token = token.expose_secret()
    ))
    .add_header(CONTENT_TYPE, "application/json")
    .json(&json!(AcceptWorkspaceInvitationRequest {
        nickname,
        password: password.unwrap_or("test".to_string().into()).into(),
        email: None,
    }))
    .add_header(ACCEPT, "application/json")
    .await
}

async fn reject_workspace_invitation(api: &TestServer, token: InvitationToken) -> TestResponse {
    api.put(&format!(
        "/v1/invitation-tokens/{token}/reject",
        token = token.expose_secret()
    ))
    .add_header(ACCEPT, "application/json")
    .await
}

async fn cancel_workspace_invitation(
    api: &TestServer,
    auth: &AuthToken,
    invitation_id: &InvitationId,
) -> TestResponse {
    api.delete(&format!(
        "/v1/invitations/{invitation_id}",
        invitation_id = invitation_id.expose_secret()
    ))
    .add_header(AUTHORIZATION, format!("Bearer {}", auth.expose_secret()))
    .add_header(ACCEPT, "application/json")
    .await
}

async fn workspace_invitation_admin_action(
    api: &TestServer,
    auth: &AuthToken,
    invitation_id: &InvitationId,
    action: &'static str,
) -> TestResponse {
    api.post(&format!(
        "/v1/invitations/{invitation_id}/{action}",
        invitation_id = invitation_id.expose_secret()
    ))
    .add_header(ACCEPT, "application/json")
    .add_header(AUTHORIZATION, format!("Bearer {}", auth.expose_secret()))
    .await
}

#[when(expr = r#"{} invites <{email}> as a(n) {member_role}"#)]
async fn when_inviting(
    world: &mut TestWorld,
    name: String,
    email_address: parameters::EmailAddress,
    pre_assigned_role: parameters::MemberRole,
) {
    let ref auth = world.token(&name).await;
    let email_address = email_address.0;
    let res = invite_member(
        world.api(),
        auth,
        &JidNode::from(email_address.to_owned()),
        pre_assigned_role,
        InvitationContact::Email { email_address },
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "{} lists pending invitations")]
async fn when_listing_workspace_invitations(world: &mut TestWorld, name: String) {
    let ref auth = world.token(&name).await;
    let res = list_workspace_invitations(world.api(), auth).await;
    world.result = Some(res.into());
}

#[when(expr = "{} lists pending invitations by pages of {int}")]
async fn when_listing_workspace_invitations_paged(
    world: &mut TestWorld,
    name: String,
    page_size: u64,
) {
    let ref auth = world.token(&name).await;
    let res = list_workspace_invitations_paged(world.api(), auth, 1, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "{} gets page {int} of pending invitations by pages of {int}")]
async fn when_getting_workspace_invitations_page(
    world: &mut TestWorld,
    name: String,
    page_number: u64,
    page_size: u64,
) {
    let ref auth = world.token(&name).await;
    let res = list_workspace_invitations_paged(world.api(), auth, page_number, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> requests the invitation associated to their accept token")]
async fn when_getting_workspace_invitation_for_accept_token(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) -> Result<(), anyhow::Error> {
    let invitation = world
        .mock_invitation_repository()
        .invitation_for_email(&email_address)
        .unwrap();

    let res =
        get_workspace_invitation_by_token(world.api(), invitation.accept_token.expose_secret())
            .await?;
    world.result = Some(res.into());

    Ok(())
}

#[when(expr = "<{email}> requests the invitation associated to their reject token")]
async fn when_getting_workspace_invitation_for_reject_token(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) -> Result<(), anyhow::Error> {
    let invitation = world
        .mock_invitation_repository()
        .invitation_for_email(&email_address)
        .unwrap();

    let res =
        get_workspace_invitation_by_token(world.api(), invitation.reject_token.expose_secret())
            .await?;
    world.result = Some(res.into());

    Ok(())
}

#[when(expr = "<{email}> requests the invitation associated to their previous accept token")]
async fn when_getting_workspace_invitation_for_previous_accept_token(
    world: &mut TestWorld,
    _email_address: parameters::EmailAddress,
) -> Result<(), anyhow::Error> {
    // SHORTCUT: Invitations are deleted once they expire so let’s just
    //   create a new token and pretend it existed before.
    let previous_token = InvitationToken::from(random_secret(32));

    let res =
        get_workspace_invitation_by_token(world.api(), previous_token.expose_secret()).await?;
    world.result = Some(res.into());

    Ok(())
}

#[when(expr = "<{email}> accepts their invitation")]
async fn when_invited_accepts_invitation(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) -> Result<(), anyhow::Error> {
    let invitation = world
        .mock_invitation_repository()
        .invitation_for_email(&email_address)
        .unwrap();

    let res = accept_workspace_invitation(
        world.api(),
        invitation.accept_token,
        email_address.0.into(),
        None,
    )
    .await;
    world.result = Some(res.into());

    Ok(())
}

#[when(expr = "<{email}> accepts their invitation using the nickname {string}")]
async fn when_invited_accepts_invitation_with_nickname(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
    nickname: String,
) -> Result<(), anyhow::Error> {
    let invitation = world
        .mock_invitation_repository()
        .invitation_for_email(&email_address)
        .unwrap();
    let nickname = Nickname::from_string_unsafe(nickname);

    let res =
        accept_workspace_invitation(world.api(), invitation.accept_token, nickname, None).await;
    world.result = Some(res.into());

    Ok(())
}

#[when(expr = "<{email}> uses the previous invitation accept link they received")]
async fn when_invited_uses_old_accept_link(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) -> Result<(), anyhow::Error> {
    // SHORTCUT: Invitations are deleted once they expire so let’s just
    //   create a new token and pretend it existed before.
    let previous_token = InvitationToken::from(random_secret(32));

    let res =
        accept_workspace_invitation(world.api(), previous_token, email_address.0.into(), None)
            .await;
    world.result = Some(res.into());

    Ok(())
}

#[when(expr = "<{email}> rejects their invitation")]
async fn when_invited_rejects_invitation(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) -> Result<(), anyhow::Error> {
    let invitation = world
        .mock_invitation_repository()
        .invitation_for_email(&email_address)
        .unwrap();

    let res = reject_workspace_invitation(world.api(), invitation.reject_token).await;
    world.result = Some(res.into());

    Ok(())
}

#[when(expr = "{} resends the invitation")]
async fn when_user_resends_workspace_invitation(world: &mut TestWorld, name: String) {
    let ref auth = world.token(&name).await;
    let invitation = world.scenario_workspace_invitation().1;
    let res = workspace_invitation_admin_action(world.api(), auth, &invitation.id, "resend").await;
    world.result = Some(res.into());
}

#[when(expr = "{} cancels the invitation")]
async fn when_user_cancels_workspace_invitation(world: &mut TestWorld, name: String) {
    let ref auth = world.token(&name).await;
    let invitation = world.scenario_workspace_invitation().1;
    let res = cancel_workspace_invitation(world.api(), auth, &invitation.id).await;
    world.result = Some(res.into());
}

// MARK: - Then

#[then(expr = "they should see {int} pending invitation(s)")]
fn then_n_pending_invitations(world: &mut TestWorld, n: usize) {
    let res: Vec<WorkspaceInvitationDto> = world.result().json();
    assert_eq!(res.len(), n)
}

#[then(expr = "there should be an invitation for <{email}> in the database")]
async fn then_invitation_for_email(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) -> Result<(), DbErr> {
    let invitation = world
        .mock_invitation_repository()
        .invitation_for_email(&email_address);

    assert!(invitation.is_some());

    Ok(())
}

#[then(expr = "there should not be any invitation for <{email}> in the database")]
async fn then_no_invitation_for_email(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) -> Result<(), Error> {
    let invitation = world
        .mock_invitation_repository()
        .invitation_for_email(&email_address);

    assert!(invitation.is_none());

    Ok(())
}
