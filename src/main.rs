// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate rocket;

use prose_pod_api::custom_rocket;
use service::config::Config;
use service::dependencies::{Notifier, UUIDStanzaIdProvider};
use service::prosody::{ProsodyAdminRest, ProsodyRest};
use service::xmpp::{LiveXmppService, StanzaSender, UserAccountService};
use service::{JWTService, ServerCtl, XmppServiceInner};
use std::sync::{Arc, Mutex};

#[launch]
fn rocket() -> _ {
    env_logger::init();

    let config = Config::figment();
    #[cfg(debug_assertions)]
    dbg!(&config);
    let jwt_service = match JWTService::from_env() {
        Ok(service) => service,
        Err(err) => panic!("{err}"),
    };
    let server_ctl = ServerCtl::new(Arc::new(Mutex::new(ProsodyAdminRest::from_config(&config))));
    let stanza_sender = StanzaSender::from(ProsodyRest::from_config(&config));
    let stanza_id_provider = Box::new(UUIDStanzaIdProvider);
    let user_account_service = UserAccountService {
        stanza_sender: stanza_sender.clone(),
        stanza_id_provider: stanza_id_provider.clone(),
    };
    let xmpp_service = XmppServiceInner::new(Arc::new(Mutex::new(LiveXmppService {
        stanza_sender,
        stanza_id_provider,
        user_account_service,
    })));
    let notifier = Notifier::from_config(&config).unwrap_or_else(|e| panic!("{e}"));

    todo!("Manage AuthService");

    custom_rocket(rocket::build(), config, server_ctl, xmpp_service)
        .manage(jwt_service)
        .manage(notifier)
}
