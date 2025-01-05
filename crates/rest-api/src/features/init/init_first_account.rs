// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{http::HeaderValue, Json};
use serde::{Deserialize, Serialize};
use service::{
    init::{InitFirstAccountError, InitFirstAccountForm, InitService},
    members::UnauthenticatedMemberService,
    models::JidNode,
    server_config::ServerConfig,
};

use crate::{
    error::prelude::*, features::members::Member, models::SerializableSecretString,
    responders::Created,
};

#[derive(Serialize, Deserialize)]
pub struct InitFirstAccountRequest {
    pub username: JidNode,
    pub password: SerializableSecretString,
    pub nickname: String,
}

pub async fn init_first_account_route(
    init_service: InitService,
    server_config: ServerConfig,
    member_service: UnauthenticatedMemberService,
    Json(req): Json<InitFirstAccountRequest>,
) -> Result<Created<Member>, Error> {
    let member = init_service
        .init_first_account(&server_config, &member_service, req)
        .await?;

    let resource_uri = format!("/v1/members/{jid}", jid = member.jid());
    Ok(Created {
        location: HeaderValue::from_str(&resource_uri)?,
        body: Member::from(member),
    })
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
