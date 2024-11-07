// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{str::FromStr as _, sync::Arc};

use prose_xmpp::{
    mods::{self, AvatarData},
    stanza::avatar::{self, ImageId},
    BareJid, IDProvider,
};
use reqwest::Client as HttpClient;
use tracing::{debug, trace};
use xmpp_parsers::hashes::Sha1HexAttribute;

use crate::{
    models::jid::ResourcePart,
    prosody::ProsodyRest,
    xmpp::{VCard, XmppServiceContext, XmppServiceError, XmppServiceImpl},
    AppConfig,
};

use super::{non_standard_xmpp_client::NonStandardXmppClient, xmpp_client::XMPPClient};

#[derive(Clone)]
pub struct LiveXmppService {
    http_client: HttpClient,
    pub rest_api_url: String,
    pub non_standard_xmpp_client: Arc<dyn NonStandardXmppClient>,
    id_provider: Arc<dyn IDProvider>,
}

// TODO: Make `IDProvider` implement `Debug`
impl std::fmt::Debug for LiveXmppService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveXmppService")
            .field("http_client", &self.http_client)
            .field("rest_api_url", &self.rest_api_url)
            .field("non_standard_xmpp_client", &self.non_standard_xmpp_client)
            .field(
                "id_provider",
                &std::any::type_name_of_val(self.id_provider.as_ref()),
            )
            .finish()
    }
}

impl LiveXmppService {
    pub fn from_config(
        config: &AppConfig,
        http_client: HttpClient,
        non_standard_xmpp_client: Arc<dyn NonStandardXmppClient>,
        id_provider: Arc<dyn IDProvider>,
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
    pub async fn load_latest_avatar_metadata(
        &self,
        from: &BareJid,
        ctx: &XmppServiceContext,
    ) -> Result<Option<avatar::Info>, XmppServiceError> {
        let xmpp_client = self.xmpp_client(ctx).await?;
        let profile = xmpp_client.get_mod::<mods::Profile>();
        profile
            .load_latest_avatar_metadata(from)
            .await
            .map_err(Into::into)
    }
}

#[async_trait::async_trait]
impl XmppServiceImpl for LiveXmppService {
    async fn get_vcard(
        &self,
        ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<Option<VCard>, XmppServiceError> {
        let xmpp_client = self.xmpp_client(ctx).await?;
        let profile = xmpp_client.get_mod::<mods::Profile>();
        profile
            .load_vcard4(jid.to_owned())
            .await
            .map_err(Into::into)
    }
    async fn set_own_vcard(
        &self,
        ctx: &XmppServiceContext,
        vcard: &VCard,
    ) -> Result<(), XmppServiceError> {
        let xmpp_client = self.xmpp_client(ctx).await?;
        let profile = xmpp_client.get_mod::<mods::Profile>();

        trace!("Publishing {}'s vCard4…", ctx.bare_jid);
        profile.publish_vcard4(vcard.to_owned(), None).await?;
        debug!("Published {}'s vCard4", ctx.bare_jid);

        Ok(())
    }

    async fn get_avatar(
        &self,
        ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<Option<AvatarData>, XmppServiceError> {
        let Some(avatar_metadata) = self.load_latest_avatar_metadata(jid, ctx).await? else {
            return Ok(None);
        };
        let image_id = avatar_metadata.id;

        let xmpp_client = self.xmpp_client(ctx).await?;
        let profile = xmpp_client.get_mod::<mods::Profile>();
        profile
            .load_avatar_image(
                jid.to_owned(),
                &Sha1HexAttribute::from_str(&image_id.as_ref()).unwrap(),
            )
            .await
            .map_err(Into::into)
    }
    /// Inspired by <https://github.com/prose-im/prose-core-client/blob/adae6b5a5ec6ca550c2402a75b57e17ef50583f9/crates/prose-core-client/src/app/services/account_service.rs#L116-L157>.
    async fn set_own_avatar(
        &self,
        ctx: &XmppServiceContext,
        png_data: Vec<u8>,
    ) -> Result<(), XmppServiceError> {
        let xmpp_client = self.xmpp_client(ctx).await?;
        let profile = xmpp_client.get_mod::<mods::Profile>();

        let image_data_len = png_data.len();
        let image_data = AvatarData::Data(png_data.into_boxed_slice());
        let checksum: ImageId = image_data
            .generate_sha1_checksum()
            .map_err(|err| {
                XmppServiceError::Other(format!("Could not generate avatar checksum: {err}"))
            })?
            .as_ref()
            .into();

        debug!("Uploading avatar…");
        profile
            .set_avatar_image(&checksum, image_data.base64())
            .await
            .map_err(|err| XmppServiceError::Other(format!("Could not upload avatar: {err}")))?;

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
    }

    async fn is_connected(
        &self,
        _ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<bool, XmppServiceError> {
        self.non_standard_xmpp_client
            .is_connected(jid)
            .await
            .map_err(XmppServiceError::from)
    }
}
