// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::Arc;

use prose_xmpp::client::ConnectorProvider;
use prose_xmpp::{Client, ClientBuilder};

#[derive(Clone)]
pub struct XMPPClient {
    pub(crate) client: Arc<Client>,
}

impl XMPPClient {
    pub fn builder() -> XMPPClientBuilder {
        XMPPClientBuilder {
            builder: Client::builder(),
        }
    }
}

impl Deref for XMPPClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        self.client.as_ref()
    }
}

pub struct XMPPClientBuilder {
    builder: ClientBuilder,
}

impl XMPPClientBuilder {
    pub fn set_connector_provider(mut self, connector_provider: ConnectorProvider) -> Self {
        self.builder = self.builder.set_connector_provider(connector_provider);
        self
    }

    pub fn build(self) -> XMPPClient {
        let client = self.builder.build();

        XMPPClient {
            client: Arc::new(client),
        }
    }
}
