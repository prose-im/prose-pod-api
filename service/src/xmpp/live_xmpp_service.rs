// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::JID;
use minidom::Element;
use xmpp_parsers::iq::{Iq, IqType};

use crate::dependencies::StanzaIdProvider;
use crate::{into_jid, VCard};

use super::stanza::ns;
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

        let res = self.stanza_sender.send_iq(iq)?;

        todo!()
    }
    fn set_vcard(&self, ctx: &XmppServiceContext, jid: &JID, vcard: &VCard) -> R<()> {
        let iq = Iq {
            from: Some(into_jid(&ctx.bare_jid)),
            to: Some(into_jid(jid)),
            id: self.stanza_id_provider.new_id(),
            payload: IqType::Get(Element::builder("vcard", ns::VCARD4).build()),
        };

        self.stanza_sender.send_iq(iq)?;

        todo!()
    }
}
