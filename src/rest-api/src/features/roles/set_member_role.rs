// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::either::Either;
use service::{
    auth::UserInfo,
    members::{MemberRepository, MemberRole, MemberService, SetMemberRoleError},
    xmpp::BareJid,
};

use crate::{
    error::{self, CustomErrorCode, Error, ErrorCode, HttpApiError},
    features::members::Member,
    impl_into_error, AppState,
};

pub async fn set_member_role_route(
    State(AppState { db, .. }): State<AppState>,
    Path(jid): Path<BareJid>,
    member_service: MemberService,
    user_info: UserInfo,
    Json(role): Json<MemberRole>,
) -> Result<Either<Json<Member>, StatusCode>, Error> {
    {
        let Some(caller) = MemberRepository::get(&db, &user_info.jid).await? else {
            return Err(Error::from(error::Forbidden(format!(
                "Cannot get role for '{jid}'."
            ))));
        };
        if caller.jid() == jid {
            return Err(Error::from(error::Forbidden(
                "Cannot change your own role.".to_string(),
            )));
        };
        if caller.role < role {
            return Err(Error::from(error::Forbidden(
                "Cannot give a role you don't have.".to_string(),
            )));
        };
    }

    match member_service.set_member_role(&jid, role).await? {
        Some(member) => {
            let response = Member::from(member);
            Ok(Either::E1(response.into()))
        }
        None => Ok(Either::E2(StatusCode::NO_CONTENT)),
    }
}

impl CustomErrorCode for SetMemberRoleError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::DbErr(err) => err.code(),
            Self::XmppServerCannotSetUserRole(err) => err.code(),
        }
    }
}
impl_into_error!(SetMemberRoleError);
