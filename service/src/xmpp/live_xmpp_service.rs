// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::JID;
use log::debug;
use minidom::Element;
use xmpp_parsers::avatar;
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::pubsub::pubsub::Items;
use xmpp_parsers::pubsub::{self, NodeName, PubSub};

use crate::avatar_metadata::AvatarMetadata;
use crate::dependencies::StanzaIdProvider;
use crate::{into_jid, VCard, XmppServiceError};

use super::stanza::avatar::AvatarData;
use super::stanza::ns;
use super::stanza_sender::StanzaSender;
use super::user_account_service::UserAccountService;
use super::util::{PubSubItemsExt, PubSubQuery};
use super::xmpp_service::{XmppServiceContext, XmppServiceImpl};

pub struct LiveXmppService {
    pub stanza_sender: StanzaSender,
    pub stanza_id_provider: Box<dyn StanzaIdProvider>,
    pub user_account_service: UserAccountService,
}

impl LiveXmppService {
    pub fn load_latest_avatar_metadata(
        &self,
        from: &JID,
    ) -> Result<Option<AvatarMetadata>, XmppServiceError> {
        let metadata = self
            .stanza_sender
            .query_pubsub_node(
                PubSubQuery::new(
                    self.stanza_id_provider.new_id(),
                    xmpp_parsers::ns::AVATAR_METADATA,
                )
                .set_to(from.clone())
                .set_max_items(1),
            )?
            .unwrap_or_default()
            .find_first_payload::<avatar::Metadata>("metadata", xmpp_parsers::ns::AVATAR_METADATA)
            .map_err(|e| XmppServiceError::Other(format!("{e}")))?;

        let Some(mut metadata) = metadata else {
            return Ok(None);
        };

        if metadata.infos.is_empty() {
            return Ok(None);
        }

        let info = metadata.infos.swap_remove(0);

        Ok(Some(info.into()))
    }
}

impl From<avatar::Info> for AvatarMetadata {
    fn from(value: avatar::Info) -> Self {
        AvatarMetadata {
            bytes: value.bytes as usize,
            mime_type: value.type_,
            checksum: value.id.to_base64().into(),
            width: value.width,
            height: value.height,
            url: value.url,
        }
    }
}

impl XmppServiceImpl for LiveXmppService {
    fn get_vcard(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
    ) -> Result<Option<VCard>, XmppServiceError> {
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
    fn set_vcard(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
        vcard: &VCard,
    ) -> Result<(), XmppServiceError> {
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
    ) -> Result<Option<AvatarData>, XmppServiceError> {
        let Some(avatar_metadata) = self.load_latest_avatar_metadata(jid)? else {
            return Ok(None);
        };
        let image_id = avatar_metadata.checksum;

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
                            id: Some(pubsub::ItemId(image_id.to_string())),
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
    /// Inspired by <https://github.com/prose-im/prose-core-client/blob/adae6b5a5ec6ca550c2402a75b57e17ef50583f9/crates/prose-core-client/src/app/services/account_service.rs#L116-L157>.
    fn set_avatar(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
        png_data: String,
    ) -> Result<(), XmppServiceError> {
        let image_data_len = png_data.as_str().len();
        let image_data = AvatarData::Data(png_data.into_bytes());

        // TODO: Allow specifying width and height
        let metadata = AvatarMetadata {
            bytes: image_data_len,
            mime_type: "image/png".to_string(),
            checksum: image_data
                .generate_sha1_checksum()
                .map_err(|err| {
                    XmppServiceError::Other(format!("Could not generate avatar checksum: {err}"))
                })?
                .as_ref()
                .into(),
            width: None,
            height: None,
            url: None,
        };

        debug!("Uploading avatar…");
        self.user_account_service
            .set_avatar_image(
                ctx,
                jid,
                &metadata.checksum,
                image_data.base64().to_string(),
            )
            .map_err(|err| XmppServiceError::Other(format!("Could not upload avatar: {err}")))?;

        debug!("Uploading avatar metadata…");
        self.user_account_service
            .set_avatar_metadata(ctx, jid, &metadata)
            .map_err(|err| {
                XmppServiceError::Other(format!("Could not upload avatar metadata: {err}"))
            })?;

        Ok(())
    }
    fn disable_avatar(&self, ctx: &XmppServiceContext, jid: &JID) -> Result<(), XmppServiceError> {
        todo!()
    }
}
