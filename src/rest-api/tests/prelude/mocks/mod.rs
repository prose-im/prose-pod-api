// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod prelude {
    pub use std::{
        collections::{HashMap, HashSet},
        str::FromStr as _,
        sync::{Arc, LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
    };

    pub use anyhow::Context as _;
    pub use async_trait::async_trait;
    pub use linked_hash_map::LinkedHashMap;
    pub use service::{
        auth::{util::random_secret, AuthToken, Password},
        errors::*,
        models::DatabaseRwConnectionPools,
        models::EmailAddress,
        prosody::{AsProsody as _, ProsodyRoleName},
        util::either::*,
        xmpp::jid::*,
        AppConfig,
    };
    pub use time::{Duration, OffsetDateTime};

    #[allow(unused)]
    pub use crate::util::{jid_missing, user_missing, USER_MISSING};

    pub(crate) use super::mock_auth_service::check_admin;
    pub use super::mocks::*;
}

mod mock_auth_service;
mod mock_identity_provider;
mod mock_invitation_repository;
mod mock_invitation_service;
mod mock_licensing_service;
mod mock_network_checker;
mod mock_notifier;
mod mock_pod_version_service;
mod mock_server_service;
mod mock_user_repository;
mod mock_user_service;
mod mock_workspace_service;
mod mock_xmpp_service;

mod mocks {
    pub use super::mock_auth_service::*;
    pub use super::mock_identity_provider::*;
    pub use super::mock_invitation_repository::*;
    pub use super::mock_invitation_service::*;
    pub use super::mock_licensing_service::*;
    pub use super::mock_network_checker::*;
    pub use super::mock_notifier::*;
    pub use super::mock_pod_version_service::*;
    pub use super::mock_server_service::*;
    pub use super::mock_user_repository::*;
    pub use super::mock_user_service::*;
    pub use super::mock_workspace_service::*;
    pub use super::mock_xmpp_service::*;
}

pub use self::mocks::*;
