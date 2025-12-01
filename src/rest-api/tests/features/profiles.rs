// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    members::Nickname,
    prose_xmpp::stanza::vcard4,
    xmpp::{XmppServiceContext, XmppServiceImpl},
};

use super::prelude::*;

// MARK: - Given

#[given(expr = "{}’s nickname is {string}")]
#[given(expr = "{}'s nickname is {string}")]
async fn given_nickname(
    world: &mut TestWorld,
    name: String,
    nickname: String,
) -> Result<(), Error> {
    let auth_token = world.token(&name).await;
    let jid = name_to_jid(world, &name).await?;

    world
        .mock_xmpp_service()
        .set_own_nickname(
            &XmppServiceContext {
                bare_jid: jid,
                auth_token,
            },
            &Nickname::from_string_unsafe(nickname),
        )
        .await?;

    Ok(())
}

// MARK: - When

api_call_fn!(
    set_member_nickname,
    PUT,
    "/v1/members/{jid}/nickname"; jid=BareJid,
    payload: String,
);

#[when(expr = "{} sets {}’s nickname to {string}")]
#[when(expr = "{} sets {}'s nickname to {string}")]
async fn when_set_nickname(
    world: &mut TestWorld,
    actor: String,
    subject: String,
    nickname: String,
) -> Result<(), Error> {
    let ref auth = world.token(&actor).await;
    let jid = name_to_jid(world, &subject).await?;
    let res = set_member_nickname(world.api(), auth, jid, nickname).await;
    world.result = Some(res.unwrap().into());
    Ok(())
}

#[when(expr = "{} sets their nickname to {string}")]
async fn when_set_nickname_self(
    world: &mut TestWorld,
    name: String,
    nickname: String,
) -> Result<(), Error> {
    when_set_nickname(world, name.clone(), name, nickname).await
}

// MARK: - Then

#[then(expr = "{}’s nickname should be {string}")]
#[then(expr = "{}'s nickname should be {string}")]
async fn then_nickname(world: &mut TestWorld, name: String, nickname: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    let vcard = world
        .mock_xmpp_service()
        .get_vcard(&jid)?
        .expect("vCard not found");

    assert_eq!(vcard.nickname, vec![vcard4::Nickname { value: nickname }]);

    Ok(())
}
