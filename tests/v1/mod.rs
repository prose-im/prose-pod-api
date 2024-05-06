// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod init;
pub mod invitations;
pub mod members;
pub mod server;
pub mod workspace;

use std::str::FromStr;

use cucumber::given;
use entity::member;
use entity::model::{self, JIDNode, MemberRole, JID};
use prose_pod_api::error::Error;
use prose_pod_api::guards::JWTService;
use service::sea_orm::{ActiveModelTrait, Set};

use crate::TestWorld;

use self::init::DEFAULT_DOMAIN;

fn name_to_jid(name: &str) -> JID {
    let jid_node = name.to_lowercase().replace(" ", "-");
    model::JID {
        node: JIDNode::from_str(&jid_node)
            .expect(&format!("JID node '{}' constructed from '{}' is invalid. Choose a different name or improve this function.", jid_node, name)),
        domain: DEFAULT_DOMAIN.to_owned(),
    }
}

#[given(regex = r"^(.+) is an admin$")]
async fn given_admin(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();

    let jid = name_to_jid(&name);
    let mut model = member::ActiveModel {
        role: Set(MemberRole::Admin),
        ..Default::default()
    };
    model.set_username(&jid.node);
    let model = model.insert(db).await?;

    let jwt_service: &JWTService = world.client.rocket().state().unwrap();
    let token = jwt_service.generate_jwt(&jid)?;

    world.members.insert(name, (model, token));

    Ok(())
}

#[given(regex = r"^(.+) is (not an admin|a regular member)$")]
async fn given_not_admin(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();

    let jid = name_to_jid(&name);
    let mut model = member::ActiveModel {
        role: Set(MemberRole::Member),
        ..Default::default()
    };
    model.set_username(&jid.node);
    let model = model.insert(db).await?;

    let jwt_service: &JWTService = world.client.rocket().state().unwrap();
    let token = jwt_service.generate_jwt(&jid)?;

    world.members.insert(name, (model, token));

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
