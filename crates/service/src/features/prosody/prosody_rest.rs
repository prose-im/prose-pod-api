// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Debug, sync::Arc};

use anyhow::anyhow;
use async_trait::async_trait;
use minidom::Element;
use parking_lot::RwLock;
use prose_xmpp::{
    client::ConnectorProvider,
    connector::{
        Connection as ConnectionTrait, ConnectionError, ConnectionEvent, ConnectionEventHandler,
        Connector as ConnectorTrait,
    },
};
use reqwest::Client as HttpClient;
use secrecy::{ExposeSecret as _, SecretString};
use tokio::runtime::Handle;
use tracing::{debug, trace};

use crate::models::FullJid;

/// Rust interface to [`mod_http_rest`](https://hg.prosody.im/prosody-modules/file/tip/mod_http_rest).
#[derive(Debug, Clone)]
pub struct ProsodyRest {
    connection: Connection,
}

impl ProsodyRest {
    pub fn provider(http_client: HttpClient, rest_api_url: String) -> ConnectorProvider {
        Box::new(move || {
            Box::new(Self {
                connection: Connection {
                    http_client: http_client.clone(),
                    rest_api_url: rest_api_url.clone(),
                    jid: Default::default(),
                    prosody_token: Default::default(),
                    inner: Default::default(),
                },
            })
        })
    }
}

// MARK: Connector

#[async_trait]
impl ConnectorTrait for ProsodyRest {
    async fn connect(
        &self,
        jid: &FullJid,
        password: SecretString,
        event_handler: ConnectionEventHandler,
    ) -> Result<Box<dyn ConnectionTrait>, ConnectionError> {
        *self.connection.inner.event_handler.write() = Some(event_handler);
        *self.connection.jid.write() = Some(jid.clone());
        *self.connection.prosody_token.write() = Some(password);
        Ok(Box::new(self.connection.clone()))
    }
}

// MARK: Connection

#[derive(Debug, Clone)]
pub struct Connection {
    http_client: HttpClient,
    rest_api_url: String,
    jid: Arc<RwLock<Option<FullJid>>>,
    prosody_token: Arc<RwLock<Option<SecretString>>>,
    inner: Arc<ConnectionInner>,
}

#[derive(Default)]
struct ConnectionInner {
    event_handler: RwLock<Option<ConnectionEventHandler>>,
}

impl Debug for ConnectionInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(ConnectionInner))
            .field(
                "event_handler",
                if self.event_handler.read().is_some() {
                    &"Some"
                } else {
                    &"None"
                },
            )
            .finish()
    }
}

impl Connection {
    pub async fn receive_stanza(&self, stanza: impl Into<Element>) {
        let guard = self.inner.event_handler.read();
        let event_handler = guard.as_ref().expect("No event handler registered");
        let conn = Connection {
            http_client: self.http_client.clone(),
            inner: self.inner.clone(),
            rest_api_url: self.rest_api_url.clone(),
            jid: self.jid.clone(),
            prosody_token: self.prosody_token.clone(),
        };
        (event_handler)(&conn, ConnectionEvent::Stanza(stanza.into())).await
    }
}

impl ConnectionTrait for Connection {
    fn send_stanza(&self, stanza: Element) -> anyhow::Result<()> {
        let Some(token) = (*self.prosody_token.read()).clone() else {
            Err(anyhow!("Logic error: Cannot authenticate Prosody REST API call. Call `ProsodyRest.connect` before `Connection.send_stanza`."))?
        };

        let client = self.http_client.clone();
        let request_body = String::from(&stanza);
        trace!(
            "Sending stanza as <{}>: {stanza:#?}…\nSerialized `request_body`: {request_body}",
            self.jid.read().clone().unwrap(),
        );
        let request = client
            .post(self.rest_api_url.to_owned())
            .header("Content-Type", "application/xmpp+xml")
            .body(request_body)
            .bearer_auth(token.expose_secret())
            .build()?;
        debug!("Calling `{} {}`…", request.method(), request.url());

        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let (response, request_clone) = {
                    let request_clone = request.try_clone();
                    (client.execute(request).await?, request_clone)
                };
                if !response.status().is_success() {
                    let mut err = format!(
                        "Prosody REST API call failed.\n  Status: {}\n  Headers: {:?}\n  Body: {}",
                        response.status(),
                        response.headers().clone(),
                        response.text().await.unwrap_or("<none>".to_owned()),
                    );
                    if let Some(request) = request_clone {
                        err.push_str(&format!(
                            "\n  Request headers: {:?}\n  Request body: {}",
                            request.headers().clone(),
                            request
                                .body()
                                .and_then(|body| body.as_bytes())
                                .map(std::str::from_utf8)
                                .unwrap_or(Ok("<none>"))
                                .map(ToOwned::to_owned)
                                .unwrap_or_else(|err| format!("<error: {err}>")),
                        ));
                    }
                    return Err(anyhow!("Unexpected Prosody REST API response: {err}"));
                }
                let response_body = response.text().await?;
                trace!("response_body: {response_body:?}");
                let xml = format!(r#"<wrapper xmlns="jabber:client">{response_body}</wrapper>"#);
                let wrapper = xml.parse::<Element>()?;
                let stanza = wrapper
                    .get_child("iq", "jabber:client")
                    .expect(&format!(
                        "Prosody response is not an `iq` stanza (`{response_body}`).",
                    ))
                    .to_owned();
                self.receive_stanza(stanza).await;
                Result::<_, anyhow::Error>::Ok(())
            })
        })?;

        Ok(())
    }

    fn disconnect(&self) {}
}
