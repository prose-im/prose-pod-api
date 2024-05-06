// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::then;
use entity::prelude::Member;
use migration::DbErr;
use service::{
    server_ctl::{self, ServerCtlImpl},
    vcard_parser::{
        constants::PropertyName,
        traits::HasValue as _,
        vcard::{self, property::property_nickname::PropertyNickNameData},
    },
};

use crate::{
    cucumber_parameters::{MemberRole, JID},
    TestWorld,
};

#[then(expr = "<{jid}> should have the {member_role} role")]
async fn then_role(world: &mut TestWorld, jid: JID, role: MemberRole) -> Result<(), DbErr> {
    let db = world.db();

    let member = Member::find_by_username(&jid.node)
        .one(db)
        .await?
        .expect(&format!("Member {jid} not found"));
    assert_eq!(member.role, role.0);

    Ok(())
}

#[then(expr = "<{jid}> should have the nickname {string}")]
async fn then_nickname(
    world: &mut TestWorld,
    jid: JID,
    nickname: String,
) -> Result<(), server_ctl::Error> {
    let vcard = world
        .server_ctl()
        .get_vcard(&jid)?
        .expect("vCard not found");
    let properties = vcard.get_properties_by_name(PropertyName::NICKNAME);
    let properties = properties
        .iter()
        .map(vcard::property::Property::get_value)
        .collect::<Vec<_>>();

    let expected = PropertyNickNameData::try_from((None, nickname.as_str(), vec![])).unwrap();
    assert_eq!(properties, vec![expected.get_value()]);

    Ok(())
}
