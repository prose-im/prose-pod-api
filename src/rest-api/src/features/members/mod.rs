// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod delete_member;
mod enrich_members;
mod get_member;
mod get_members;
mod guards;
mod model;

use axum::http::StatusCode;
use axum::middleware::from_extractor_with_state;
use axum::routing::{delete, get};
use axum_extra::handler::HandlerCallWithExtractors as _;
use service::members::{UserCreateError, UserDeleteError};

use crate::error::{CustomErrorCode, ErrorCode, HttpApiError as _, LogLevel};
use crate::util::content_type_or::*;
use crate::{impl_into_error, AppState};

use super::auth::guards::{Authenticated, IsAdmin};

pub use self::delete_member::*;
pub use self::enrich_members::*;
pub use self::get_member::*;
pub use self::get_members::*;
pub use self::model::*;

pub(crate) const MEMBERS_ROUTE: &'static str = "/v1/members";
pub(crate) const MEMBER_ROUTE: &'static str = "/v1/members/{jid}";

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route(
            "/v1/enrich-members",
            get(
                with_accept::<TextEventStream, _>(enrich_members_stream_route)
                    .or(enrich_members_route),
            ),
        )
        .nest(
            MEMBERS_ROUTE,
            axum::Router::new()
                .route("/", get(get_members_route))
                .route("/{jid}", get(get_member_route))
                .route(
                    "/{jid}",
                    delete(delete_member_route)
                        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone())),
                ),
        )
        .route_layer(from_extractor_with_state::<Authenticated, _>(
            app_state.clone(),
        ))
        .with_state(app_state)
}

impl ErrorCode {
    const MEMBER_LIMIT_REACHED: Self = Self {
        value: "member_limit_reached",
        http_status: StatusCode::FORBIDDEN,
        log_level: LogLevel::Error,
    };
}
impl CustomErrorCode for UserCreateError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::DbErr(err) => err.code(),
            Self::CouldNotCreateVCard(_)
            | Self::XmppServerCannotCreateUser(_)
            | Self::XmppServerCannotAddTeamMember(_)
            | Self::XmppServerCannotSetUserRole(_) => ErrorCode::INTERNAL_SERVER_ERROR,
            Self::LimitReached => ErrorCode::MEMBER_LIMIT_REACHED,
        }
    }
}
impl_into_error!(UserCreateError);

impl CustomErrorCode for UserDeleteError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::CannotSelfRemove => ErrorCode::FORBIDDEN,
            Self::DbErr(err) => err.code(),
            Self::XmppServerCannotRemoveTeamMember(err) => err.code(),
            Self::XmppServerCannotDeleteUser(err) => err.code(),
        }
    }
}
impl_into_error!(UserDeleteError);
