// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::JID;
use minidom::Element;
use xmpp_parsers::hashes::Sha1HexAttribute;
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::pubsub::pubsub::Items;
use xmpp_parsers::pubsub::{self, NodeName, PubSub};

use crate::dependencies::StanzaIdProvider;
use crate::{into_jid, VCard, XmppServiceError};

use super::stanza::avatar::AvatarData;
use super::stanza::{avatar, ns};
use super::stanza_sender::StanzaSender;
use super::xmpp_service::{XmppServiceContext, XmppServiceImpl, R};

pub struct LiveXmppService {
    pub stanza_sender: StanzaSender,
    pub stanza_id_provider: Box<dyn StanzaIdProvider>,
}

impl XmppServiceImpl for LiveXmppService {
    fn get_vcard(&self, ctx: &XmppServiceContext, jid: &JID) -> R<Option<VCard>> {
        let iq = Iq {
            from: Some(into_jid(&ctx.bare_jid)),
            to: Some(into_jid(jid)),
            id: self.stanza_id_provider.new_id(),
            payload: IqType::Get(Element::builder("vcard", ns::VCARD4).build()),
        };

        let response = self
            .stanza_sender
            .send_iq(iq)?
            .ok_or(XmppServiceError::UnexpectedResponse)?;

        let vcard =
            VCard::try_from(response).map_err(|e| XmppServiceError::Other(format!("{e}")))?;

        Ok(Some(vcard))
    }
    fn set_vcard(&self, ctx: &XmppServiceContext, jid: &JID, vcard: &VCard) -> R<()> {
        let mut iq = Iq::from_set(self.stanza_id_provider.new_id(), vcard.clone());
        iq.from = Some(into_jid(&ctx.bare_jid));
        iq.to = Some(into_jid(jid));
        self.stanza_sender.send_iq(iq)?;
        Ok(())
    }

    fn get_avatar(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
        image_id: &Sha1HexAttribute,
    ) -> R<Option<AvatarData>> {
        let iq = Iq {
            from: Some(into_jid(&ctx.bare_jid)),
            to: Some(into_jid(jid)),
            id: self.stanza_id_provider.new_id(),
            payload: IqType::Get(
                PubSub::Items(Items {
                    max_items: Some(1),
                    node: NodeName(ns::AVATAR_DATA.to_string()),
                    subid: None,
                    items: vec![
                        pubsub::pubsub::Item(xmpp_parsers::pubsub::Item {
                            id: Some(pubsub::ItemId(image_id.to_hex())),
                            publisher: None,
                            payload: None,
                        }),
                    ],
                })
                .into(),
            ),
        };

        let response = self
            .stanza_sender
            .send_iq(iq)?
            .ok_or(XmppServiceError::UnexpectedResponse)?;

        let PubSub::Items(items) =
            PubSub::try_from(response).map_err(|e| XmppServiceError::Other(format!("{e}")))?
        else {
            return Err(XmppServiceError::UnexpectedResponse);
        };

        let avatar = items
            .items
            .first()
            .and_then(|item| item.payload.to_owned())
            .map(|payload| AvatarData::Base64(payload.text()));

        Ok(avatar)
    }
    fn set_avatar(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
        checksum: &avatar::ImageId,
        base64_image_data: String,
    ) -> R<()> {
        let mut iq = Iq::from_set(
            self.stanza_id_provider.new_id(),
            pubsub::PubSub::Publish {
                publish: pubsub::pubsub::Publish {
                    node: NodeName(ns::AVATAR_DATA.to_string()),
                    items: vec![
                        pubsub::pubsub::Item(pubsub::Item {
                            id: Some(pubsub::ItemId(checksum.to_string())),
                            publisher: None,
                            payload: Some(
                                Element::builder("data", ns::AVATAR_DATA)
                                    .append(base64_image_data.as_str())
                                    .build(),
                            ),
                        }),
                    ],
                },
                publish_options: None,
            },
        );
        iq.from = Some(into_jid(&ctx.bare_jid));
        iq.to = Some(into_jid(jid));
        self.stanza_sender.send_iq(iq)?;
        Ok(())
    }
}
