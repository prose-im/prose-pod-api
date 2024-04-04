// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::JID;
use rocket::outcome::try_outcome;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use service::ServerCtl;

use crate::error::{self, Error};

pub struct UserFactory<'r> {
    server_ctl: &'r State<ServerCtl>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserFactory<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let server_ctl =
            try_outcome!(req
                .guard::<&State<ServerCtl>>()
                .await
                .map_error(|(status, _)| (
                    status,
                    Error::InternalServerError {
                        reason: "Could not get a `&State<ServerCtl>` from a request.".to_string(),
                    }
                )));

        Outcome::Success(Self { server_ctl })
    }
}

impl<'r> UserFactory<'r> {
    pub async fn create_user(
        &self,
        jid: &JID,
        password: &str,
        nickname: &str,
    ) -> Result<(), Error> {
        let server_ctl = self.server_ctl.lock().expect("Serverctl lock poisonned");

        server_ctl.add_user(jid, password)?;
        // TODO: Create the vCard using a display name instead of the nickname
        server_ctl.create_vcard(jid, nickname)?;
        server_ctl.set_nickname(jid, nickname)?;

        Ok(())
    }
}
