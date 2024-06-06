// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use entity::model::JID;
use log::debug;
use minidom::Element;
use reqwest::Client;
use tokio::runtime::Handle;
use xmpp_parsers::iq::Iq;

use crate::{
    config::Config,
    xmpp::stanza_sender::{Error, StanzaSenderError, StanzaSenderInner},
};

/// Rust interface to [`mod_http_rest`](https://hg.prosody.im/prosody-modules/file/tip/mod_http_rest).
#[derive(Debug, Clone)]
pub struct ProsodyRest {
    rest_api_url: String,
    api_auth_username: JID,
    api_auth_password: String,
}

impl ProsodyRest {
    pub fn from_config(config: &Config) -> Self {
        Self {
            rest_api_url: config.server.rest_api_url(),
            api_auth_username: config.api_jid(),
            api_auth_password: config.api.admin_password.to_owned().unwrap(),
        }
    }
}

impl StanzaSenderInner for ProsodyRest {
    fn send_iq(&self, iq: Iq) -> Result<Option<Element>, StanzaSenderError> {
        let client = Client::new();
        let element: Element = iq.into();
        let request = client
            .post(self.rest_api_url.to_owned())
            .header("Content-Type", "application/xmpp+xml")
            .body(String::from(&element))
            .basic_auth(
                self.api_auth_username.node.to_string(),
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
                        "Prosody REST API call failed.\n  Status: {}\n  Headers: {:?}\n  Body: {}",
                        response.status(),
                        response.headers().clone(),
                        response.text().await.unwrap_or("<nil>".to_string())
                    )))
                }
            })
        })
        .and_then(|res| {
            tokio::task::block_in_place(move || Handle::current().block_on(res.text()))
                .map_err(Error::from)
        })
        .map(|string| Element::from_str(string.as_str()).ok())
    }
    // fn get_vcard(&self, jid: &JID, image_id: &Sha1HexAttribute) -> Result<Option<VCard>, Error> {
    //     let iq = Iq {
    //         from: None,
    //         to: Some(jid.into()),
    //         id: self.ctx.generate_id(),
    //         payload: IqType::Get(
    //             PubSub::Items(Items {
    //                 max_items: Some(1),
    //                 node: NodeName(ns::AVATAR_DATA.to_string()),
    //                 subid: None,
    //                 items: vec![
    //                     pubsub::pubsub::Item(xmpp_parsers::pubsub::Item {
    //                         id: Some(pubsub::ItemId(image_id.to_hex())),
    //                         publisher: None,
    //                         payload: None,
    //                     }),
    //                 ],
    //             })
    //             .into(),
    //         ),
    //     };

    //     let element: Element = iq.into();
    //     let request_body = String::from(element);

    //     self.call(|client| {
    //         client
    //             .get(self.rest())
    //             .header("Content-Type", "text/xml")
    //             .body(element)
    //     })
    //     .and_then(|res| {
    //         tokio::task::block_in_place(move || Handle::current().block_on(res.text()))
    //             .map_err(Error::from)
    //     })
    //     .and_then(|vcard| {
    //         todo!()
    //         // let response = Element::from_str(&vcard).unwrap();

    //         // let PubSub::Items(mut items) = PubSub::try_from(response)? else {
    //         //     return Err(RequestError::UnexpectedResponse.into());
    //         // };

    //         // if items.items.is_empty() {
    //         //     return Ok(None);
    //         // }

    //         // let Some(payload) = items.items.swap_remove(0).payload.take() else {
    //         //     return Ok(None);
    //         // };

    //         // Ok(Some(AvatarData::Base64(payload.text())))
    //     })
    // }
    // fn set_vcard(&self, jid: &JID, vcard: &VCard) -> Result<(), Error> {
    //     self.call(|client| {
    //         client
    //             .put(format!(
    //                 "{}/{}",
    //                 self.admin_rest("vcards"),
    //                 urlencoding::encode(&jid.to_string())
    //             ))
    //             .body(vcard.export())
    //     })
    //     .map(|_| ())
    // }
}
