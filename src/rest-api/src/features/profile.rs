// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::middleware::from_extractor_with_state;
use axum::routing::{put, MethodRouter};
use service::auth::Authenticated;

use crate::AppState;

pub use self::routes::*;

use super::members::MEMBER_ROUTE;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            MEMBER_ROUTE,
            axum::Router::new()
                .route("/avatar", put(set_member_avatar_route))
                .route("/nickname", put(set_member_nickname_route))
                .route(
                    "/email-address",
                    MethodRouter::new()
                        .put(set_member_email_address_route)
                        .get(get_member_email_address_route),
                ),
        )
        .route_layer(from_extractor_with_state::<Authenticated, _>(
            app_state.clone(),
        ))
        .with_state(app_state)
}

mod routes {
    use axum::{
        extract::{Path, State},
        response::NoContent,
        Json,
    };
    use service::{
        auth::UserInfo,
        members::{MemberRepository, MemberRole, Nickname},
        models::{Avatar, BareJid, EmailAddress},
        xmpp::XmppService,
    };

    use crate::{
        error::{self, Error},
        AppState,
    };

    pub async fn set_member_avatar_route<'a>(
        Path(member_id): Path<BareJid>,
        UserInfo { jid, .. }: UserInfo,
        xmpp_service: XmppService,
        avatar: Avatar<'a>,
    ) -> Result<NoContent, Error> {
        if jid != member_id {
            Err(error::Forbidden(
                "You can’t change someone else’s avatar.".to_string(),
            ))?
        }

        xmpp_service.set_own_avatar(avatar).await?;

        Ok(NoContent)
    }

    pub async fn set_member_email_address_route(
        State(AppState { ref db, .. }): State<AppState>,
        Path(jid): Path<BareJid>,
        caller: UserInfo,
        Json(email_address): Json<EmailAddress>,
    ) -> Result<NoContent, Error> {
        if !(caller.jid == jid || caller.role == MemberRole::Admin) {
            Err(error::Forbidden("You cannot do that.".to_string()))?
        }

        MemberRepository::set_email_address(&db.write, &jid, Some(email_address)).await?;

        Ok(NoContent)
    }

    pub async fn get_member_email_address_route(
        State(AppState { ref db, .. }): State<AppState>,
        Path(jid): Path<BareJid>,
        caller: UserInfo,
    ) -> Result<Json<Option<EmailAddress>>, Error> {
        if !(caller.jid == jid || caller.role == MemberRole::Admin) {
            Err(error::Forbidden("You cannot do that.".to_string()))?
        }

        let email_address = MemberRepository::get_email_address(&db.read, &jid).await?;

        Ok(Json(email_address))
    }

    pub async fn set_member_nickname_route(
        Path(member_id): Path<BareJid>,
        UserInfo { jid, .. }: UserInfo,
        xmpp_service: XmppService,
        Json(req): Json<Nickname>,
    ) -> Result<NoContent, Error> {
        if jid != member_id {
            Err(error::Forbidden(
                "You can’t change someone else’s nickname.".to_string(),
            ))?
        }

        xmpp_service.set_own_nickname(&req).await?;

        Ok(NoContent)
    }
}
