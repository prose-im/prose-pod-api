// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use chrono::{TimeDelta, Utc};
use cucumber::{given, then, when};
use entity::{
    model,
    workspace_invitation::{self, InvitationContact},
};
use migration::DbErr;
use prose_pod_api::v1::invitations::{
    AcceptWorkspaceInvitationRequest, InvitationTokenType as TokenType, InviteMemberRequest,
};
use rocket::{
    http::{Accept, ContentType, Header},
    local::asynchronous::{Client, LocalResponse},
};
use serde_json::json;
use service::{
    sea_orm::{prelude::*, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, Set},
    Mutation, MutationError,
};

use crate::{
    cucumber_parameters::{EmailAddress, InvitationStatus, MemberRole, Name, JID},
    TestWorld,
};

const DEFAULT_MEMBER_ROLE: model::MemberRole = model::MemberRole::Member;

async fn invite_member<'a>(
    client: &'a Client,
    token: String,
    jid: JID,
    pre_assigned_role: MemberRole,
    contact: InvitationContact,
) -> LocalResponse<'a> {
    client
        .post("/v1/invitations")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .body(
            json!(InviteMemberRequest {
                jid: jid.0,
                pre_assigned_role: pre_assigned_role.0,
                contact,
            })
            .to_string(),
        )
        .dispatch()
        .await
}

async fn list_workspace_invitations<'a>(client: &'a Client, token: String) -> LocalResponse<'a> {
    client
        .get("/v1/invitations")
        .header(Accept::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .dispatch()
        .await
}

async fn list_workspace_invitations_paged<'a>(
    client: &'a Client,
    token: String,
    page_number: u64,
    page_size: u64,
) -> LocalResponse<'a> {
    client
        .get(format!(
            "/v1/invitations?page_number={page_number}&page_size={page_size}"
        ))
        .header(Accept::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .dispatch()
        .await
}

async fn get_workspace_invitation_by_token<'a>(
    client: &'a Client,
    token: &Uuid,
    token_type: TokenType,
) -> LocalResponse<'a> {
    client
        .get(format!(
            "/v1/invitations?token={token}&token_type={token_type}"
        ))
        .header(Accept::JSON)
        .dispatch()
        .await
}

async fn accept_workspace_invitation<'a>(
    client: &'a Client,
    token: Uuid,
    invitation_id: i32,
    nickname: String,
    password: Option<String>,
) -> LocalResponse<'a> {
    client
        .post(format!(
            "/v1/invitations/{invitation_id}?action=accept&token={token}"
        ))
        .header(ContentType::JSON)
        .body(
            json!(AcceptWorkspaceInvitationRequest {
                nickname: nickname,
                password: password.unwrap_or("test".to_string()),
            })
            .to_string(),
        )
        .header(Accept::JSON)
        .dispatch()
        .await
}

async fn reject_workspace_invitation<'a>(
    client: &'a Client,
    token: Uuid,
    invitation_id: i32,
) -> LocalResponse<'a> {
    client
        .post(format!(
            "/v1/invitations/{invitation_id}?action=reject&token={token}"
        ))
        .header(Accept::JSON)
        .dispatch()
        .await
}

async fn cancel_workspace_invitation<'a>(
    client: &'a Client,
    token: String,
    invitation_id: i32,
) -> LocalResponse<'a> {
    client
        .delete(format!("/v1/invitations/{invitation_id}"))
        .header(Accept::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .dispatch()
        .await
}

async fn workspace_invitation_admin_action<'a>(
    client: &'a Client,
    token: String,
    invitation_id: i32,
    action: &'static str,
) -> LocalResponse<'a> {
    client
        .post(format!("/v1/invitations/{invitation_id}?action={action}"))
        .header(Accept::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .dispatch()
        .await
}

