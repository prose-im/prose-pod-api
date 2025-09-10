// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use minidom::Element;
use prose_xmpp::{
    ns,
    stanza::{vcard4, VCard4},
};
use serdev::Serialize;

use crate::{
    models::{AvatarOwned, Color},
    workspace::errors::WorkspaceNotInitialized,
};

#[derive(Debug, PartialEq, Eq)]
#[derive(Serialize)]
pub struct Workspace {
    pub name: String,
    pub icon: Option<AvatarOwned>,
    pub accent_color: Option<Color>,
}

const ACCENT_COLOR_EXTENSION_KEY: &'static str = "x-accent-color";

impl TryFrom<VCard4> for Workspace {
    type Error = WorkspaceNotInitialized;

    fn try_from(vcard: VCard4) -> Result<Self, Self::Error> {
        let Some(name) = vcard.fn_.first() else {
            return Err(WorkspaceNotInitialized::WithReason("Missing name."));
        };
        Ok(Self {
            name: name.value.to_owned(),
            // Avatars are not stored in vCards.
            icon: None,
            accent_color: vcard
                .unknown_properties
                .get(ACCENT_COLOR_EXTENSION_KEY)
                .first()
                .map(|v| {
                    Color::from_str(&v.text())
                        .inspect_err(|err| {
                            tracing::warn!("Invalid accent color stored in workspace vCard: {err}")
                        })
                        .ok()
                })
                .flatten(),
        })
    }
}

impl From<&Workspace> for VCard4 {
    fn from(
        Workspace {
            name, accent_color, ..
        }: &Workspace,
    ) -> Self {
        // NOTE: When updating this function, also update `WorkspaceService::migrate_workspace_vcard`.
        Self {
            fn_: vec![vcard4::Fn_ {
                value: name.clone(),
            }],
            // See [RFC 6473: vCard KIND:application](https://www.rfc-editor.org/rfc/rfc6473.html).
            kind: Some(vcard4::Kind::Application),
            unknown_properties: vec![accent_color
                .as_ref()
                .map(|c| (ACCENT_COLOR_EXTENSION_KEY, c.to_string()))]
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
