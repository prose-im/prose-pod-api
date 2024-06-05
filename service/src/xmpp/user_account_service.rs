// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use entity::model::JID;
use minidom::Element;
use xmpp_parsers::{
    avatar,
    hashes::Sha1HexAttribute,
    iq::Iq,
    ns,
    pubsub::{self, NodeName},
};

use crate::{
    avatar_metadata::AvatarMetadata, dependencies::StanzaIdProvider, into_jid, XmppServiceContext,
};

use super::{stanza_sender::StanzaSenderError, StanzaSender};

/// Inspired by <https://github.com/prose-im/prose-core-client/blob/adae6b5a5ec6ca550c2402a75b57e17ef50583f9/crates/prose-core-client/src/domain/account/services/user_account_service.rs>.
pub struct UserAccountService {
    pub stanza_sender: StanzaSender,
    pub stanza_id_provider: Box<dyn StanzaIdProvider>,
}

#[derive(Debug, thiserror::Error)]
pub enum SetAvatarMetadataError {
    #[error("Could not send stanza: {0}")]
    StanzaSendFailure(#[from] StanzaSenderError),
    // NOTE: For some reason Rust doen't like when we add `#[from]` in this case…
    #[error("Could not generate payload ID: {0}")]
    PayloadIdGenerationFailure(<Sha1HexAttribute as FromStr>::Err),
}

impl UserAccountService {
    pub fn set_avatar_metadata(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
        metadata: &AvatarMetadata,
    ) -> Result<(), SetAvatarMetadataError> {
        let mut iq = Iq::from_set(
            self.stanza_id_provider.new_id(),
            pubsub::PubSub::Publish {
                publish: pubsub::pubsub::Publish {
                    node: NodeName(ns::AVATAR_METADATA.to_string()),
                    items: vec![
                        pubsub::pubsub::Item(pubsub::Item {
                            id: Some(pubsub::ItemId(metadata.checksum.to_string())),
                            publisher: None,
                            payload: Some(
                                avatar::Metadata {
                                    infos: vec![avatar::Info {
                                        bytes: metadata.bytes as u32,
                                        width: metadata.width,
                                        height: metadata.height,
                                        id: Sha1HexAttribute::from_str(metadata.checksum.as_ref())
                                            .map_err(
                                                SetAvatarMetadataError::PayloadIdGenerationFailure,
                                            )?,
                                        type_: metadata.mime_type.to_owned(),
                                        url: None,
                                    }],
                                }
                                .into(),
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
    pub fn set_avatar_image(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
        checksum: &super::stanza::avatar::ImageId,
        base64_image_data: String,
    ) -> Result<(), StanzaSenderError> {
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
                                    .append(base64_image_data)
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
