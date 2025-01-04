// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_pod_api::features::members::Member as MemberDTO;
use service::{members::*, prose_xmpp::stanza::vcard4::Nickname, xmpp::xmpp_service};

use super::prelude::*;

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
        serde_qs::to_string(&json!({ "jids": jids })).unwrap()
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
        };
        let model = MemberRepository::create(db, member).await?;
        let token = world.mock_auth_service.log_in_unchecked(&jid)?;

        world.members.insert(jid.to_string(), (model, token));
    }
    Ok(())
}

#[given(expr = "{}'s avatar is {}")]
async fn given_avatar(world: &mut TestWorld, name: String, avatar: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    world
        .mock_xmpp_service
        .set_avatar(&jid, Some(AvatarData::Base64(avatar)))?;
    Ok(())
}

#[given(expr = "{} has no avatar")]
async fn given_no_avatar(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    world.mock_xmpp_service.set_avatar(&jid, None)?;
    Ok(())
}

#[when(expr = "{word} lists members")]
async fn when_listing_members(world: &mut TestWorld, name: String) {
    let token = world.token(name);
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
    let token = world.token(name);
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
    let token = world.token(name);
    let res = list_members_paged(world.api(), token, page_number, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "{word} gets detailed information about {array}")]
async fn when_getting_members_details(
    world: &mut TestWorld,
    name: String,
    names: parameters::Array<parameters::Text>,
) {
    let token = world.token(name);
    let mut jids = Vec::with_capacity(names.len());
    for name in names.iter() {
        jids.push(name_to_jid(world, name).await.unwrap());
    }
    let res = enrich_members(world.api(), token, jids).await;
    world.result = Some(res.into());
}

#[when(expr = "{} deletes {jid}’s account")]
async fn when_delete_member(world: &mut TestWorld, name: String, jid: parameters::JID) {
    let token = user_token!(world, name);
    let res = delete_member(world.api(), token, &jid).await;
    world.result = Some(res.into());
}

#[then(expr = "they should see {int} member(s)")]
fn then_n_members(world: &mut TestWorld, n: usize) {
    let res: Vec<MemberDTO> = world.result().json();
    assert_eq!(res.len(), n)
}

#[then(expr = "<{jid}> should have the nickname {string}")]
async fn then_nickname(
    world: &mut TestWorld,
    jid: parameters::JID,
    nickname: String,
) -> Result<(), xmpp_service::Error> {
    let vcard = world
        .mock_xmpp_service
        .get_vcard(&jid)?
        .expect("vCard not found");

    assert_eq!(vcard.nickname, vec![Nickname { value: nickname }]);

    Ok(())
}
