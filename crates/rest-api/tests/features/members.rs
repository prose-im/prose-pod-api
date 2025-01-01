// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp::max;

use cucumber::{given, then, when};
use prose_pod_api::error::Error;
use prose_pod_api::features::members::Member as MemberDTO;
use rocket::{
    http::{Accept, Header},
    local::asynchronous::{Client, LocalResponse},
};
use secrecy::{ExposeSecret as _, SecretString};
use service::{
    members::{MemberCreateForm, MemberRepository},
    prose_xmpp::stanza::vcard4::Nickname,
    xmpp::{xmpp_service, BareJid},
};
use urlencoding::encode;

use crate::{cucumber_parameters::JID as JIDParam, TestWorld};
use crate::{
    cucumber_parameters::{Array, Text},
    user_token,
};

use super::name_to_jid;

async fn list_members_<'a>(client: &'a Client, token: Option<SecretString>) -> LocalResponse<'a> {
    let mut req = client.get("/v1/members").header(Accept::JSON);
    if let Some(token) = token {
        req = req.header(Header::new(
            "Authorization",
            format!("Bearer {}", token.expose_secret()),
        ));
    }
    req.dispatch().await
}

async fn list_members<'a>(client: &'a Client, token: SecretString) -> LocalResponse<'a> {
    list_members_(client, Some(token)).await
}

async fn list_members_paged<'a>(
    client: &'a Client,
    token: SecretString,
    page_number: u64,
    page_size: u64,
) -> LocalResponse<'a> {
    client
        .get(format!(
            "/v1/members?page_number={page_number}&page_size={page_size}"
        ))
        .header(Accept::JSON)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", token.expose_secret()),
        ))
        .dispatch()
        .await
}

async fn enrich_members<'a>(
    client: &'a Client,
    token: SecretString,
    jids: Vec<BareJid>,
) -> LocalResponse<'a> {
    client
        .get(format!(
            "/v1/enrich-members?{}",
            jids.iter()
                .map(ToString::to_string)
                .map(|s| encode(&s).to_string())
                .map(|s| format!("jids={s}"))
                .collect::<Vec<_>>()
                .join("&")
        ))
        .header(Accept::EventStream)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", token.expose_secret()),
        ))
        .dispatch()
        .await
}

async fn delete_member<'a>(
    client: &'a Client,
    token: SecretString,
    jid: &BareJid,
) -> LocalResponse<'a> {
    client
        .delete(format!("/v1/members/{jid}"))
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", token.expose_secret()),
        ))
        .dispatch()
        .await
}

#[given(expr = "the workspace has {int} member(s)")]
async fn given_n_members(world: &mut TestWorld, n: u64) -> Result<(), Error> {
    let domain = world.server_config().await?.domain;
    let n = {
        let db = world.db();
        max(0u64, n - MemberRepository::count(db).await?)
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

#[when(expr = "{word} lists members")]
async fn when_listing_members(world: &mut TestWorld, name: String) {
    let token = world.token(name);
    let res = list_members(world.client(), token).await;
    world.result = Some(res.into());
}

#[when("someone lists members without authenticating")]
async fn when_listing_members_unauthenticated(world: &mut TestWorld) {
    let res = list_members_(world.client(), None).await;
    world.result = Some(res.into());
}

#[when(expr = "someone lists members using {string} as Bearer token")]
async fn when_listing_members_custom_token(world: &mut TestWorld, token: String) {
    let res = list_members(world.client(), token.into()).await;
    world.result = Some(res.into());
}

#[when(expr = "{word} lists members by pages of {int}")]
async fn when_listing_members_paged(world: &mut TestWorld, name: String, page_size: u64) {
    let token = world.token(name);
    let res = list_members_paged(world.client(), token, 1, page_size).await;
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
    let res = list_members_paged(world.client(), token, page_number, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "{word} gets detailed information about {array}")]
async fn when_getting_members_details(world: &mut TestWorld, name: String, names: Array<Text>) {
    let token = world.token(name);
    let mut jids = Vec::with_capacity(names.len());
    for name in names.iter() {
        jids.push(name_to_jid(world, name).await.unwrap());
    }
    let res = enrich_members(world.client(), token, jids).await;
    world.result = Some(res.into());
}

#[when(expr = "{} deletes {jid}’s account")]
async fn when_delete_member(world: &mut TestWorld, name: String, jid: JIDParam) {
    let token = user_token!(world, name);
    let res = delete_member(world.client(), token, &jid).await;
    world.result = Some(res.into());
}

#[then(expr = "they should see {int} member(s)")]
fn then_n_members(world: &mut TestWorld, n: usize) {
    let res: Vec<MemberDTO> = world.result().body_into();
    assert_eq!(res.len(), n)
}

#[then(expr = "<{jid}> should have the nickname {string}")]
async fn then_nickname(
    world: &mut TestWorld,
    jid: JIDParam,
    nickname: String,
) -> Result<(), xmpp_service::Error> {
    let vcard = world
        .mock_xmpp_service
        .get_vcard(&jid)?
        .expect("vCard not found");

    assert_eq!(vcard.nickname, vec![Nickname { value: nickname }]);

    Ok(())
}
