// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use axum::{extract::State, http::HeaderValue, Json};
use service::{
    init::{InitFirstAccountForm, InitService},
    members::{Member, MemberRepository, Nickname, UnauthenticatedMemberService},
    xmpp::JidNode,
    AppConfig,
};
use validator::Validate;

use crate::{error::prelude::*, features::auth::models::Password, responders::Created, AppState};

// MARK: INIT FIRST ACCOUNT

#[derive(Debug, Validate, serdev::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(validate = "Validate::validate")]
#[cfg_attr(feature = "test", derive(serdev::Serialize))]
pub struct InitFirstAccountRequest {
    #[validate(skip)] // NOTE: Already parsed.
    pub username: JidNode,

    #[validate(nested)]
    pub password: Password,

    #[validate(nested)]
    pub nickname: Nickname,
}

pub async fn init_first_account_route(
    State(ref app_config): State<Arc<AppConfig>>,
    init_service: InitService,
    ref member_service: UnauthenticatedMemberService,
    Json(req): Json<InitFirstAccountRequest>,
) -> Result<Created<Member>, Error> {
    let ref server_domain = app_config.server_domain();

    let member = init_service
        .init_first_account(server_domain, member_service, req)
        .await?;

    let resource_uri = format!("/v1/members/{jid}", jid = member.jid);
    Ok(Created {
        location: HeaderValue::from_str(&resource_uri)?,
        body: Member::from(member),
    })
}

pub async fn is_first_account_created_route(
    State(AppState { db, .. }): State<AppState>,
) -> StatusCode {
    if MemberRepository::count(&db).await.unwrap_or_default() == 0 {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::OK
    }
}

// MARK: BOILERPLATE

impl Into<InitFirstAccountForm> for InitFirstAccountRequest {
    fn into(self) -> InitFirstAccountForm {
        InitFirstAccountForm {
            username: self.username,
            password: self.password.into(),
            nickname: self.nickname,
        }
    }
}
