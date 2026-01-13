// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod prelude {
    pub use std::sync::Arc;

    pub use async_trait::async_trait;
    pub use secrecy::SecretString;

    pub use crate::{
        auth::{
            errors::{InvalidCredentials, PasswordResetTokenExpired, PasswordValidationError},
            AuthToken,
        },
        errors::{Forbidden, Unauthorized},
        invitations::InvitationContact,
        models::jid::{BareJid, NodeRef},
        util::either::{Either, Either3, Either4},
    };

    pub use super::{BackupId, BackupMetadata, BackupService, BackupServiceImpl};
}

use anyhow::bail;
use serdev::Serialize;
use time::OffsetDateTime;

use crate::{
    app_config::{BackupBackend, BackupsConfig},
    backups::{backup_repository::S3BackupRepository, BackupRepository},
    errors::MissingConfiguration,
};

use self::prelude::*;

#[derive(Debug, Clone)]
pub struct BackupService {
    pub implem: Arc<dyn BackupServiceImpl>,
}

impl BackupService {
    pub fn from_config(backups_config: &BackupsConfig) -> Result<Self, MissingConfiguration> {
        let repository = BackupRepository::from_config(backups_config)?;

        Ok(Self {
            implem: Arc::new(LiveBackupService { repository }),
        })
    }
}

#[async_trait::async_trait]
pub trait BackupServiceImpl: std::fmt::Debug + Sync + Send {
    async fn create_backup(
        &self,
        auth: &AuthToken,
    ) -> Result<BackupMetadata, Either3<Unauthorized, Forbidden, anyhow::Error>>;

    async fn get_backup(
        &self,
        backup_id: BackupId,
        auth: &AuthToken,
    ) -> Result<Option<BackupMetadata>, Either3<Unauthorized, Forbidden, anyhow::Error>>;

    async fn list_backups(
        &self,
        auth: &AuthToken,
    ) -> Result<Vec<BackupMetadata>, Either3<Unauthorized, Forbidden, anyhow::Error>>;

    async fn delete_backup(
        &self,
        backup_id: BackupId,
        auth: &AuthToken,
    ) -> Result<(), Either3<Unauthorized, Forbidden, anyhow::Error>>;
}

pub type BackupId = String;

#[derive(Debug, Clone)]
#[derive(Serialize)]
pub struct BackupMetadata {
    pub backup_id: BackupId,
    pub created_at: Option<OffsetDateTime>,
    pub size_bytes: Option<u64>,
    pub checksum: Option<String>,
}

use self::live::*;
mod live {
    use anyhow::Context as _;
    use async_trait::async_trait;
    use bytes::Bytes;

    use crate::{
        auth::{AuthService, AuthToken},
        backups::BackupRepository,
        util::either::to_either3_1_3,
    };

    use super::*;

    #[derive(Debug)]
    pub struct LiveBackupService {
        pub repository: BackupRepository,
        pub auth_service: AuthService,
    }

    impl LiveBackupService {
        fn object_key(backup_id: &str) -> String {
            format!("{}{}.bin", PREFIX, backup_id)
        }
    }

    #[async_trait]
    impl BackupServiceImpl for LiveBackupService {
        #[tracing::instrument(level = "trace", skip_all)]
        async fn create_backup(
            &self,
            auth: &AuthToken,
        ) -> Result<BackupMetadata, Either3<Unauthorized, Forbidden, anyhow::Error>> {
            let caller = self
                .auth_service
                .get_user_info(auth)
                .await
                .map_err(to_either3_1_3)?;

            if !caller.is_admin() {
                return Err(Either3::E2(Forbidden(format!(
                    "{} is not an admin.",
                    caller.jid
                ))));
            }

            let backup_id = uuid::Uuid::new_v4().to_string();
            let key = Self::object_key(&backup_id);

            // TODO: Replace this with real backup data
            let data = ByteStream::from_static(b"backup content");

            todo!();
            let data = Bytes::new();

            self.repository.create_backup(data).await?;

            Ok(BackupMetadata {
                backup_id,
                size_bytes,
                created_at,
                checksum: etag,
            })
        }

        #[tracing::instrument(level = "trace", skip_all)]
        async fn get_backup(
            &self,
            backup_id: BackupId,
            auth: &AuthToken,
        ) -> Result<Option<BackupMetadata>, Either3<Unauthorized, Forbidden, anyhow::Error>>
        {
            let caller = self
                .auth_service
                .get_user_info(auth)
                .await
                .map_err(to_either3_1_3)?;

            if !caller.is_admin() {
                return Err(Either3::E2(Forbidden(format!(
                    "{} is not an admin.",
                    caller.jid
                ))));
            }

            let key = Self::object_key(&backup_id);

            let result = self
                .client
                .head_object()
                .bucket(&self.bucket)
                .key(&key)
                .send()
                .await;

            match result {
                Ok(head) => {
                    let created_at = head
                        .last_modified()
                        .map(|lm| OffsetDateTime::from_unix_timestamp(lm.secs()))
                        .transpose()
                        .context("invalid timestamp")?
                        .unwrap_or_else(OffsetDateTime::now_utc);

                    Ok(Some(BackupMetadata {
                        backup_id,
                        size_bytes: head.content_length() as u64,
                        created_at,
                        checksum: head.e_tag().unwrap_or("").trim_matches('"').to_string(),
                    }))
                }
                Err(err) => {
                    // If it is 404
                    if matches!(err, SdkError::ServiceError { err, .. } if err.is_not_found()) {
                        return Ok(None);
                    }
                    Err(err).context("failed to head backup object")
                }
            }
        }

        #[tracing::instrument(level = "trace", skip_all)]
        async fn list_backups(
            &self,
            auth: &AuthToken,
        ) -> Result<Vec<BackupMetadata>, Either3<Unauthorized, Forbidden, anyhow::Error>> {
            let caller = self
                .auth_service
                .get_user_info(auth)
                .await
                .map_err(to_either3_1_3)?;

            if !caller.is_admin() {
                return Err(Either3::E2(Forbidden(format!(
                    "{} is not an admin.",
                    caller.jid
                ))));
            }

            todo!()
        }

        #[tracing::instrument(level = "trace", skip_all, fields(backup_id))]
        async fn delete_backup(
            &self,
            backup_id: BackupId,
            auth: &AuthToken,
        ) -> Result<(), Either3<Unauthorized, Forbidden, anyhow::Error>> {
            let caller = self
                .auth_service
                .get_user_info(auth)
                .await
                .map_err(to_either3_1_3)?;

            if !caller.is_admin() {
                return Err(Either3::E2(Forbidden(format!(
                    "{} is not an admin.",
                    caller.jid
                ))));
            }

            todo!()
        }
    }
}

// MARK: - Boilerplate

impl std::ops::Deref for BackupService {
    type Target = Arc<dyn BackupServiceImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}
