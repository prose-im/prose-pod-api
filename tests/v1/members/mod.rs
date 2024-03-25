// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use cucumber::{given, then, when};
use entity::{
    member_invite,
    model::{
        member_invite::{MemberInviteContact, MemberInviteState as MemberInviteStateModel},
        EmailAddress as EmailAddressEntityModel, MemberRole,
    },
};
use migration::DbErr;
use prose_pod_api::v1::members::{InviteAction, InviteMemberRequest};
use rocket::{
    http::{Accept, ContentType, Header},
    local::asynchronous::{Client, LocalResponse},
};
use serde_json::json;
use service::{
    sea_orm::{prelude::*, EntityTrait, PaginatorTrait, QueryFilter},
    Mutation, MutationError,
};

use crate::{
    cucumber_parameters::{EmailAddress, MemberInviteState, Name},
    TestWorld,
};

const DEFAULT_MEMBER_ROLE: MemberRole = MemberRole::Member;

async fn invite_member<'a>(
    client: &'a Client,
    token: String,
    pre_assigned_role: MemberRole,
    contact: MemberInviteContact,
) -> LocalResponse<'a> {
    client
        .post("/v1/members/invites")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .body(
            json!(InviteMemberRequest {
                pre_assigned_role,
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

async fn invite_action<'a>(
    client: &'a Client,
    invite_id: i32,
    action: InviteAction,
) -> LocalResponse<'a> {
    client
        .post(format!("/v1/members/invites/{invite_id}?action={action}"))
        .header(Accept::JSON)
        .dispatch()
        .await
}

async fn invite_resend<'a>(client: &'a Client, token: String, invite_id: i32) -> LocalResponse<'a> {
    client
        .post(format!("/v1/members/invites/{invite_id}?action=resend"))
        .header(Accept::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .dispatch()
        .await
}

#[given(expr = "<{email}> has been invited via email")]
async fn given_invited(world: &mut TestWorld, email_address: EmailAddress) -> Result<(), DbErr> {
    let db = world.db();
    let model = Mutation::create_member_invite(
        db,
        DEFAULT_MEMBER_ROLE,
        MemberInviteContact::Email {
            email_address: email_address.0.clone(),
        },
    )
    .await?;
    world
        .member_invites
        .insert(email_address.0.clone(), model.clone());
    world.scenario_invite = Some((email_address.0, model));
    Ok(())
}

#[given(expr = "{int} people have been invited via email")]
async fn given_n_invited(world: &mut TestWorld, n: u32) -> Result<(), DbErr> {
    for i in 0..n {
        let db = world.db();
        let email_address =
            EmailAddressEntityModel::from_str(format!("person.{i}@test.org").as_str()).unwrap();
        let model = Mutation::create_member_invite(
            db,
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
        MemberInviteStateModel::Received,
    )
    .await?;
    Ok(())
}

#[given("the invitation did not go through")]
async fn given_invite_not_received(world: &mut TestWorld) -> Result<(), MutationError> {
    let db = world.db();
    Mutation::update_member_invite_status(
        db,
        world.scenario_invite().1,
        MemberInviteStateModel::ReceptionFailure,
    )
    .await?;
    Ok(())
}

#[when(regex = r#"^(.+) invites <(.+)> as an? (.+)$"#)]
async fn when_inviting(
    world: &mut TestWorld,
    name: String,
    email_address: EmailAddress,
    pre_assigned_role: MemberRole,
) {
    let token = world.token(name);
    let res = invite_member(
        &world.client,
        token,
        pre_assigned_role,
        MemberInviteContact::Email {
            email_address: email_address.0,
        },
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

#[when(expr = "<{email}> accepts their invitation")]
async fn when_invited_accpts_invite(world: &mut TestWorld, email_address: EmailAddress) {
    let res = invite_action(
        &world.client,
        world.invite(email_address.0).id,
        InviteAction::Accept,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "<{email}> rejects their invitation")]
async fn when_invited_rejects_invite(world: &mut TestWorld, email_address: EmailAddress) {
    let res = invite_action(
        &world.client,
        world.invite(email_address.0).id,
        InviteAction::Reject,
    )
    .await;
    world.result = Some(res.into());
}

#[when(expr = "{name} resends the invitation")]
async fn when_user_resends_invite(world: &mut TestWorld, name: Name) {
    let token = world.token(name.0);
    let invite = world.scenario_invite().1;
    let res = invite_resend(&world.client, token, invite.id).await;
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
