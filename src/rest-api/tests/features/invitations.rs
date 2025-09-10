// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_pod_api::features::invitations::*;
use service::{invitations::*, members::Nickname};

use super::prelude::*;

async fn invite_member(
    api: &TestServer,
    token: &SecretString,
    username: &JidNode,
    pre_assigned_role: parameters::MemberRole,
    contact: InvitationContact,
) -> TestResponse {
    api.post(&"/v1/invitations")
        .add_header(CONTENT_TYPE, "application/json")
        .add_header(AUTHORIZATION, format!("Bearer {}", token.expose_secret()))
        .json(&json!(InviteMemberRequest {
            username: username.to_owned(),
            pre_assigned_role: pre_assigned_role.0,
            contact,
        }))
        .await
}

async fn list_workspace_invitations(api: &TestServer, token: SecretString) -> TestResponse {
    api.get(&"/v1/invitations")
        .add_header(ACCEPT, "application/json")
        .add_header(AUTHORIZATION, format!("Bearer {}", token.expose_secret()))
        .await
}

async fn list_workspace_invitations_paged(
    api: &TestServer,
    token: SecretString,
    page_number: u64,
    page_size: u64,
) -> TestResponse {
    api.get(&format!(
        "/v1/invitations?page_number={page_number}&page_size={page_size}"
    ))
    .add_header(ACCEPT, "application/json")
    .add_header(AUTHORIZATION, format!("Bearer {}", token.expose_secret()))
    .await
}

async fn get_workspace_invitation_by_token(
    api: &TestServer,
    token: &Uuid,
    token_type: InvitationTokenType,
) -> TestResponse {
    api.get(&format!(
        "/v1/invitation-tokens/{token}/details?token_type={token_type}"
    ))
    .add_header(ACCEPT, "application/json")
    .await
}

pub(super) async fn accept_workspace_invitation(
    api: &TestServer,
    token: Uuid,
    nickname: Nickname,
    password: Option<SecretString>,
) -> TestResponse {
    api.put(&format!("/v1/invitation-tokens/{token}/accept"))
        .add_header(CONTENT_TYPE, "application/json")
        .json(&json!(AcceptWorkspaceInvitationRequest {
            nickname,
            password: password.unwrap_or("test".to_string().into()).into(),
        }))
        .add_header(ACCEPT, "application/json")
        .await
}

async fn reject_workspace_invitation(api: &TestServer, token: Uuid) -> TestResponse {
    api.put(&format!("/v1/invitation-tokens/{token}/reject"))
        .add_header(ACCEPT, "application/json")
        .await
}

async fn cancel_workspace_invitation(
    api: &TestServer,
    token: SecretString,
    invitation_id: i32,
) -> TestResponse {
    api.delete(&format!("/v1/invitations/{invitation_id}"))
        .add_header(AUTHORIZATION, format!("Bearer {}", token.expose_secret()))
        .add_header(ACCEPT, "application/json")
        .await
}

async fn workspace_invitation_admin_action(
    api: &TestServer,
    token: SecretString,
    invitation_id: i32,
    action: &'static str,
) -> TestResponse {
    api.post(&format!("/v1/invitations/{invitation_id}/{action}"))
        .add_header(ACCEPT, "application/json")
        .add_header(AUTHORIZATION, format!("Bearer {}", token.expose_secret()))
        .await
}

#[given(expr = "<{email}> has been invited via email")]
async fn given_invited(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) -> Result<(), Error> {
    let domain = world.server_config().await?.domain;
    let email_address = email_address.0;
    let username = email_address.local_part();
    let jid = BareJid::new(&format!("{username}@{domain}")).unwrap();

    // Create invitation
    let model = InvitationRepository::create(
        world.db(),
        InvitationCreateForm {
            jid,
            pre_assigned_role: None,
            contact: InvitationContact::Email {
                email_address: email_address.to_owned(),
            },
            created_at: None,
        },
        &world.uuid_gen,
    )
    .await?;

    // Store current invitation data
    world
        .workspace_invitations
        .insert(email_address.clone(), model.clone());
    world.scenario_workspace_invitation = Some((email_address, model));

    Ok(())
}

#[given(expr = "<{email}> is pre-assigned the {member_role} role")]
async fn given_pre_assigned_role(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
    role: parameters::MemberRole,
) -> Result<(), DbErr> {
    let db = world.db();

    let mut active = world
        .workspace_invitation(&email_address)
        .into_active_model();
    active.pre_assigned_role = Set(role.0);
    active.update(db).await?;

    Ok(())
}

