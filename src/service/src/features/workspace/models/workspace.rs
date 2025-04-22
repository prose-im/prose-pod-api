// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;
use prose_xmpp::{
    ns,
    stanza::{vcard4, VCard4},
};
use serde::{Deserialize, Serialize};

use crate::xmpp::xmpp_service::Avatar;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub icon: Option<Avatar>,
    pub accent_color: Option<String>,
}

#[derive(Debug, thiserror::Error)]
#[error("Workspace name not initialized.")]
pub struct WorkspaceNameNotInitialized;

const ACCENT_COLOR_EXTENSION_KEY: &'static str = "x-accent-color";

impl TryFrom<VCard4> for Workspace {
    type Error = WorkspaceNameNotInitialized;

    fn try_from(vcard: VCard4) -> Result<Self, Self::Error> {
        let Some(name) = vcard.fn_.first() else {
            return Err(WorkspaceNameNotInitialized);
        };
        Ok(Self {
            name: name.value.to_owned(),
            // Avatars are not stored in vCards.
            icon: None,
            accent_color: vcard
                .unknown_properties
                .get(ACCENT_COLOR_EXTENSION_KEY)
                .first()
                .map(|v| v.text()),
        })
    }
}

impl From<Workspace> for VCard4 {
    fn from(
        Workspace {
            name, accent_color, ..
        }: Workspace,
    ) -> Self {
        // NOTE: When updating this function, also update `WorkspaceService::migrate_workspace_vcard`.
        Self {
            fn_: vec![vcard4::Fn_ { value: name }],
            // See [RFC 6473: vCard KIND:application](https://www.rfc-editor.org/rfc/rfc6473.html).
            kind: Some(vcard4::Kind::Application),
            unknown_properties: vec![accent_color
                .as_ref()
                .map(|c| (ACCENT_COLOR_EXTENSION_KEY, c.as_str()))]
            .into_iter()
            .flatten()
            .map(|(k, v)| {
                Element::builder(k, ns::VCARD4)
                    .append(Element::builder("text", ns::VCARD4).append(v))
                    .build()
            })
            .collect(),
            ..Default::default()
        }
    }
}
