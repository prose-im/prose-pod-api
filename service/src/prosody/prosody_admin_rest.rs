// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::JID;
use log::debug;
use reqwest::{Client, RequestBuilder, Response};
use tokio::runtime::Handle;
use vcard_parser::vcard::Vcard;

use crate::{config::Config, server_ctl::*};

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

    fn call<F: FnOnce(&Client) -> RequestBuilder>(&self, req: F) -> Result<Response, Error> {
        let client = Client::new();
        let request = req(&client)
            .basic_auth(
                self.api_auth_username.to_string(),
                Some(self.api_auth_password.clone()),
            )
            .build()?;
        debug!("Calling `{} {}`…", request.method(), request.url());

        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let response = client.execute(request).await?;
                if response.status().is_success() {
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