#[given(expr = "{int} people have been invited via email")]
async fn given_n_invited(world: &mut TestWorld, n: u32) -> Result<(), Error> {
    let domain = world.server_config().await?.domain;
    for i in 0..n {
        let email_address =
            service::models::EmailAddress::from_str(&format!("person.{i}@{domain}")).unwrap();
        let model = InvitationRepository::create(
            world.db(),
            InvitationCreateForm {
                jid: BareJid::new(&format!("person.{i}@{domain}")).unwrap(),
                pre_assigned_role: None,
                contact: InvitationContact::Email {
                    email_address: email_address.to_owned(),
                },
                created_at: None,
            },
            &world.uuid_gen,
        )
        .await?;
        world.workspace_invitations.insert(email_address, model);
    }
    Ok(())
}

#[given(expr = "<{email}> has received their invitation")]
async fn given_invitation_received(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) -> Result<(), MutationError> {
    let db = world.db();
    InvitationRepository::update_status_by_email(
        db,
        email_address.0,
        service::invitations::InvitationStatus::Sent,
    )
    .await?;
    Ok(())
}

#[given("an admin resent the invitation")]
async fn given_invitation_resent(world: &mut TestWorld) -> Result<(), MutationError> {
    let (email_address, invitation_before) = world.scenario_workspace_invitation();

    // Store previous accept token for other steps requiring it
    world
        .previous_workspace_invitation_accept_tokens
        .insert(email_address.clone(), invitation_before.accept_token);

    // Resend invitation
    let db = world.db();
    let model = InvitationRepository::resend(db, &world.uuid_gen, invitation_before).await?;

    // Store current invitation data
    world
        .workspace_invitations
        .insert(email_address.clone(), model.clone());
    world.scenario_workspace_invitation = Some((email_address, model));

    Ok(())
}

#[given("the invitation has already expired")]
async fn given_invitation_expired(world: &mut TestWorld) -> Result<(), MutationError> {
    let db = world.db();
    let (email_address, invitation_before) = world.scenario_workspace_invitation();

    // Update invitation
    let mut active = invitation_before.into_active_model();
    active.accept_token_expires_at =
        Set(Utc::now().checked_sub_signed(TimeDelta::days(1)).unwrap());
    let model = active.update(db).await?;

    // Store current invitation data
    world
        .workspace_invitations
        .insert(email_address.clone(), model.clone());
    world.scenario_workspace_invitation = Some((email_address, model));

    Ok(())
}

#[given("the invitation did not go through")]
async fn given_invitation_not_received(world: &mut TestWorld) -> Result<(), MutationError> {
    let db = world.db();
    InvitationRepository::update_status(
        db,
        world.scenario_workspace_invitation().1,
        service::invitations::InvitationStatus::SendFailed,
    )
    .await?;
    Ok(())
}

#[when(expr = r#"{} invites <{email}> as a(n) {member_role}"#)]
async fn when_inviting(
    world: &mut TestWorld,
    name: String,
    email_address: parameters::EmailAddress,
    pre_assigned_role: parameters::MemberRole,
) {
    let token = world.token(&name);
    let email_address = email_address.0;
    let res = invite_member(
        world.api(),
        &token,
        &JidNode::from(email_address.to_owned()),
        pre_assigned_role,
        InvitationContact::Email { email_address },
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "{} lists pending invitations")]
async fn when_listing_workspace_invitations(world: &mut TestWorld, name: String) {
    let token = world.token(&name);
    let res = list_workspace_invitations(world.api(), token).await;
    world.result = Some(res.into());
}

#[when(expr = "{} lists pending invitations by pages of {int}")]
async fn when_listing_workspace_invitations_paged(
    world: &mut TestWorld,
    name: String,
    page_size: u64,
) {
    let token = world.token(&name);
    let res = list_workspace_invitations_paged(world.api(), token, 1, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "{} gets page {int} of pending invitations by pages of {int}")]
