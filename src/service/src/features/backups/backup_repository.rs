// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod prelude {
    pub use async_trait::async_trait;
    pub use bytes::Bytes;

    pub use crate::{
        auth::AuthToken,
        backups::{
            backup_repository::BackupRepositoryImpl,
            backup_service::{BackupId, BackupMetadata},
        },
        errors::{Forbidden, Unauthorized},
        members::{
            errors::MemberNotFound,
            models::{Member, MemberRole, UsersStats},
        },
        util::either::{Either, Either3},
        xmpp::jid::NodeRef,
    };
}

use std::sync::Arc;

use prosody_http::admin_api;

use crate::{
    app_config::{BackupBackend, BackupsConfig},
    errors::MissingConfiguration,
};

use self::prelude::*;

#[derive(Debug, Clone)]
pub struct BackupRepository {
    pub implem: Arc<dyn BackupRepositoryImpl>,
}

impl BackupRepository {
    pub fn from_config(backups_config: &BackupsConfig) -> Result<Self, MissingConfiguration> {
        match backups_config.backend {
            BackupBackend::S3 => {
                let Some(ref s3_config) = backups_config.s3 else {
                    return Err(MissingConfiguration("backups.s3"));
                };

                let repository = S3BackupRepository::new(s3_config);

                Ok(Self {
                    implem: Arc::new(repository),
                })
            }
        }
    }
}

#[async_trait]
pub trait BackupRepositoryImpl: std::fmt::Debug + Sync + Send {
    async fn list_backups(&self) -> Result<Vec<BackupMetadata>, anyhow::Error>;

    async fn get_backup(
        &self,
        backup_id: &BackupId,
    ) -> Result<Option<BackupMetadata>, anyhow::Error>;

    async fn create_backup(&self, backup_data: Bytes) -> Result<BackupMetadata, anyhow::Error>;

    async fn delete_backup(&self, backup_id: &BackupId) -> Result<(), anyhow::Error>;
}

#[derive(Debug)]
pub struct UsersStats {
    pub count: usize,
}

