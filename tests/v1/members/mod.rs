// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use chrono::{TimeDelta, Utc};
use cucumber::{given, then, when};
use entity::{
    member, member_invite,
    model::{self, MemberInviteContact},
};
use migration::DbErr;
use prose_pod_api::{
    forms::MemberInviteTokenType::{self as TokenType},
    v1::members::{AcceptInviteRequest, InviteMemberRequest},
};
use rocket::{
    http::{Accept, ContentType, Header},
    local::asynchronous::{Client, LocalResponse},
};
use serde_json::json;
use service::{
    sea_orm::{prelude::*, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, Set},
    server_ctl::{self, ServerCtlImpl},
    vcard_parser::{
        constants::PropertyName,
        traits::HasValue as _,
        vcard::{self, property::property_nickname::PropertyNickNameData},
    },
    Mutation, MutationError,
};

use crate::{
    cucumber_parameters::{EmailAddress, MemberInviteState, MemberRole, Name, JID},
    TestWorld,
};

const DEFAULT_MEMBER_ROLE: model::MemberRole = model::MemberRole::Member;

async fn invite_member<'a>(
    client: &'a Client,
    token: String,
    jid: JID,
    pre_assigned_role: MemberRole,
    contact: MemberInviteContact,
) -> LocalResponse<'a> {
    client
        .post("/v1/members/invites")
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

async fn list_invites<'a>(client: &'a Client, token: String) -> LocalResponse<'a> {
    client
        .get("/v1/members/invites")
        .header(Accept::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .dispatch()
        .await
}

async fn list_invites_paged<'a>(
    client: &'a Client,
    token: String,
    page_number: u64,
    page_size: u64,
) -> LocalResponse<'a> {
    client
        .get(format!(
            "/v1/members/invites?page_number={page_number}&page_size={page_size}"
        ))
        .header(Accept::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .dispatch()
        .await
}

async fn get_invite_by_token<'a>(
    client: &'a Client,
    token: &Uuid,
    token_type: TokenType,
) -> LocalResponse<'a> {
    client
        .get(format!(
            "/v1/members/invites?token={token}&token_type={token_type}"
        ))
        .header(Accept::JSON)
        .dispatch()
        .await
}

async fn accept_invite<'a>(
    client: &'a Client,
    token: Uuid,
    invite_id: i32,
    nickname: String,
    password: Option<String>,
) -> LocalResponse<'a> {
    client
        .post(format!(
            "/v1/members/invites/{invite_id}?action=accept&token={token}"
        ))
        .header(ContentType::JSON)
        .body(
            json!(AcceptInviteRequest {
                nickname: nickname,
                password: password.unwrap_or("test".to_string()),
            })
            .to_string(),
        )
        .header(Accept::JSON)
        .dispatch()
        .await
}

async fn reject_invite<'a>(client: &'a Client, token: Uuid, invite_id: i32) -> LocalResponse<'a> {
    client
        .post(format!(
            "/v1/members/invites/{invite_id}?action=reject&token={token}"
        ))
        .header(Accept::JSON)
        .dispatch()
        .await
}

async fn invite_admin_action<'a>(
    client: &'a Client,
    token: String,
    invite_id: i32,
    action: &'static str,
) -> LocalResponse<'a> {
    client
        .post(format!("/v1/members/invites/{invite_id}?action={action}"))
        .header(Accept::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .dispatch()
        .await
}

#[given(expr = "<{jid}> has been invited via email")]
async fn given_invited(world: &mut TestWorld, jid: JID) -> Result<(), DbErr> {
    let email_address = model::EmailAddress::from_str(&jid.0.to_string()).unwrap();
    let jid = jid.0;

    // Create invite
    let db = world.db();
    let model = Mutation::create_member_invite(
        db,
        jid,
        DEFAULT_MEMBER_ROLE,
        MemberInviteContact::Email {
            email_address: email_address.clone(),
        },
    )
    .await?;

    // Store current invite data
    world
        .member_invites
        .insert(email_address.clone(), model.clone());
    world.scenario_invite = Some((email_address, model));

    Ok(())
}

