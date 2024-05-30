// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp::max;

use cucumber::{given, then, when};
use entity::{model::JID, prelude::Member};
use migration::DbErr;
use prose_pod_api::v1::members::Member as MemberDTO;
use prose_pod_api::{error::Error, guards::JWTService};
use rocket::{
    http::{Accept, Header},
    local::asynchronous::{Client, LocalResponse},
};
use service::xmpp::stanza::vcard::Nickname;
use service::{xmpp_service, Query, XmppServiceContext};
use service::{xmpp_service::XmppServiceImpl as _, Mutation};

use crate::cucumber_parameters::Array;
use crate::v1::test_jid;
use crate::{
    cucumber_parameters::{MemberRole, JID as JIDParam},
    TestWorld,
};

async fn list_members<'a>(client: &'a Client, token: String) -> LocalResponse<'a> {
    client
        .get("/v1/members")
        .header(Accept::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .dispatch()
        .await
}

async fn list_members_paged<'a>(
    client: &'a Client,
    token: String,
    page_number: u64,
    page_size: u64,
) -> LocalResponse<'a> {
    client
        .get(format!(
            "/v1/members?page_number={page_number}&page_size={page_size}"
        ))
        .header(Accept::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .dispatch()
        .await
}

#[given(expr = "the workspace has {int} member(s)")]
async fn given_n_members(world: &mut TestWorld, n: u64) -> Result<(), Error> {
    let domain = world.server_config().await?.domain;
    let n = {
        let db = world.db();
        max(0u64, n - Query::get_member_count(db).await?)
    };
    for i in 0..n {
        let db = world.db();
        let jid = &JID::new(format!("person.{i}"), domain.to_owned()).unwrap();
        let model = Mutation::create_user(db, jid, &None).await?;

        let jwt_service: &JWTService = world.client.rocket().state().unwrap();
        let token = jwt_service.generate_jwt(&jid)?;

        world.members.insert(jid.to_string(), (model, token));
    }
    Ok(())
}

#[given("the XMPP server is offline")]
fn given_xmpp_server_offline(world: &mut TestWorld) {
    world.server_ctl.lock().unwrap().online = false;
}

#[when(expr = "{} lists members")]
async fn when_listing_members(world: &mut TestWorld, name: String) {
    let token = world.token(name);
    let res = list_members(&world.client, token).await;
    world.result = Some(res.into());
}

#[when(expr = "{} lists members by pages of {int}")]
async fn when_listing_members_paged(world: &mut TestWorld, name: String, page_size: u64) {
    let token = world.token(name);
    let res = list_members_paged(&world.client, token, 1, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "{} gets page {int} of members by pages of {int}")]
async fn when_getting_members_page(
    world: &mut TestWorld,
    name: String,
    page_number: u64,
    page_size: u64,
) {
    let token = world.token(name);
    let res = list_members_paged(&world.client, token, page_number, page_size).await;
    world.result = Some(res.into());
}

#[when(expr = "{} gets detailed information about {array}")]
async fn when_getting_members_details(world: &mut TestWorld, name: String, jids: Array<JIDParam>) {
    todo!("{jids}")
}

#[then(expr = "they should see {int} member(s)")]
fn then_n_members(world: &mut TestWorld, n: usize) {
    let res: Vec<MemberDTO> = world.result().body_into();
    assert_eq!(res.len(), n)
}

#[then(expr = "<{jid}> should have the {member_role} role")]
async fn then_role(world: &mut TestWorld, jid: JIDParam, role: MemberRole) -> Result<(), DbErr> {
    let db = world.db();

    let member = Member::find_by_jid(&jid)
        .one(db)
        .await?
        .expect(&format!("Member {jid} not found"));
    assert_eq!(member.role, role.0);

    Ok(())
}

#[then(expr = "<{jid}> should have the nickname {string}")]
async fn then_nickname(
    world: &mut TestWorld,
    jid: JIDParam,
    nickname: String,
) -> Result<(), xmpp_service::Error> {
    let ctx = XmppServiceContext {
        bare_jid: test_jid(world).await.unwrap(),
    };
    let vcard = world
        .xmpp_service()
        .get_vcard(&ctx, &jid)?
        .expect("vCard not found");

    assert_eq!(vcard.nickname, vec![Nickname { value: nickname }]);

    Ok(())
}
