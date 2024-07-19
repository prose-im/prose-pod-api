// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod init;
pub mod invitations;
pub mod members;
pub mod server;
pub mod workspace;

use cucumber::given;
use prose_pod_api::error::{self, Error};
use service::{
    model::MemberRole,
    prose_xmpp::{mods::AvatarData, BareJid},
    repositories::{MemberCreateForm, MemberRepository},
};

use crate::TestWorld;

async fn name_to_jid(world: &TestWorld, name: &str) -> Result<BareJid, Error> {
    let domain = world.server_config().await?.domain;
    Ok(BareJid::new(&format!("{name}@{domain}")).map_err(|err| {
        error::InternalServerError(format!(
            "'{name}' cannot be used in a JID (or '{domain}' isn't a valid domain): {err}"
        ))
    })?)
}

#[given(expr = "{} is an admin")]
async fn given_admin(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();

    let jid = name_to_jid(world, &name).await?;
    let member = MemberCreateForm {
        jid: jid.clone(),
        role: Some(MemberRole::Admin),
        joined_at: None,
    };
    let model = MemberRepository::create(db, member).await?;

    let token = world.auth_service.log_in_unchecked(&jid)?;

    world.members.insert(name, (model, token));

    Ok(())
}

#[given(regex = r"^(.+) is (not an admin|a regular member)$")]
async fn given_not_admin(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();

    let jid = name_to_jid(world, &name).await?;
    let member = MemberCreateForm {
        jid: jid.clone(),
        role: Some(MemberRole::Member),
        joined_at: None,
    };
    let model = MemberRepository::create(db, member).await?;

    let token = world.auth_service.log_in_unchecked(&jid)?;

    world.members.insert(name, (model, token));

    Ok(())
}

#[given(regex = "^(\\w+) is (online|offline)$")]
async fn given_presence(
    world: &mut TestWorld,
    name: String,
    presence: String,
) -> Result<(), Error> {
    let mut state = world.xmpp_service_state_mut();

    let jid = name_to_jid(world, &name).await?;
    println!("{} is {}\n", name, presence);
    match presence.as_str() {
        "online" => state.online_members.insert(jid),
        "offline" => state.online_members.remove(&jid),
        p => panic!("Unexpected presence: '{p}'"),
    };

    Ok(())
}

#[given(expr = "{}'s avatar is {}")]
async fn given_avatar(world: &mut TestWorld, name: String, avatar: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    world
        .xmpp_service
        .set_avatar(&jid, Some(AvatarData::Base64(avatar)))?;
    Ok(())
}

#[given(expr = "{} has no avatar")]
async fn given_no_avatar(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    world.xmpp_service.set_avatar(&jid, None)?;
    Ok(())
}

// LOGIN

// async fn login<'a>(client: &'a Client) -> LocalResponse<'a> {
//     client
//         .post("/v1/login")
//         .header(ContentType::JSON)
//         .dispatch()
//         .await
// }
