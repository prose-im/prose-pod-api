// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;
use prose_xmpp::{
    ns,
    stanza::{vcard4::Fn_, VCard4},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub icon: Option<String>,
    pub accent_color: Option<String>,
}

#[derive(Debug, thiserror::Error)]
#[error("Workspace not initialized.")]
pub struct WorkspaceNotInitialized;

const ACCENT_COLOR_EXTENSION_KEY: &'static str = "x-accent-color";

impl TryFrom<VCard4> for Workspace {
    type Error = WorkspaceNotInitialized;

    fn try_from(vcard: VCard4) -> Result<Self, Self::Error> {
        let Some(name) = vcard.fn_.first() else {
            return Err(WorkspaceNotInitialized);
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

impl Into<VCard4> for Workspace {
    fn into(self) -> VCard4 {
        VCard4 {
            fn_: vec![Fn_ { value: self.name }],
            unknown_properties: vec![self.accent_color.map(|c| (ACCENT_COLOR_EXTENSION_KEY, c))]
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
