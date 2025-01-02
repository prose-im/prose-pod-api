// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{response::status, serde::json::Json};
use serde::{Deserialize, Serialize};
use service::{
    init::{InitFirstAccountError, InitFirstAccountForm, InitService},
    members::UnauthenticatedMemberService,
    models::JidNode,
    server_config::ServerConfig,
};

use crate::{
    error::prelude::*,
    features::members::{rocket_uri_macro_get_member_route, Member},
    forms::JID as JIDUriParam,
    guards::LazyGuard,
    models::SerializableSecretString,
    responders::Created,
};

#[derive(Serialize, Deserialize)]
pub struct InitFirstAccountRequest {
    pub username: JidNode,
    pub password: SerializableSecretString,
    pub nickname: String,
}

#[rocket::put("/v1/init/first-account", format = "json", data = "<req>")]
pub async fn init_first_account_route(
    init_service: LazyGuard<InitService>,
    server_config: LazyGuard<ServerConfig>,
    member_service: LazyGuard<UnauthenticatedMemberService>,
    req: Json<InitFirstAccountRequest>,
) -> Created<Member> {
    let init_service = init_service.inner?;
    let server_config = &server_config.inner?;
    let member_service = &member_service.inner?;
    let form = req.into_inner();

    let member = init_service
        .init_first_account(server_config, member_service, form)
        .await?;

    let resource_uri = rocket::uri!(get_member_route(member.jid())).to_string();
    let response = Member::from(member);
    Ok(status::Created::new(resource_uri).body(response.into()))
}

pub async fn init_first_account_route_axum() {
    todo!()
}

// ERRORS

impl ErrorCode {
    pub const FIRST_ACCOUNT_ALREADY_CREATED: Self = Self {
        value: "first_account_already_created",
        http_status: StatusCode::CONFLICT,
        log_level: LogLevel::Info,
    };
}

impl CustomErrorCode for InitFirstAccountError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::FirstAccountAlreadyCreated => ErrorCode::FIRST_ACCOUNT_ALREADY_CREATED,
            Self::InvalidJid(_) => ErrorCode::BAD_REQUEST,
            Self::CouldNotCreateFirstAccount(err) => err.code(),
            Self::DbErr(err) => err.code(),
        }
    }
}
impl_into_error!(InitFirstAccountError);

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