#[given(expr = "<{jid}> has been invited via email")]
async fn given_invited(world: &mut TestWorld, jid: JID) -> Result<(), DbErr> {
    let email_address = model::EmailAddress::from_str(&jid.0.to_string()).unwrap();
    let jid = jid.0;

    // Create invitation
    let db = world.db();
    let model = Mutation::create_workspace_invitation(
        db,
        jid,
        DEFAULT_MEMBER_ROLE,
        InvitationContact::Email {
            email_address: email_address.clone(),
        },
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
    email_address: EmailAddress,
    role: MemberRole,
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
async fn given_n_invited(world: &mut TestWorld, n: u32) -> Result<(), DbErr> {
    for i in 0..n {
        let db = world.db();
        let jid = model::JID::from_str(format!("person.{i}@test.org").as_str()).unwrap();
        let email_address =
            model::EmailAddress::from_str(format!("person.{i}@test.org").as_str()).unwrap();
        let model = Mutation::create_workspace_invitation(
            db,
            jid,
            DEFAULT_MEMBER_ROLE,
            InvitationContact::Email {
                email_address: email_address.clone(),
            },
        )
        .await?;
        world.workspace_invitations.insert(email_address, model);
    }
    Ok(())
}

#[given(expr = "<{email}> has received their invitation")]
async fn given_invitation_received(
    world: &mut TestWorld,
    email_address: EmailAddress,
) -> Result<(), MutationError> {
    let db = world.db();
    Mutation::update_workspace_invitation_status_by_email(
        db,
        email_address.0,
        model::InvitationStatus::Sent,
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
    let model = Mutation::resend_workspace_invitation(db, invitation_before).await?;

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
    Mutation::update_workspace_invitation_status(
        db,
        world.scenario_workspace_invitation().1,
        model::InvitationStatus::SendFailed,
    )
    .await?;
    Ok(())
}

#[when(expr = r#"{name} invites <{jid}> as a(n) {member_role}"#)]
async fn when_inviting(world: &mut TestWorld, name: Name, jid: JID, pre_assigned_role: MemberRole) {
    let token = world.token(name.0);
    let email_address = model::EmailAddress::from_str(&jid.to_string()).unwrap();
    let res = invite_member(
        &world.client,
        token,
        jid,
        pre_assigned_role,
        InvitationContact::Email { email_address },
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "{name} lists pending invitations")]
async fn when_listing_workspace_invitations(world: &mut TestWorld, name: Name) {
    let token = world.token(name.0);
    let res = list_workspace_invitations(&world.client, token).await;
    world.result = Some(res.into());
}

#[when(expr = "{name} lists pending invitations by pages of {int}")]
async fn when_listing_workspace_invitations_paged(
    world: &mut TestWorld,
    name: Name,
    page_size: u64,
) {
    let token = world.token(name.0);
    let res = list_workspace_invitations_paged(&world.client, token, 1, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "{name} gets page {int} of pending invitations by pages of {int}")]
async fn when_getting_workspace_invitations_page(
    world: &mut TestWorld,
    name: Name,
    page_number: u64,
    page_size: u64,
) {
    let token = world.token(name.0);
    let res = list_workspace_invitations_paged(&world.client, token, page_number, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> requests the invitation associated to their accept token")]
async fn when_getting_workspace_invitation_for_accept_token(
    world: &mut TestWorld,
    email_address: EmailAddress,
) {
    let invitation = world.workspace_invitation(&email_address);
    let res = get_workspace_invitation_by_token(
        &world.client,
        &invitation.accept_token,
        TokenType::Accept,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> requests the invitation associated to their reject token")]
async fn when_getting_workspace_invitation_for_reject_token(
    world: &mut TestWorld,
    email_address: EmailAddress,
) {
    let invitation = world.workspace_invitation(&email_address);
    let res = get_workspace_invitation_by_token(
        &world.client,
        &invitation.reject_token,
        TokenType::Reject,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> requests the invitation associated to their previous accept token")]
async fn when_getting_workspace_invitation_for_previous_accept_token(
    world: &mut TestWorld,
    email_address: EmailAddress,
) {
    let res = get_workspace_invitation_by_token(
        &world.client,
        &world.previous_workspace_invitation_accept_token(&email_address),
        TokenType::Accept,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> accepts their invitation")]
async fn when_invited_accepts_invitation(world: &mut TestWorld, email_address: EmailAddress) {
    let invitation = world.workspace_invitation(&email_address.0);
    let res = accept_workspace_invitation(
        &world.client,
        invitation.accept_token,
        invitation.id,
        email_address.0.local_part().to_string(),
        None,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> accepts their invitation using the nickname {string}")]
async fn when_invited_accepts_invitation_with_nickname(
    world: &mut TestWorld,
    email_address: EmailAddress,
    nickname: String,
) {
    let invitation = world.workspace_invitation(&email_address.0);
    let res = accept_workspace_invitation(
        &world.client,
        invitation.accept_token,
        invitation.id,
        nickname,
        None,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> uses the previous invitation accept link they received")]
async fn when_invited_uses_old_accept_link(world: &mut TestWorld, email_address: EmailAddress) {
    let workspace_invitation = world.workspace_invitation(&email_address);
    let res = accept_workspace_invitation(
        &world.client,
        world.previous_workspace_invitation_accept_token(&email_address),
        workspace_invitation.id,
        email_address.0.local_part().to_string(),
        None,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> rejects their invitation")]
async fn when_invited_rejects_invitation(world: &mut TestWorld, email_address: EmailAddress) {
    let invitation = world.workspace_invitation(&email_address.0);
    let res =
        reject_workspace_invitation(&world.client, invitation.reject_token, invitation.id).await;
    world.result = Some(res.into());
}

#[when(expr = "{name} resends the invitation")]
async fn when_user_resends_workspace_invitation(world: &mut TestWorld, name: Name) {
    let token = world.token(name.0);
    let invitation = world.scenario_workspace_invitation().1;
    let res =
        workspace_invitation_admin_action(&world.client, token, invitation.id, "resend").await;
    world.result = Some(res.into());
}

#[when(expr = "{name} cancels the invitation")]
async fn when_user_cancels_workspace_invitation(world: &mut TestWorld, name: Name) {
    let token = world.token(name.0);
    let invitation = world.scenario_workspace_invitation().1;
    let res = cancel_workspace_invitation(&world.client, token, invitation.id).await;
    world.result = Some(res.into());
}

#[then(expr = "they should see {int} pending invitation(s)")]
fn then_n_pending_invitations(world: &mut TestWorld, n: usize) {
    let res: Vec<workspace_invitation::Model> = world.result().body_into();
    assert_eq!(res.len(), n)
}

#[then(expr = "{int} invitation(s) should be {invitation_status}")]
fn then_n_invitations_with_status(
    world: &mut TestWorld,
    n: usize,
    invitation_status: InvitationStatus,
) {
    let res: Vec<workspace_invitation::Model> = world.result().body_into();
    let filtered = res.iter().filter(|m| m.status == invitation_status.0);
    assert_eq!(filtered.count(), n)
}

#[then(expr = "there should be an invitation for <{email}> in the database")]
async fn then_invitation_for_email(
    world: &mut TestWorld,
    email_address: EmailAddress,
) -> Result<(), DbErr> {
    let db = world.db();
    let count = workspace_invitation::Entity::find()
        .filter(workspace_invitation::Column::EmailAddress.eq(email_address.0.clone()))
        .count(db)
        .await?;
    assert_eq!(
        count, 1,
        "Found {count} workspace_invitation(s) for <{email_address}> in the database"
    );
    Ok(())
}

#[then(expr = "there should not be any invitation for <{email}> in the database")]
async fn then_no_invitation_for_email(
    world: &mut TestWorld,
    email_address: EmailAddress,
) -> Result<(), DbErr> {
    let db = world.db();
    let count = workspace_invitation::Entity::find()
        .filter(workspace_invitation::Column::EmailAddress.eq(email_address.0.clone()))
        .count(db)
        .await?;
    assert_eq!(
        count, 0,
        "Found {count} invitation(s) for <{email_address}> in the database"
    );
    Ok(())
}
