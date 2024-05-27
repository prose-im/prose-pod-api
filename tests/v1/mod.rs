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
use entity::model::{self, JIDNode, MemberRole, JID};
use prose_pod_api::error::Error;
use prose_pod_api::guards::JWTService;
use service::Mutation;

use crate::TestWorld;

async fn name_to_jid(world: &TestWorld, name: &str) -> Result<JID, Error> {
    let jid_node = name.to_lowercase().replace(" ", "-");
    let domain = world.server_config().await?.domain;
    Ok(model::JID {
        node: JIDNode::from_str(&jid_node)
            .expect(&format!("JID node '{}' constructed from '{}' is invalid. Choose a different name or improve this function.", jid_node, name)),
        domain,
    })
}

#[given(expr = "{} is an admin")]
async fn given_admin(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();

    let jid = name_to_jid(world, &name).await?;
    let model = Mutation::create_user(db, &jid, &Some(MemberRole::Admin)).await?;

    let jwt_service: &JWTService = world.client.rocket().state().unwrap();
    let token = jwt_service.generate_jwt(&jid)?;

    world.members.insert(name, (model, token));

    Ok(())
}

#[given(regex = r"^(.+) is (not an admin|a regular member)$")]
async fn given_not_admin(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();

    let jid = name_to_jid(world, &name).await?;
    let model = Mutation::create_user(db, &jid, &Some(MemberRole::Member)).await?;

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
