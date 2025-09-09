// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    invitations::{InvitationContact, InviteMemberForm},
    members::{entities::member, *},
    models::EmailAddress,
    sea_orm::QueryOrder as _,
    util::detect_image_mime_type,
    xmpp::xmpp_service::Avatar,
};
use urlencoding::encode;

use super::{invitations::accept_workspace_invitation, prelude::*};

async fn list_members_(api: &TestServer, token: Option<SecretString>) -> TestResponse {
    let mut req = api
        .get("/v1/members")
        .add_header(ACCEPT, "application/json");
    if let Some(token) = token {
        req = req.add_header(AUTHORIZATION, format!("Bearer {}", token.expose_secret()));
    }
    req.await
}

async fn list_members(api: &TestServer, token: SecretString) -> TestResponse {
    list_members_(api, Some(token)).await
}

async fn list_members_paged(
    api: &TestServer,
    token: SecretString,
    page_number: u64,
    page_size: u64,
) -> TestResponse {
    api.get(&format!(
        "/v1/members?page_number={page_number}&page_size={page_size}"
    ))
    .add_header(AUTHORIZATION, format!("Bearer {}", token.expose_secret()))
    .add_header(ACCEPT, "application/json")
    .await
}

async fn enrich_members(api: &TestServer, token: SecretString, jids: Vec<BareJid>) -> TestResponse {
    api.get(&format!(
        "/v1/enrich-members?{}",
        jids.iter()
            .map(|s| format!("jids={}", encode(s.to_string().as_str())))
            .collect::<Vec<_>>()
            .join("&")
    ))
    .add_header(AUTHORIZATION, format!("Bearer {}", token.expose_secret()))
    .add_header(ACCEPT, "text/event-stream")
    .await
}

async fn delete_member(api: &TestServer, token: SecretString, jid: &BareJid) -> TestResponse {
    api.delete(&format!("/v1/members/{jid}"))
        .add_header(AUTHORIZATION, format!("Bearer {}", token.expose_secret()))
        .await
}

#[given(expr = "the workspace has {int} member(s)")]
#[given(expr = "the Workspace has {int} member(s)")]
async fn given_n_members(world: &mut TestWorld, n: u64) -> Result<(), Error> {
    let domain = world.server_config().await?.domain;
    let n = {
        let db = world.db();
        std::cmp::max(0u64, n - MemberRepository::count(db).await?)
    };
    for i in 0..n {
        let db = world.db();
        let jid = BareJid::new(&format!("person.{i}@{domain}")).unwrap();
        let member = MemberCreateForm {
            jid: jid.clone(),
            role: None,
            joined_at: None,
            email_address: Some(EmailAddress::from_str(jid.as_str()).unwrap()),
        };
        let model = MemberRepository::create(db, member).await?;
        let token = world.mock_auth_service.log_in_unchecked(&jid).await?;

        world.members.insert(jid.to_string(), (model.into(), token));
    }
    Ok(())
}

#[given(expr = "{}'s avatar is {}")]
async fn given_avatar(world: &mut TestWorld, name: String, base64: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    world.mock_xmpp_service.set_avatar(
        &jid,
        Some(Avatar {
            mime: detect_image_mime_type(&base64).expect("Invalid base64"),
            base64,
        }),
    )?;
    Ok(())
}

#[given(expr = "{} has no avatar")]
async fn given_no_avatar(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    world.mock_xmpp_service.set_avatar(&jid, None)?;
    Ok(())
}

