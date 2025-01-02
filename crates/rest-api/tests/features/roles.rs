// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::{given, then, when};
use prose_pod_api::{error::Error, features::roles::SetMemberRoleRequest};
use rocket::{
    http::{ContentType, Header},
    local::asynchronous::{Client, LocalResponse},
};
use secrecy::{ExposeSecret as _, SecretString};
use serde_json::json;
use service::{members::MemberRepository, members::MemberRole, xmpp::BareJid};

use crate::{cucumber_parameters::MemberRole as MemberRoleParam, TestWorld};

use super::name_to_jid;

// GIVEN

#[given(expr = "{} is an admin")]
async fn given_admin(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();

    let jid = name_to_jid(world, &name).await?;
    match MemberRepository::get(db, &jid).await? {
        Some(_) => {
            MemberRepository::set_role(db, &jid, MemberRole::Admin).await?;
        }
        None => {
            let model = world
                .member_service()
                .create_user(
                    db,
                    &jid,
                    &SecretString::new("password".to_owned()),
                    &name,
                    &Some(MemberRole::Admin),
                )
                .await?;

            let token = world.mock_auth_service.log_in_unchecked(&jid)?;

            world.members.insert(name, (model, token));
        }
    };

    Ok(())
}

#[given(regex = r"^(.+) is (not an admin|a regular member|a member)$")]
async fn given_not_admin(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();

    let jid = name_to_jid(world, &name).await?;
    let model = world
        .member_service()
        .create_user(
            db,
            &jid,
            &SecretString::new("password".to_owned()),
            &name,
            &Some(MemberRole::Member),
        )
        .await?;

    let token = world.mock_auth_service.log_in_unchecked(&jid)?;

    world.members.insert(name, (model, token));

    Ok(())
}

// WHEN

async fn set_member_role<'a>(
    client: &'a Client,
    token: &SecretString,
    jid: &BareJid,
    role: MemberRole,
) -> LocalResponse<'a> {
    client
        .put(format!("/v1/members/{jid}/role"))
        .header(ContentType::JSON)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", token.expose_secret()),
        ))
        .body(json!(SetMemberRoleRequest { role }).to_string())
        .dispatch()
        .await
}

#[when(expr = "{} makes {} an admin")]
async fn when_set_role_admin(
    world: &mut TestWorld,
    actor: String,
    subject: String,
) -> Result<(), Error> {
    let token = world.token(actor);
    let jid = name_to_jid(world, &subject).await?;
    let res = set_member_role(world.client(), &token, &jid, MemberRole::Admin).await;
    world.result = Some(res.into());
    Ok(())
}

#[when(expr = "{} makes {} a regular member")]
async fn when_set_role_member(
    world: &mut TestWorld,
    actor: String,
    subject: String,
) -> Result<(), Error> {
    let token = world.token(actor);
    let jid = name_to_jid(world, &subject).await?;
    let res = set_member_role(world.client(), &token, &jid, MemberRole::Member).await;
    world.result = Some(res.into());
    Ok(())
}

// THEN

#[then(expr = "{} should have the {member_role} role")]
async fn then_role(
    world: &mut TestWorld,
    subject: String,
    role: MemberRoleParam,
) -> Result<(), Error> {
    let jid = name_to_jid(world, &subject).await?;
    let member = MemberRepository::get(world.db(), &jid)
        .await?
        .expect(&format!("Member {jid} not found"));
    assert_eq!(member.role, role.0);

    Ok(())
}

#[then(expr = "{} should have the {string} role in Prosody")]
async fn then_prosody_role(
    world: &mut TestWorld,
    subject: String,
    prosody_role: String,
) -> Result<(), Error> {
    let jid = name_to_jid(world, &subject).await?;
    let member_role = world
        .server_ctl_state()
        .users
        .get(&jid)
        .expect(&format!("Member {jid} not found"))
        .role
        .clone();
    assert_eq!(member_role, prosody_role);

    Ok(())
}