#[given(expr = "<{email}> is pre-assigned the {member_role} role")]
async fn given_pre_assigned_role(
    world: &mut TestWorld,
    email_address: EmailAddress,
    role: MemberRole,
) -> Result<(), DbErr> {
    let db = world.db();

    let mut active = world.invite(&email_address).into_active_model();
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
        let model = Mutation::create_member_invite(
            db,
            jid,
            DEFAULT_MEMBER_ROLE,
            MemberInviteContact::Email {
                email_address: email_address.clone(),
            },
        )
        .await?;
        world.member_invites.insert(email_address, model);
    }
    Ok(())
}

#[given(expr = "<{email}> has received their invitation")]
async fn given_invite_received(
    world: &mut TestWorld,
    email_address: EmailAddress,
) -> Result<(), MutationError> {
    let db = world.db();
    Mutation::update_member_invite_status_by_email(
        db,
        email_address.0,
        model::MemberInviteState::Received,
    )
    .await?;
    Ok(())
}

#[given("an admin resent the invite")]
async fn given_invite_resent(world: &mut TestWorld) -> Result<(), MutationError> {
    let (email_address, invite_before) = world.scenario_invite();

    // Store previous accept token for other steps requiring it
    world
        .previous_invite_accept_token
        .insert(email_address.clone(), invite_before.accept_token);

    // Resend invite
    let db = world.db();
    let model = Mutation::resend_invite(db, invite_before).await?;

    // Store current invite data
    world
        .member_invites
        .insert(email_address.clone(), model.clone());
    world.scenario_invite = Some((email_address, model));

    Ok(())
}

#[given("the invite has already expired")]
async fn given_invite_expired(world: &mut TestWorld) -> Result<(), MutationError> {
    let db = world.db();
    let (email_address, invite_before) = world.scenario_invite();

    // Update invite
    let mut active = invite_before.into_active_model();
    active.accept_token_expires_at =
        Set(Utc::now().checked_sub_signed(TimeDelta::days(1)).unwrap());
    let model = active.update(db).await?;

    // Store current invite data
    world
        .member_invites
        .insert(email_address.clone(), model.clone());
    world.scenario_invite = Some((email_address, model));

    Ok(())
}

#[given("the invitation did not go through")]
async fn given_invite_not_received(world: &mut TestWorld) -> Result<(), MutationError> {
    let db = world.db();
    Mutation::update_member_invite_status(
        db,
        world.scenario_invite().1,
        model::MemberInviteState::ReceptionFailure,
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
        MemberInviteContact::Email { email_address },
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "{name} lists pending invitations")]
async fn when_listing_invites(world: &mut TestWorld, name: Name) {
    let token = world.token(name.0);
    let res = list_invites(&world.client, token).await;
    world.result = Some(res.into());
}

#[when(expr = "{name} lists pending invitations by pages of {int}")]
async fn when_listing_invites_paged(world: &mut TestWorld, name: Name, page_size: u64) {
    let token = world.token(name.0);
    let res = list_invites_paged(&world.client, token, 1, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "{name} gets page {int} of pending invitations by pages of {int}")]
