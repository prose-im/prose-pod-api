// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_pod_api::error::Error;
use service::members::*;

use crate::prelude::mocks::UserAccount;

use super::prelude::*;

// MARK: - Given

#[given(expr = "{} is an admin")]
pub async fn given_admin(world: &mut TestWorld, name: String) -> Result<(), anyhow::Error> {
    let jid = name_to_jid(world, &name).await?;
    let username = jid.expect_username();
    let repository = world.mock_user_repository();

    if repository.user_exists(username, &BYPASS_TOKEN).await? {
        repository
            .set_user_role(username, &MemberRole::Admin, &BYPASS_TOKEN)
            .await?;
    } else {
        repository
            .add_user(
                &Nickname::from_string_unsafe(name),
                UserAccount::admin(jid),
                world.mock_auth_service(),
            )
            .await?;
    }

    Ok(())
}

#[given(regex = r"^(.+) is (not an admin|a regular member|a member)$")]
async fn given_not_admin(world: &mut TestWorld, name: String) -> Result<(), anyhow::Error> {
    let jid = name_to_jid(world, &name).await?;

    world
        .mock_user_repository()
        .add_user(
            &Nickname::from_string_unsafe(name),
            UserAccount::member(jid),
            world.mock_auth_service(),
        )
        .await?;

    Ok(())
}

// MARK: - When

api_call_fn!(
    set_member_role,
    PUT, "/v1/members/{jid}/role"; jid=&BareJid,
    payload: MemberRole
);

#[when(expr = "{} makes {} an admin")]
async fn when_set_role_admin(
    world: &mut TestWorld,
    actor: String,
    subject: String,
) -> Result<(), anyhow::Error> {
    let ref auth = world.token(&actor).await;
    let jid = name_to_jid(world, &subject).await?;

    let res = set_member_role(world.api(), auth, &jid, MemberRole::Admin).await?;
    world.result = Some(res.into());

    Ok(())
}

#[when(expr = "{} makes {} a regular member")]
async fn when_set_role_member(
    world: &mut TestWorld,
    actor: String,
    subject: String,
) -> Result<(), anyhow::Error> {
    let ref auth = world.token(&actor).await;
    let jid = name_to_jid(world, &subject).await?;

    let res = set_member_role(world.api(), auth, &jid, MemberRole::Member).await?;
    world.result = Some(res.into());

    Ok(())
}

// MARK: - Then

#[then(expr = "{} should have the {member_role} role")]
async fn then_role(
    world: &mut TestWorld,
    subject: String,
    expected_role: parameters::MemberRole,
) -> Result<(), Error> {
    let jid = name_to_jid(world, &subject).await?;

    let role = world.mock_user_repository().role(&jid).unwrap();
    assert_eq!(role, expected_role.0);

    Ok(())
}

#[then(expr = "{} should have the {string} role in Prosody")]
async fn then_prosody_role(
    world: &mut TestWorld,
    subject: String,
    expected_role: String,
) -> Result<(), Error> {
    let jid = name_to_jid(world, &subject).await?;

    let role = world.mock_user_repository().role(&jid).unwrap();
    assert_eq!(role.as_prosody().to_string(), expected_role);

    Ok(())
}
