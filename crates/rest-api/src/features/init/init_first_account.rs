// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{response::status, serde::json::Json};
use serde::{Deserialize, Serialize};
use service::{
    controllers::init_controller::{InitController, InitFirstAccountForm},
    model::{JidNode, ServerConfig},
};

use crate::{
    features::members::{rocket_uri_macro_get_member_route, Member},
    forms::JID as JIDUriParam,
    guards::{LazyGuard, UnauthenticatedUserService},
    model::SerializableSecretString,
    responders::Created,
};

#[derive(Serialize, Deserialize)]
pub struct InitFirstAccountRequest {
    pub username: JidNode,
    pub password: SerializableSecretString,
    pub nickname: String,
}

#[put("/v1/init/first-account", format = "json", data = "<req>")]
pub async fn init_first_account_route<'r>(
    init_controller: LazyGuard<InitController<'r>>,
    server_config: LazyGuard<ServerConfig>,
    user_service: LazyGuard<UnauthenticatedUserService<'r>>,
    req: Json<InitFirstAccountRequest>,
) -> Created<Member> {
    let init_controller = init_controller.inner?;
    let server_config = &server_config.inner?;
    let user_service = &user_service.inner?;
    let form = req.into_inner();

    let member = init_controller
        .init_first_account(server_config, user_service, form)
        .await?;

    let resource_uri = uri!(get_member_route(member.jid())).to_string();
    let response = Member::from(member);
    Ok(status::Created::new(resource_uri).body(response.into()))
}

// BOILERPLATE

impl Into<InitFirstAccountForm> for InitFirstAccountRequest {
    fn into(self) -> InitFirstAccountForm {
        InitFirstAccountForm {
            username: self.username,
            password: self.password.into(),
            nickname: self.nickname,
        }
    }
}