async fn when_getting_invites_page(
    world: &mut TestWorld,
    name: Name,
    page_number: u64,
    page_size: u64,
) {
    let token = world.token(name.0);
    let res = list_invites_paged(&world.client, token, page_number, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> requests the invite associated to their accept token")]
async fn when_getting_invite_for_accept_token(world: &mut TestWorld, email_address: EmailAddress) {
    let invite = world.invite(&email_address);
    let res = get_invite_by_token(&world.client, &invite.accept_token, TokenType::Accept).await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> requests the invite associated to their reject token")]
async fn when_getting_invite_for_reject_token(world: &mut TestWorld, email_address: EmailAddress) {
    let invite = world.invite(&email_address);
    let res = get_invite_by_token(&world.client, &invite.reject_token, TokenType::Reject).await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> requests the invite associated to their previous accept token")]
async fn when_getting_invite_for_previous_accept_token(
    world: &mut TestWorld,
    email_address: EmailAddress,
) {
    let res = get_invite_by_token(
        &world.client,
        &world.previous_invite_accept_token(&email_address),
        TokenType::Accept,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> accepts their invitation")]
async fn when_invited_accepts_invite(world: &mut TestWorld, email_address: EmailAddress) {
    let invite = world.invite(&email_address.0);
    let res = accept_invite(
        &world.client,
        invite.accept_token,
        invite.id,
        email_address.0.local_part().to_string(),
        None,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> accepts their invitation using the nickname {string}")]
async fn when_invited_accepts_invite_with_nickname(
    world: &mut TestWorld,
    email_address: EmailAddress,
    nickname: String,
) {
    let invite = world.invite(&email_address.0);
    let res = accept_invite(
        &world.client,
        invite.accept_token,
        invite.id,
        nickname,
        None,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> uses the previous invite accept link they received")]
async fn when_invited_uses_old_accept_link(world: &mut TestWorld, email_address: EmailAddress) {
    let invite = world.invite(&email_address);
    let res = accept_invite(
        &world.client,
        world.previous_invite_accept_token(&email_address),
        invite.id,
        email_address.0.local_part().to_string(),
        None,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> rejects their invitation")]
async fn when_invited_rejects_invite(world: &mut TestWorld, email_address: EmailAddress) {
    let invite = world.invite(&email_address.0);
    let res = reject_invite(&world.client, invite.reject_token, invite.id).await;
    world.result = Some(res.into());
}

#[when(expr = "{name} resends the invitation")]
async fn when_user_resends_invite(world: &mut TestWorld, name: Name) {
    let token = world.token(name.0);
    let invite = world.scenario_invite().1;
    let res = invite_admin_action(&world.client, token, invite.id, "resend").await;
    world.result = Some(res.into());
}

#[when(expr = "{name} cancels the invitation")]
async fn when_user_cancels_invite(world: &mut TestWorld, name: Name) {
    let token = world.token(name.0);
    let invite = world.scenario_invite().1;
    let res = invite_admin_action(&world.client, token, invite.id, "cancel").await;
    world.result = Some(res.into());
}

#[then(expr = "they should see {int} pending invitation(s)")]
fn then_n_pending_invitations(world: &mut TestWorld, n: usize) {
    let res: Vec<member_invite::Model> = world.result().body_into();
    assert_eq!(res.len(), n)
}

#[then(expr = "{int} invitation(s) should be {invitation_status}")]
fn then_n_invitations_with_status(
    world: &mut TestWorld,
    n: usize,
    invitation_status: MemberInviteState,
) {
    let res: Vec<member_invite::Model> = world.result().body_into();
    let filtered = res.iter().filter(|m| m.state == invitation_status.0);
    assert_eq!(filtered.count(), n)
}

#[then(expr = "there should be an invitation for <{email}> in the database")]
async fn then_invitation_for_email(
    world: &mut TestWorld,
    email_address: EmailAddress,
) -> Result<(), DbErr> {
    let db = world.db();
    let count = member_invite::Entity::find()
        .filter(member_invite::Column::EmailAddress.eq(email_address.0.clone()))
        .count(db)
        .await?;
    assert_eq!(
        count, 1,
        "Found {count} invite(s) for <{email_address}> in the database"
    );
    Ok(())
}

#[then(expr = "there should not be any invitation for <{email}> in the database")]
async fn then_no_invitation_for_email(
    world: &mut TestWorld,
    email_address: EmailAddress,
) -> Result<(), DbErr> {
    let db = world.db();
    let count = member_invite::Entity::find()
        .filter(member_invite::Column::EmailAddress.eq(email_address.0.clone()))
        .count(db)
        .await?;
    assert_eq!(
        count, 0,
        "Found {count} invite(s) for <{email_address}> in the database"
    );
    Ok(())
}

#[then(expr = "<{jid}> should have the {member_role} role")]
async fn then_role(world: &mut TestWorld, jid: JID, role: MemberRole) -> Result<(), DbErr> {
    let db = world.db();

    let member = member::Entity::find_by_id(jid.to_string())
        .one(db)
        .await?
        .expect(&format!("Member {jid} not found"));
    assert_eq!(member.role, role.0);

    Ok(())
}

#[then(expr = "<{jid}> should have the nickname {string}")]
async fn then_nickname(
    world: &mut TestWorld,
    jid: JID,
    nickname: String,
) -> Result<(), server_ctl::Error> {
    let vcard = world
        .server_ctl()
        .get_vcard(&jid)?
        .expect("vCard not found");
    let properties = vcard.get_properties_by_name(PropertyName::NICKNAME);
    let properties = properties
        .iter()
        .map(vcard::property::Property::get_value)
        .collect::<Vec<_>>();

    let expected = PropertyNickNameData::try_from((None, nickname.as_str(), vec![])).unwrap();
    assert_eq!(properties, vec![expected.get_value()]);

    Ok(())
}