async fn when_getting_workspace_invitations_page(
    world: &mut TestWorld,
    name: String,
    page_number: u64,
    page_size: u64,
) {
    let token = world.token(&name);
    let res = list_workspace_invitations_paged(world.api(), token, page_number, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> requests the invitation associated to their accept token")]
async fn when_getting_workspace_invitation_for_accept_token(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) {
    let invitation = world.workspace_invitation(&email_address);
    let res = get_workspace_invitation_by_token(
        world.api(),
        &invitation.accept_token,
        InvitationTokenType::Accept,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> requests the invitation associated to their reject token")]
async fn when_getting_workspace_invitation_for_reject_token(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) {
    let invitation = world.workspace_invitation(&email_address);
    let res = get_workspace_invitation_by_token(
        world.api(),
        &invitation.reject_token,
        InvitationTokenType::Reject,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> requests the invitation associated to their previous accept token")]
async fn when_getting_workspace_invitation_for_previous_accept_token(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) {
    let res = get_workspace_invitation_by_token(
        world.api(),
        &world.previous_workspace_invitation_accept_token(&email_address),
        InvitationTokenType::Accept,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> accepts their invitation")]
async fn when_invited_accepts_invitation(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) {
    let invitation = world.workspace_invitation(&email_address.0);
    let res = accept_workspace_invitation(
        world.api(),
        invitation.accept_token,
        email_address.0.into(),
        None,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> accepts their invitation using the nickname {string}")]
async fn when_invited_accepts_invitation_with_nickname(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
    nickname: String,
) {
    let invitation = world.workspace_invitation(&email_address.0);
    let nickname = Nickname::from_string_unsafe(nickname);
    let res =
        accept_workspace_invitation(world.api(), invitation.accept_token, nickname, None).await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> uses the previous invitation accept link they received")]
async fn when_invited_uses_old_accept_link(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) {
    let res = accept_workspace_invitation(
        world.api(),
        world.previous_workspace_invitation_accept_token(&email_address),
        email_address.0.into(),
        None,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> rejects their invitation")]
async fn when_invited_rejects_invitation(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) {
    let invitation = world.workspace_invitation(&email_address.0);
    let res = reject_workspace_invitation(world.api(), invitation.reject_token).await;
    world.result = Some(res.into());
}

#[when(expr = "{} resends the invitation")]
async fn when_user_resends_workspace_invitation(world: &mut TestWorld, name: String) {
    let token = world.token(&name);
    let invitation = world.scenario_workspace_invitation().1;
    let res = workspace_invitation_admin_action(world.api(), token, invitation.id, "resend").await;
    world.result = Some(res.into());
}

#[when(expr = "{} cancels the invitation")]
async fn when_user_cancels_workspace_invitation(world: &mut TestWorld, name: String) {
    let token = world.token(&name);
    let invitation = world.scenario_workspace_invitation().1;
    let res = cancel_workspace_invitation(world.api(), token, invitation.id).await;
    world.result = Some(res.into());
}

#[then(expr = "they should see {int} pending invitation(s)")]
fn then_n_pending_invitations(world: &mut TestWorld, n: usize) {
    let res: Vec<WorkspaceInvitationDto> = world.result().json();
    assert_eq!(res.len(), n)
}

#[then(expr = "{int} invitation(s) should be {invitation_status}")]
fn then_n_invitations_with_status(
    world: &mut TestWorld,
    n: usize,
    invitation_status: parameters::InvitationStatus,
) {
    let res: Vec<WorkspaceInvitationDto> = world.result().json();
    let filtered = res.iter().filter(|m| m.status == invitation_status.0);
    assert_eq!(filtered.count(), n)
}

#[then(expr = "there should be an invitation for <{email}> in the database")]
async fn then_invitation_for_email(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) -> Result<(), DbErr> {
    let count =
        InvitationRepository::count_for_email_address(world.db(), email_address.0.clone()).await?;
    assert_eq!(
        count, 1,
        "Found {count} workspace_invitation(s) for <{email_address}> in the database"
    );
    Ok(())
}

#[then(expr = "there should not be any invitation for <{email}> in the database")]
async fn then_no_invitation_for_email(
    world: &mut TestWorld,
    email_address: parameters::EmailAddress,
) -> Result<(), Error> {
    let count =
        InvitationRepository::count_for_email_address(world.db(), email_address.0.clone()).await?;
    let server_domain = world.server_config().await?.domain;
    assert_eq!(
        count, 0,
        "Found {count} invitation(s) for <{email_address}> in the database. Server domain is `{server_domain}`."
    );
    Ok(())
}
