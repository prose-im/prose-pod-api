// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::anyhow;
use jid::BareJid;
use sea_orm::DatabaseConnection;
use tracing::instrument;

use crate::{
    members::{MemberRepository, MemberRole, MemberService},
    util::either::{Either, Either3},
};

use super::{
    errors::{CannotAssignRole, CannotChangeOwnRole, InvalidCredentials},
    AuthService, AuthToken, Credentials, UserInfo,
};

/// Log user in and return an authentication token.
#[instrument(
    name = "auth_controller::log_in",
    level = "trace",
    skip_all, fields(jid = credentials.jid.to_string()),
)]
pub async fn log_in(
    credentials: &Credentials,
    auth_service: &AuthService,
) -> Result<AuthToken, Either<InvalidCredentials, anyhow::Error>> {
    auth_service
        .log_in(&credentials.jid, &credentials.password)
        .await
}

#[instrument(
    name = "auth_controller::set_member_role",
    level = "trace",
    skip_all, fields(jid = jid.to_string(), role = role.to_string(), caller = user_info.jid.to_string()),
)]
pub async fn set_member_role(
    db: &DatabaseConnection,
    member_service: &MemberService,
    user_info: &UserInfo,
    jid: BareJid,
    role: MemberRole,
) -> Result<(), Either3<CannotChangeOwnRole, CannotAssignRole, anyhow::Error>> {
    let Some(caller) = MemberRepository::get(db, &user_info.jid).await? else {
        return Err(Either3::E3(anyhow!("Cannot get role for '{jid}'.")));
    };
    if caller.jid() == jid {
        return Err(Either3::E1(CannotChangeOwnRole));
    };
    if caller.role < role {
        return Err(Either3::E2(CannotAssignRole));
    };

    member_service.set_member_role(&jid, role).await?;

    Ok(())
}
