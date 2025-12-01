// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    invitations::invitation_service::InviteUserCommand,
    members::*,
    models::{Avatar, EmailAddress},
};
use urlencoding::encode;

use crate::prelude::mocks::UserAccount;

use super::{invitations::accept_workspace_invitation, prelude::*};

// MARK: - Given

#[given(expr = "the workspace has {int} member(s)")]
#[given(expr = "the Workspace has {int} member(s)")]
async fn given_n_members(world: &mut TestWorld, n: usize) -> Result<(), anyhow::Error> {
    let domain = world.server_config().await?.domain;
    let user_count = world.mock_user_repository().user_count();

    let n = std::cmp::max(0, n - user_count);
    for i in 0..n {
        // Create user account `n`.
        let jid = BareJid::new(&format!("person.{i}@{domain}")).unwrap();
        let nickname = Nickname::from_string_unsafe(format!("Person {i}"));
        world
            .mock_user_repository()
            .add_user(
                &nickname,
                UserAccount::member(jid),
                world.mock_auth_service(),
            )
            .await?;
    }

    Ok(())
}

#[given(expr = "{}'s avatar is {}")]
async fn given_avatar(world: &mut TestWorld, name: String, base64: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;

    world
        .mock_xmpp_service()
        .set_avatar(&jid, Some(Avatar::try_from_base64(base64).unwrap()))?;

    Ok(())
}

#[given(expr = "{} has no avatar")]
async fn given_no_avatar(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;

    world.mock_xmpp_service().set_avatar(&jid, None)?;

    Ok(())
}

// MARK: - When

async fn list_members_(api: &TestServer, auth: Option<&AuthToken>) -> TestResponse {
    let mut req = api
        .get("/v1/members")
        .add_header(ACCEPT, "application/json");
    if let Some(auth) = auth {
        req = req.add_header(AUTHORIZATION, format!("Bearer {}", auth.expose_secret()));
    }
    req.await
}

async fn list_members(api: &TestServer, auth: &AuthToken) -> TestResponse {
    list_members_(api, Some(auth)).await
}

async fn list_members_paged(
    api: &TestServer,
    auth: &AuthToken,
    page_number: u64,
    page_size: u64,
) -> TestResponse {
    api.get(&format!(
        "/v1/members?page_number={page_number}&page_size={page_size}"
    ))
    .add_header(AUTHORIZATION, format!("Bearer {}", auth.expose_secret()))
    .add_header(ACCEPT, "application/json")
    .await
}

async fn enrich_members(api: &TestServer, auth: &AuthToken, jids: Vec<BareJid>) -> TestResponse {
    api.get(&format!(
        "/v1/enrich-members?{}",
        jids.iter()
            .map(|s| format!("jids={}", encode(s.to_string().as_str())))
            .collect::<Vec<_>>()
            .join("&")
    ))
    .add_header(AUTHORIZATION, format!("Bearer {}", auth.expose_secret()))
    .add_header(ACCEPT, "text/event-stream")
    .await
}

api_call_fn!(delete_member, DELETE, "/v1/members/{jid}"; jid=&BareJid);

#[when("a new member joins the Workspace")]
async fn when_new_member_joins(world: &mut TestWorld) -> Result<(), Error> {
    let domain = world.server_config().await?.domain;
    let user_count = world.mock_user_repository().user_count();
    let jid = BareJid::new(&format!("member.{user_count}@{domain}")).unwrap();
    let username = jid.expect_username();
    let email_address = EmailAddress::from(&jid);

    // Create an invitation.
    let invitation = world
        .invitation_service()
        .invite_user(
            InviteUserCommand {
                username: username.into(),
                role: MemberRole::Member,
                email_address,
                ttl: None,
            },
            &BYPASS_TOKEN,
        )
        .await?;

    // Accept the invitation.
    let res = accept_workspace_invitation(
        world.api(),
        invitation.accept_token,
        Nickname::from_string_unsafe(username.to_string()),
        Some("password".into()),
    )
    .await;
    world.result = Some(res.into());

    Ok(())
}

#[when(expr = "{word} lists members")]
async fn when_listing_members(world: &mut TestWorld, name: String) {
    let ref auth = world.token(&name).await;
    let res = list_members(world.api(), auth).await;
    world.result = Some(res.into());
}

#[when("someone lists members without authenticating")]
async fn when_listing_members_unauthenticated(world: &mut TestWorld) {
    let res = list_members_(world.api(), None).await;
    world.result = Some(res.into());
}

#[when(expr = "someone lists members using {string} as Bearer token")]
async fn when_listing_members_custom_token(world: &mut TestWorld, auth: String) {
    let res = list_members(world.api(), &auth.into()).await;
    world.result = Some(res.into());
}

#[when(expr = "{word} lists members by pages of {int}")]
async fn when_listing_members_paged(world: &mut TestWorld, name: String, page_size: u64) {
    let ref auth = world.token(&name).await;
    let res = list_members_paged(world.api(), auth, 1, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "{word} gets page {int} of members by pages of {int}")]
async fn when_getting_members_page(
    world: &mut TestWorld,
    name: String,
    page_number: u64,
    page_size: u64,
) {
    let ref auth = world.token(&name).await;
    let res = list_members_paged(world.api(), auth, page_number, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "{word} gets detailed information about {array}")]
async fn when_getting_members_details(
    world: &mut TestWorld,
    name: String,
    names: parameters::Array<parameters::Text>,
) {
    let ref auth = world.token(&name).await;
    let mut jids = Vec::with_capacity(names.len());
    for name in names.iter() {
        jids.push(name_to_jid(world, name).await.unwrap());
    }
    let res = enrich_members(world.api(), auth, jids).await;
    world.result = Some(res.into());
}

#[when(expr = "{} deletes {}’s account")]
async fn when_delete_member(
    world: &mut TestWorld,
    actor: String,
    target: String,
) -> Result<(), anyhow::Error> {
    let ref auth = world.token(&actor).await;
    let jid = name_to_jid(world, &target).await?;

    let res = delete_member(world.api(), auth, &jid).await?;
    world.result = Some(res.into());

    Ok(())
}

#[when(expr = "{} deletes a member")]
async fn when_member_deleted(world: &mut TestWorld, actor: String) -> Result<(), anyhow::Error> {
    let user_count = world.mock_user_repository().user_count();

    when_member_n_deleted(world, actor, user_count - 1).await
}

#[when(expr = "{} deletes member {int}")]
async fn when_member_n_deleted(
    world: &mut TestWorld,
    actor: String,
    n: usize,
) -> Result<(), anyhow::Error> {
    let ref auth = world.token(&actor).await;
    let members = world.user_repository().list_users(&BYPASS_TOKEN).await?;
    assert!(n < members.len());
    let ref jid = members[n].jid;

    let res = delete_member(world.api(), auth, jid).await?;
    world.result = Some(res.into());

    Ok(())
}

// MARK: - Then

#[then(expr = "they should see {int} member(s)")]
fn then_n_members(world: &mut TestWorld, n: usize) {
    let res: Vec<Member> = world.result().json();

    assert_eq!(res.len(), n)
}

#[then(expr = "there should be {int} account(s) on the Server")]
async fn then_n_members_in_db(world: &mut TestWorld, n: usize) -> Result<(), Error> {
    let user_count = world.mock_user_repository().user_count();

    assert_eq!(user_count, n);

    Ok(())
}
