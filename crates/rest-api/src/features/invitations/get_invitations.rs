// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use rocket::get;
use service::invitations::InvitationController;

use crate::{error::Error, forms::Timestamp, guards::LazyGuard, responders::Paginated};

use super::model::*;

/// Get workspace invitations.
#[get("/v1/invitations?<page_number>&<page_size>&<until>", rank = 2)]
pub(super) async fn get_invitations_route<'r>(
    invitation_controller: LazyGuard<InvitationController>,
    page_number: Option<u64>,
    page_size: Option<u64>,
    until: Option<Timestamp>,
) -> Result<Paginated<WorkspaceInvitation>, Error> {
    let invitation_controller = invitation_controller.inner?;
    let page_number = page_number.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    let until: Option<DateTime<Utc>> = match until {
        Some(t) => Some(t.try_into()?),
        None => None,
    };

    let (pages_metadata, invitations) = invitation_controller
        .get_invitations(page_number, page_size, until)
        .await?;

    Ok(Paginated::new(
        invitations.into_iter().map(Into::into).collect(),
        page_number,
        page_size,
        pages_metadata,
    ))
}
