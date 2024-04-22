// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::{MemberRole, JID};
use log::debug;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use tokio::runtime::Handle;
use vcard_parser::vcard::Vcard;

use crate::{config::Config, server_ctl::*};

use super::AsProsody as _;

/// Rust interface to [`mod_admin_rest`](https://github.com/wltsmrz/mod_admin_rest/tree/master).
#[derive(Debug)]
pub struct ProsodyAdminRest {
    admin_rest_api_host: String,
    admin_rest_api_port: u16,
    api_auth_username: JID,
    api_auth_password: String,
}

impl ProsodyAdminRest {
    pub fn from_config(config: &Config) -> Self {
        Self {
            admin_rest_api_host: config.server.local_hostname.clone(),
            admin_rest_api_port: config.server.admin_rest_api_port,
            api_auth_username: config.api_jid(),
            api_auth_password: config.api.admin_password.clone().unwrap(),
        }
    }

    fn call_unauthenticated<
        Req: FnOnce(&Client) -> RequestBuilder,
        Accept: FnOnce(&Response) -> bool,
    >(
        &self,
        make_req: Req,
        accept: Accept,
    ) -> Result<Response, Error> {
        let client = Client::new();
        let request = make_req(&client).build()?;
        debug!("Calling `{} {}`…", request.method(), request.url());

        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let response = client.execute(request).await?;
                if accept(&response) {
                    Ok(response)
                } else {
                    Err(Error::Other(format!(
                        "Admin REST API call failed.\n  Status: {}\n  Headers: {:?}\n  Body: {}",
                        response.status(),
                        response.headers().clone(),
                        response.text().await.unwrap_or("<nil>".to_string())
                    )))
                }
            })
        })
    }

    fn call<F: FnOnce(&Client) -> RequestBuilder>(&self, make_req: F) -> Result<Response, Error> {
        self.call_unauthenticated(
            |client| {
                make_req(client).basic_auth(
                    self.api_auth_username.to_string(),
                    Some(self.api_auth_password.clone()),
                )
            },
            |res| res.status().is_success(),
        )
    }

    fn url(&self, path: &str) -> String {
        format!(
            "http://{}:{}/admin_rest/{path}",
            self.admin_rest_api_host, self.admin_rest_api_port
        )
    }
}

impl ServerCtlImpl for ProsodyAdminRest {
    fn reload(&self) -> Result<(), Error> {
        unimplemented!();
    }

    fn add_user(&self, jid: &JID, password: &str) -> Result<(), Error> {
        self.call(|client| {
            client
                .post(format!(
                    "{}/{}",
                    self.url("user"),
                    urlencoding::encode(&jid.to_string())
                ))
                .body(format!(r#"{{"password":"{}"}}"#, password))
        })
        .map(|_| ())
    }
    fn remove_user(&self, jid: &JID) -> Result<(), Error> {
        self.call(|client| {
            client.delete(format!(
                "{}/{}",
                self.url("user"),
                urlencoding::encode(&jid.to_string())
            ))
        })
        .map(|_| ())
    }
    fn set_user_role(&self, jid: &JID, role: &MemberRole) -> Result<(), Error> {
        self.call(|client| {
            client
                .patch(format!(
                    "{}/{}/role",
                    self.url("user"),
                    urlencoding::encode(&jid.to_string()),
                ))
                .body(format!(r#"{{"role":"{}"}}"#, role.as_prosody()))
        })
        .map(|_| ())
    }

    fn test_user_password(&self, jid: &JID, password: &str) -> Result<bool, Error> {
        self.call_unauthenticated(
            |client| {
                client
                    .get(self.url("ping"))
                    .basic_auth(jid.to_string(), Some(password.to_string()))
            },
            |res| res.status().is_success() || res.status() == StatusCode::UNAUTHORIZED,
        )
        .map(|res| res.status().is_success())
    }

    fn get_vcard(&self, jid: &JID) -> Result<Option<Vcard>, Error> {
        self.call(|client| {
            client.get(format!(
                "{}/{}",
                self.url("vcards"),
                urlencoding::encode(&jid.to_string())
            ))
        })
        .and_then(|res| {
            tokio::task::block_in_place(move || Handle::current().block_on(res.text()))
                .map_err(Error::from)
        })
        .and_then(|vcard| {
            let vcards = vcard_parser::parse_vcards(vcard.as_str())?;
            Ok(vcards.into_iter().next())
        })
    }
    fn set_vcard(&self, jid: &JID, vcard: &Vcard) -> Result<(), Error> {
        self.call(|client| {
            client
                .put(format!(
                    "{}/{}",
                    self.url("vcards"),
                    urlencoding::encode(&jid.to_string())
                ))
                .body(vcard.export())
        })
        .map(|_| ())
    }
}
