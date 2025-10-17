// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::State, http::HeaderValue, Json};
use service::{
    invitations::invitation_service::AcceptAccountInvitationCommand,
    members::{Member, MemberService, Nickname},
    xmpp::JidNode,
};
use validator::Validate;

use crate::{error::prelude::*, features::auth::models::Password, responders::Created, AppState};

// MARK: - Init first account

#[derive(Debug, Validate, serdev::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(validate = "Validate::validate")]
#[cfg_attr(feature = "test", derive(serdev::Serialize))]
pub struct InitFirstAccountRequest {
    #[validate(skip)] // NOTE: Already parsed.
    pub username: JidNode,

    #[validate(skip)] // NOTE: Will be checked later.
    pub password: Password,

    #[validate(nested)]
    pub nickname: Nickname,
}

pub async fn init_first_account_route(
    ref member_service: MemberService,
    Json(req): Json<InitFirstAccountRequest>,
) -> Result<Created<Member>, Error> {
    let member = member_service
        .create_first_acount(
            &req.username,
            &AcceptAccountInvitationCommand {
                nickname: req.nickname,
                password: req.password.into(),
                email: None,
            },
        )
        .await?;

    let resource_uri = format!("/v1/members/{jid}", jid = member.jid);
    Ok(Created {
        location: HeaderValue::from_str(&resource_uri)?,
        body: Member::from(member),
    })
}

pub async fn is_first_account_created_route(
    State(AppState {
        ref user_repository,
        ..
    }): State<AppState>,
) -> StatusCode {
    let user_count = user_repository
        .users_stats(None)
        .await
        .map_or(0, |stats| stats.count);
    if user_count == 0 {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::OK
    }
}