pub use self::s3::*;
mod s3 {
    use anyhow::{anyhow, Context};
    use aws_sdk_s3::{
        config::{Credentials, Region},
        operation::get_object::GetObjectOutput,
        primitives::ByteStream,
        types::Object,
        Client, Config,
    };
    use time::UtcDateTime;
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};

    use crate::{app_config::S3Config, prose_pod_server_api::ProsePodServerApi};

    use super::*;

    #[derive(Debug)]
    pub struct S3BackupRepository {
        pub client: Client,
        pub bucket: String,
    }

    impl S3BackupRepository {
        pub fn new(config: &S3Config, server_api: ProsePodServerApi) -> Self {
            let todo = "Move all to the Server";

            // 1. Provide your own AWS credentials
            let credentials = Credentials::from_keys(
                "YOUR_ACCESS_KEY_ID",
                "YOUR_SECRET_ACCESS_KEY",
                None, // optional session token
            );

            // 2. Provide your region
            let region = Region::new("us-east-1");

            // 3. Build the AWS SDK config manually
            let s3_config = Config::builder()
                .credentials_provider(credentials)
                .region(region)
                .build();

            // 4. Create the S3 client
            let client = Client::from_conf(s3_config);

            Self {
                client,
                bucket: config.bucket,
                server_api,
            }
        }
    }

    #[async_trait]
    impl BackupRepositoryImpl for S3BackupRepository {
        async fn list_backups(&self) -> Result<Vec<BackupMetadata>, anyhow::Error> {
            let response = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .send()
                .await
                .context("Failed to list backups")?;
            let objects = response.contents();

            // Pre-allocate `objects.len() / 3` as backups can have a checksum
            // and a signature. If there is no signature, then we allocated too
            // few elements but that’s negligible.
            let mut results = Vec::with_capacity(objects.len() / 3);

            let objects = objects.into_iter().filter(|obj| match obj.key() {
                None => false,
                Some(key) => !key.ends_with(".sig") && !key.ends_with(".sha256"),
            });

            for object in objects {
                let Some(key) = object.key() else { continue };
                let backup_id = key.to_owned();

                let created_at_opt = object
                    .creation_date()
                    .context(format!("Invalid creation date for '{backup_id}'"))
                    .inspect_err(|err| tracing::warn!("{err:?}"))
                    .ok();

                let checksum_opt = self
                    .get_backup_checksum(&backup_id)
                    .await
                    .inspect_err(|err| tracing::warn!("{err:?}"))
                    .ok();

                let size_bytes = object.size().map_or_else(
                    || {
                        tracing::warn!("Backup '{backup_id}' has no size.");
                        None
                    },
                    |size: i64| Some(size as u64),
                );

                results.push(BackupMetadata {
                    backup_id,
                    size_bytes,
                    checksum: checksum_opt,
                    created_at: created_at_opt,
                });
            }

            Ok(results)
        }

        async fn get_backup(
            &self,
            backup_id: &BackupId,
        ) -> Result<Option<BackupMetadata>, anyhow::Error> {
            let object = self
                .client
                .get_object()
                .bucket(&self.bucket)
                .key(backup_id)
                .send()
                .await
                .context("Failed to get backup")?;

            let created_at_opt = object
                .creation_date()
                .context(format!("Invalid creation date for '{backup_id}'"))
                .inspect_err(|err| tracing::warn!("{err:?}"))
                .ok();

            let checksum_opt = self
                .get_backup_checksum(&backup_id)
                .await
                .inspect_err(|err| tracing::warn!("{err:?}"))
                .ok();

            let size_bytes = object.size().map_or_else(
                || {
                    tracing::warn!("Backup '{backup_id}' has no size.");
                    None
                },
                |size: i64| Some(size as u64),
            );

            BackupMetadata {
                backup_id,
                size_bytes,
                checksum: checksum_opt,
                created_at: created_at_opt,
            }
        }

        async fn create_backup(&self, backup_data: Bytes) -> Result<BackupMetadata, anyhow::Error> {
            let now = UtcDateTime::now()
                .replace_millisecond(0)
                .expect("0 should be a valid millisecond")
                .format(&Rfc3339)
                .context("Could not get current time as RFC 3339")?;
            let key = format!("prose_{now}");

            let resp = self
                .client
                .put_object()
                .bucket(&self.bucket)
                .key(&key)
                .body(ByteStream::from(backup_data))
                .send()
                .await
                .context("failed to upload backup")?;

            let etag = resp.e_tag().unwrap_or("").trim_matches('"').to_string();

            // After upload, get metadata (size, creation, etc.)
            let head = self
                .client
                .head_object()
                .bucket(&self.bucket)
                .key(&key)
                .send()
                .await
                .context("failed to head uploaded backup")?;

            let size_bytes = head.content_length() as u64;
            let created_at = head
                .last_modified()
                .map(|lm| OffsetDateTime::from_unix_timestamp(lm.secs()))
                .transpose()
                .context("invalid timestamp")?
                .unwrap_or_else(OffsetDateTime::now_utc);

            todo!()
        }

        async fn delete_backup(&self, backup_id: &BackupId) -> Result<(), anyhow::Error> {
            let key = Self::object_key(&backup_id);

            let identifiers: Vec<ObjectIdentifier> = objects
                .into_iter()
                .map(|obj| ObjectIdentifier::builder().key(obj.key.unwrap()).build())
                .collect();

            if !identifiers.is_empty() {
                client
                    .delete_objects()
                    .bucket(bucket)
                    .delete(
                        aws_sdk_s3::types::Delete::builder()
                            .set_objects(Some(identifiers))
                            .build(),
                    )
                    .send()
                    .await?;
            }

            self.client
                .delete_object()
                .bucket(&self.bucket)
                .key(&key)
                .send()
                .await
                .context("failed to delete backup")?;

            Ok(())
        }
    }

    impl S3BackupRepository {
        async fn get_backup_checksum(&self, backup_id: &BackupId) -> Result<String, anyhow::Error> {
            let checksum_data = self
                .client
                .get_object()
                .bucket(&self.bucket)
                .key(format!("{backup_id}.sha256"))
                .send()
                .await
                .context(format!("Could not get checksum for backup '{backup_id}'"))?;

            let checksum_bytes = checksum_data
                .body
                .collect()
                .await
                .context(format!("Could not read checksum for backup '{backup_id}'"))?;

            let checksum = String::from_utf8(checksum_bytes.into_bytes().to_vec())
                .context(format!("Invalid checksum for backup '{backup_id}'"))?;

            Ok(checksum)
        }
    }

    trait ObjectExt {
        fn creation_date(&self) -> Result<OffsetDateTime, anyhow::Error>;
    }

    impl ObjectExt for Object {
        fn creation_date(&self) -> Result<OffsetDateTime, anyhow::Error> {
            match self.last_modified() {
                Some(date) => OffsetDateTime::from_unix_timestamp_nanos(date.as_nanos())
                    .context("Invalid “last modified” date"),
                None => Err(anyhow!("No “last modified” date.")),
            }
        }
    }

    impl ObjectExt for GetObjectOutput {
        fn creation_date(&self) -> Result<OffsetDateTime, anyhow::Error> {
            match self.last_modified() {
                Some(date) => OffsetDateTime::from_unix_timestamp_nanos(date.as_nanos())
                    .context("Invalid “last modified” date"),
                None => Err(anyhow!("No “last modified” date.")),
            }
        }
    }
}

// MARK: - Boilerplate

impl std::ops::Deref for BackupRepository {
    type Target = Arc<dyn BackupRepositoryImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}
