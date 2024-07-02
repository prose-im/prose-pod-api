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
use entity::model::MemberRole;
use prose_pod_api::error::Error;
use service::{prose_xmpp::BareJid, Mutation};

use crate::TestWorld;

async fn name_to_jid(world: &TestWorld, name: &str) -> Result<BareJid, Error> {
    let domain = world.server_config().await?.domain;
    Ok(
        BareJid::new(&format!("{name}@{domain}")).map_err(|err| Error::InternalServerError {
            reason: format!(
                "'{name}' cannot be used in a JID (or '{domain}' isn't a valid domain): {err}"
            ),
        })?,
    )
}

#[given(expr = "{} is an admin")]
async fn given_admin(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();

    let jid = name_to_jid(world, &name).await?;
    let model = Mutation::create_user(db, &jid, &Some(MemberRole::Admin)).await?;

    let token = world.auth_service.log_in_unchecked(&jid)?;

    world.members.insert(name, (model, token));

    Ok(())
}

#[given(regex = r"^(.+) is (not an admin|a regular member)$")]
async fn given_not_admin(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();

    let jid = name_to_jid(world, &name).await?;
    let model = Mutation::create_user(db, &jid, &Some(MemberRole::Member)).await?;

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
        .set_avatar(&jid, Some(avatar.into_bytes()))?;
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
