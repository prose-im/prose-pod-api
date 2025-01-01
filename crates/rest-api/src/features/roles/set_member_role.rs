// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{put, response::status::NoContent, serde::json::Json};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::{
    auth::UserInfo,
    members::{MemberRepository, MemberRole, MemberService, SetMemberRoleError},
};

use crate::{
    error::{self, CustomErrorCode, Error, ErrorCode, HttpApiError},
    features::members::Member,
    forms::JID as JIDUriParam,
    guards::{Db, LazyGuard},
    impl_into_error,
    responders::Either,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SetMemberRoleRequest {
    pub role: MemberRole,
}

#[put("/v1/members/<jid>/role", format = "json", data = "<req>")]
pub async fn set_member_role_route<'r>(
    conn: Connection<'r, Db>,
    jid: JIDUriParam,
    req: Json<SetMemberRoleRequest>,
    member_service: LazyGuard<MemberService>,
    user_info: LazyGuard<UserInfo>,
) -> Result<Either<Json<Member>, NoContent>, Error> {
    let db = conn.into_inner();
    let req = req.into_inner();
    let member_service = member_service.inner?;

    {
        let Some(caller) = MemberRepository::get(db, &user_info.inner?.jid).await? else {
            return Err(Error::from(error::Forbidden(format!(
                "Cannot get role for '{jid}'."
            ))));
        };
        if caller.jid() == jid.0 {
            return Err(Error::from(error::Forbidden(
                "Cannot change your own role.".to_string(),
            )));
        };
        if caller.role < req.role {
            return Err(Error::from(error::Forbidden(
                "Cannot give a role you don't have.".to_string(),
            )));
        };
    }

    match member_service.set_member_role(&jid, req.role).await? {
        Some(member) => {
            let response = Member::from(member);
            Ok(Either::left(response.into()))
        }
        None => Ok(Either::right(NoContent)),
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