#[when("a new member joins the Workspace")]
async fn when_new_member_joins(world: &mut TestWorld) -> Result<(), Error> {
    let db = world.db();

    let domain = world.server_config().await?.domain;
    let username = format!("member.{}", MemberRepository::count(db).await?);
    let jid = BareJid::new(&format!("{username}@{domain}")).unwrap();
    let email_address = EmailAddress::from_str(jid.as_str()).unwrap();

    let invitation = world
        .invitation_service()
        .invite_member(
            &world.app_config(),
            &world.server_config().await?.domain,
            &world.notifcation_service(),
            &world.workspace_service().await,
            InviteMemberForm {
                username: JidNode::from_str(&username).unwrap(),
                pre_assigned_role: MemberRole::Member,
                contact: InvitationContact::Email { email_address },
            },
            false,
        )
        .await?;

    let res = accept_workspace_invitation(
        world.api(),
        invitation.accept_token,
        username,
        Some("password".into()),
    )
    .await;
    world.result = Some(res.into());

    Ok(())
}

#[when(expr = "{word} lists members")]
async fn when_listing_members(world: &mut TestWorld, name: String) {
    let token = world.token(&name);
    let res = list_members(world.api(), token).await;
    world.result = Some(res.into());
}

#[when("someone lists members without authenticating")]
async fn when_listing_members_unauthenticated(world: &mut TestWorld) {
    let res = list_members_(world.api(), None).await;
    world.result = Some(res.into());
}

#[when(expr = "someone lists members using {string} as Bearer token")]
async fn when_listing_members_custom_token(world: &mut TestWorld, token: String) {
    let res = list_members(world.api(), token.into()).await;
    world.result = Some(res.into());
}

#[when(expr = "{word} lists members by pages of {int}")]
async fn when_listing_members_paged(world: &mut TestWorld, name: String, page_size: u64) {
    let token = world.token(&name);
    let res = list_members_paged(world.api(), token, 1, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "{word} gets page {int} of members by pages of {int}")]
async fn when_getting_members_page(
    world: &mut TestWorld,
    name: String,
    page_number: u64,
    page_size: u64,
) {
    let token = world.token(&name);
    let res = list_members_paged(world.api(), token, page_number, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "{word} gets detailed information about {array}")]
async fn when_getting_members_details(
    world: &mut TestWorld,
    name: String,
    names: parameters::Array<parameters::Text>,
) {
    let token = world.token(&name);
    let mut jids = Vec::with_capacity(names.len());
    for name in names.iter() {
        jids.push(name_to_jid(world, name).await.unwrap());
    }
    let res = enrich_members(world.api(), token, jids).await;
    world.result = Some(res.into());
}

#[when(expr = "{} deletes {}’s account")]
async fn when_delete_member(
    world: &mut TestWorld,
    actor: String,
    target: String,
) -> Result<(), Error> {
    let token = user_token!(world, actor);
    let jid = name_to_jid(world, &target).await?;
    let res = delete_member(world.api(), token, &jid).await;
    world.result = Some(res.into());
    Ok(())
}

#[when(expr = "{} deletes a member")]
async fn when_member_deleted(world: &mut TestWorld, actor: String) -> Result<(), Error> {
    let count = MemberRepository::count(world.db()).await? as usize;
    when_member_n_deleted(world, actor, count - 1).await
}

#[when(expr = "{} deletes member {int}")]
async fn when_member_n_deleted(
    world: &mut TestWorld,
    actor: String,
    n: usize,
) -> Result<(), Error> {
    let token = user_token!(world, actor);
    let members = member::Entity::find()
        .order_by_asc(member::Column::JoinedAt)
        .all(world.db())
        .await?;
    let jid = members[n].jid();
    let res = delete_member(world.api(), token, &jid).await;
    world.result = Some(res.into());
    Ok(())
}

#[then(expr = "they should see {int} member(s)")]
fn then_n_members(world: &mut TestWorld, n: usize) {
    let res: Vec<Member> = world.result().json();
    assert_eq!(res.len(), n)
}

#[then(expr = "there should be {int} member(s) in the database")]
async fn then_n_members_in_db(world: &mut TestWorld, n: u64) -> Result<(), Error> {
    let count = MemberRepository::count(world.db()).await?;
    assert_eq!(count, n);
    Ok(())
}
