// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use log::{debug, trace};
use prose_xmpp::mods::{self, AvatarData};
use prose_xmpp::stanza::avatar::{self, ImageId};
use prose_xmpp::{BareJid, IDProvider};
use reqwest::Client as HttpClient;
use tokio::runtime::Handle;
use xmpp_parsers::hashes::Sha1HexAttribute;
use xmpp_parsers::jid::ResourcePart;

use crate::config::Config;
use crate::prosody::ProsodyRest;
use crate::services::xmpp_service::{VCard, XmppServiceContext, XmppServiceError, XmppServiceImpl};

use super::non_standard_xmpp_client::NonStandardXmppClient;
use super::xmpp_client::XMPPClient;

pub struct LiveXmppService {
    http_client: HttpClient,
    pub rest_api_url: String,
    pub non_standard_xmpp_client: Box<dyn NonStandardXmppClient + Send + Sync>,
    id_provider: Box<dyn IDProvider>,
}

impl LiveXmppService {
    pub fn from_config(
        config: &Config,
        http_client: HttpClient,
        non_standard_xmpp_client: Box<dyn NonStandardXmppClient + Send + Sync>,
        id_provider: Box<dyn IDProvider>,
    ) -> Self {
        Self {
            http_client,
            rest_api_url: config.server.rest_api_url(),
            non_standard_xmpp_client,
            id_provider,
        }
    }

    async fn xmpp_client(&self, ctx: &XmppServiceContext) -> Result<XMPPClient, XmppServiceError> {
        let http_client = self.http_client.clone();
        let rest_api_url = self.rest_api_url.clone();
        let xmpp_client = XMPPClient::builder()
            .set_connector_provider(ProsodyRest::provider(http_client, rest_api_url))
            .build();
        xmpp_client
            .connect(
                &ctx.bare_jid
                    .with_resource(&ResourcePart::new(&self.id_provider.new_id()).unwrap()),
                ctx.prosody_token.clone(),
            )
            .await
            .map_err(XmppServiceError::from)?;
        Ok(xmpp_client)
    }
    pub fn load_latest_avatar_metadata(
        &self,
        from: &BareJid,
        ctx: &XmppServiceContext,
    ) -> Result<Option<avatar::Info>, XmppServiceError> {
        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let xmpp_client = self.xmpp_client(ctx).await?;
                let profile = xmpp_client.get_mod::<mods::Profile>();
                profile
                    .load_latest_avatar_metadata(from)
                    .await
                    .map_err(Into::into)
            })
        })
    }
}

impl XmppServiceImpl for LiveXmppService {
    fn get_vcard(
        &self,
        ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<Option<VCard>, XmppServiceError> {
        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let xmpp_client = self.xmpp_client(ctx).await?;
                let profile = xmpp_client.get_mod::<mods::Profile>();
                profile.load_vcard(jid.to_owned()).await.map_err(Into::into)
            })
        })
    }
    fn set_own_vcard(
        &self,
        ctx: &XmppServiceContext,
        vcard: &VCard,
    ) -> Result<(), XmppServiceError> {
        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let xmpp_client = self.xmpp_client(ctx).await?;
                let profile = xmpp_client.get_mod::<mods::Profile>();

                trace!("Setting {}'s vCard…", ctx.bare_jid);
                profile.set_vcard(vcard.to_owned()).await?;
                debug!("Set {}'s vCard", ctx.bare_jid);

                trace!("Publishing {}'s vCard…", ctx.bare_jid);
                profile.publish_vcard(vcard.to_owned()).await?;
                debug!("Published {}'s vCard", ctx.bare_jid);

                Ok(())
            })
        })
    }

    fn get_avatar(
        &self,
        ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<Option<AvatarData>, XmppServiceError> {
        let Some(avatar_metadata) = self.load_latest_avatar_metadata(jid, ctx)? else {
            return Ok(None);
        };
        let image_id = avatar_metadata.id;

        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let xmpp_client = self.xmpp_client(ctx).await?;
                let profile = xmpp_client.get_mod::<mods::Profile>();
                profile
                    .load_avatar_image(
                        jid.to_owned(),
                        &Sha1HexAttribute::from_str(&image_id.as_ref()).unwrap(),
                    )
                    .await
                    .map_err(Into::into)
            })
        })
    }
    /// Inspired by <https://github.com/prose-im/prose-core-client/blob/adae6b5a5ec6ca550c2402a75b57e17ef50583f9/crates/prose-core-client/src/app/services/account_service.rs#L116-L157>.
    fn set_own_avatar(
        &self,
        ctx: &XmppServiceContext,
        png_data: Vec<u8>,
    ) -> Result<(), XmppServiceError> {
        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let xmpp_client = self.xmpp_client(ctx).await?;
                let profile = xmpp_client.get_mod::<mods::Profile>();

                let image_data_len = png_data.len();
                let image_data = AvatarData::Data(png_data);
                let checksum: ImageId = image_data
                    .generate_sha1_checksum()
                    .map_err(|err| {
                        XmppServiceError::Other(format!(
                            "Could not generate avatar checksum: {err}"
                        ))
                    })?
                    .as_ref()
                    .into();

                debug!("Uploading avatar…");
                profile
                    .set_avatar_image(&checksum, image_data.base64())
                    .await
                    .map_err(|err| {
                        XmppServiceError::Other(format!("Could not upload avatar: {err}"))
                    })?;

                debug!("Uploading avatar metadata…");
                profile
                    // TODO: Allow specifying width and height
                    // TODO: Support other MIME types
                    .set_avatar_metadata(image_data_len, &checksum, "image/png", None, None)
                    .await
                    .map_err(|err| {
                        XmppServiceError::Other(format!("Could not upload avatar metadata: {err}"))
                    })?;

                Ok(())
            })
        })
    }

    fn is_connected(
        &self,
        _ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<bool, XmppServiceError> {
        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                self.non_standard_xmpp_client
                    .is_connected(jid)
                    .await
                    .map_err(XmppServiceError::from)
            })
        })
    }
}
