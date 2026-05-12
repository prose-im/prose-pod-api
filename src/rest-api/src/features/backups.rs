// prose-pod-server
//
// Copyright: 2026, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::LazyLock;

use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, HeaderValue, Uri},
    response::Response,
    routing::{any, put},
};
use service::auth::UserInfo;

use crate::{error::Error, AppState};

static RESTORE_TOKEN: LazyLock<String> =
    LazyLock::new(|| service::util::random_string_alphanumeric(32));

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/v1/backups", any(super::backups::server_api_backups_proxy))
        .route(
            "/v1/backups/{*path}",
            any(super::backups::server_api_backups_proxy),
        )
        .route(
            "/v1/backups-internal/restore",
            put(routes::put_backup_internal_restore),
        )
        .with_state(app_state)
}

pub async fn server_api_backups_proxy(
    state: State<AppState>,
    request_uri: Uri,
    request_method: axum::http::Method,
    mut request_headers: HeaderMap,
    user_info: UserInfo,
    request_body: Bytes,
) -> Result<Response, Error> {
    if request_method == axum::http::Method::POST && request_uri.path() == "/v1/backups" {
        routes::post_backups(state, request_headers, user_info, request_body).await
    } else {
        if request_uri.path().ends_with("/restore") {
            request_headers.insert(
                "x-prose-token",
                HeaderValue::from_str(RESTORE_TOKEN.as_str()).unwrap(),
            );
        }

        super::server_api_proxy::server_api_proxy(
            state,
            request_uri,
            request_method,
            request_headers,
            Body::from(request_body),
        )
        .await
    }
}

mod routes {
    use std::path::Path;

    use anyhow::Context as _;
    use axum::{
        body::{Body, Bytes},
        extract::State,
        http::{HeaderMap, HeaderValue, Uri},
        response::{IntoResponse, Response},
        Json,
    };
    use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
    use service::{auth::UserInfo, sea_orm::ConnectionTrait as _};
    use validator::Validate;

    use crate::{
        error::{self, Error},
        features::backups::RESTORE_TOKEN,
        AppState, MinimalAppState,
    };

    const PROSE_POD_API_DATA_KEY: &str = "prose-pod-api-data";

    /// `POST /v1/backups`.
    pub async fn post_backups(
        state: State<AppState>,
        mut request_headers: HeaderMap,
        user_info: UserInfo,
        request_body: Bytes,
    ) -> Result<Response, Error> {
        if !user_info.is_admin() {
            return Err(error::Forbidden("You cannot do that.".to_owned()).into());
        }

        #[derive(Validate, serdev::Deserialize)]
        #[serde(deny_unknown_fields)]
        #[serde(validate = "Validate::validate")]
        struct Request {
            #[validate(length(min = 1, max = 128), non_control_character)]
            description: String,
        }

        let Ok(Json(req)) = Json::<Request>::from_bytes(request_body.as_ref()) else {
            return Err(error::BadRequest {
                reason: "Missing JSON body (description).".to_owned(),
            }
            .into());
        };

        // Flush SQLite before backing up.
        (state.db.write)
            .execute_unprepared("PRAGMA wal_checkpoint(FULL);")
            .await?;

        let buf: Vec<u8> = Vec::new();
        let mut builder = tar::Builder::new(buf);

        let api_data_path = "/var/lib/prose-pod-api";
        builder
            .append_dir_all(PROSE_POD_API_DATA_KEY, api_data_path)
            .context(format!("Dir: {api_data_path:?}"))?;

        // NOTE: We don’t compress because the destination should be
        //   on the same network and this API will disappear anyway
        //   (https://github.com/prose-im/prose-pod-api/discussions/368)
        //   so let’s ignore the overhead. Also the Prose Pod API shouldn’t
        //   have much data so it’s negligible.
        let prose_pod_api_data = builder.into_inner().map_err(|err| {
            error::InternalServerError(format!("Failed archiving Prose Pod API data: {err:#}"))
        })?;

        let uri = Uri::try_from(format!(
            "/v1/backups?description={}",
            urlencoding::encode(&req.description)
        ))
        .map_err(|err| error::InternalServerError(format!("{err:#}")))?;

        request_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-tar"));
        request_headers.insert(CONTENT_LENGTH, HeaderValue::from(prose_pod_api_data.len()));

        crate::features::server_api_proxy::server_api_proxy(
            state,
            uri,
            axum::http::Method::POST,
            request_headers,
            Body::from(prose_pod_api_data),
        )
        .await
    }

    /// `PUT /v1/backups-internal/restore`.
    pub async fn put_backup_internal_restore(
        State(AppState {
            db,
            base:
                MinimalAppState {
                    ref lifecycle_manager,
                    ..
                },
            ..
        }): State<AppState>,
        headers: HeaderMap,
        request_body: Bytes,
    ) -> Result<Response, Error> {
        // NOTE: We cannot use normal auth tokens as the auth server is stopped
        //   during a restore.
        if headers
            .get("x-prose-token")
            .is_none_or(|token| token != RESTORE_TOKEN.as_str())
        {
            return Err(error::Forbidden("You cannot do that.".to_owned()).into());
        }

        let mut archive = tar::Archive::new(std::io::Cursor::new(request_body));

        let api_data_path = Path::new("/var/lib/prose-pod-api");

        let mut revert_guard = super::helpers::backup_children(api_data_path, lifecycle_manager)
            .map_err(|err| {
                error::InternalServerError(format!("Failed backing up data: {err:#}"))
            })?;

        // NOTE: Creating temporary directory in data directory to prevent
        //   “Cross-device link (os error 18)” errors. It’s not ideal, but we
        //   don’t care as this code will disappear anyway
        //   (see <https://github.com/prose-im/prose-pod-api/discussions/368>).
        let tmpdir =
            tempfile::TempDir::with_prefix_in("backup-data-", api_data_path).map_err(|err| {
                error::InternalServerError(format!(
                    "Failed creating temp dir to unpack archive: {err:#}"
                ))
            })?;

        archive.unpack(tmpdir.path()).map_err(|err| {
            error::InternalServerError(format!("Failed unpacking archive: {err:#}"))
        })?;

        // Close the database connection to prevent concurrent writes.
        (db.read.close())
            .await
            .context("Could not close the database connection (read)")?;
        // Close the database connection to prevent concurrent writes.
        (db.write.close())
            .await
            .context("Could not close the database connection (write)")?;

        super::helpers::move_children(tmpdir.path().join(PROSE_POD_API_DATA_KEY), api_data_path)
            .map_err(|err| {
                error::InternalServerError(format!("Failed backing up data: {err:#}"))
            })?;

        revert_guard.defuse();

        lifecycle_manager.set_restarting();

        Ok(Json(()).into_response())
    }
}

mod helpers {
    use std::path::{Path, PathBuf};

    use anyhow::Context;

    use crate::util::LifecycleManager;

    /// A structure that holds the data necessary to revert all changes made
    /// during a restoration when it is dropped. This ensures nothing has
    /// changed if the restoration fails anywhere during the process.
    #[derive(Debug)]
    pub struct RestoreRevertGuard<'a> {
        /// Destination paths which already existed, and which were backed up
        /// (e.g. `.bak`) to prevent data loss.
        ///
        /// This is a list of `(path, backup_path_opt)` pairs.
        paths: Vec<(PathBuf, PathBuf)>,

        /// Indicate if everything went successfully or not. If defused (which
        /// should be the case), dropping this will delete backed up paths. If not,
        /// It will delete created paths and recover backups.
        is_defused: bool,

        lifecycle_manager: &'a LifecycleManager,
    }

    impl<'a> RestoreRevertGuard<'a> {
        pub fn defuse(&mut self) {
            self.is_defused = true;
        }
    }

    impl<'a> Drop for RestoreRevertGuard<'a> {
        fn drop(&mut self) {
            if self.is_defused {
                for (_, backup_path) in self.paths.iter() {
                    if backup_path.exists() {
                        if let Err(err) = self::remove(backup_path) {
                            tracing::error!("Could not delete path backup {backup_path:?}: {err:?}")
                        }
                    }
                }
            } else {
                revert(self.paths.iter());

                tracing::debug!("Restarting to reopen database connections…");
                self.lifecycle_manager.set_restarting();
            }
        }
    }

    pub fn move_children(dir: PathBuf, dest: &Path) -> Result<(), anyhow::Error> {
        for child in std::fs::read_dir(dir)? {
            let child = child?;

            let from = child.path();
            let to = dest.join(child.file_name());
            std::fs::rename(&from, &to)
                .with_context(|| format!("Failed renaming {from:?} to {to:?}"))?;
        }

        Ok(())
    }

    /// Backup destination paths to revert in case an error happens.
    pub fn backup_children<'a>(
        dir: &Path,
        lifecycle_manager: &'a LifecycleManager,
    ) -> Result<RestoreRevertGuard<'a>, anyhow::Error> {
        let mut revert_guard = RestoreRevertGuard {
            paths: Vec::new(),
            is_defused: false,
            lifecycle_manager,
        };

        // NOTE: Read all children instead of iterating because we’ll
        //   be creating more children while iterating (potentially
        //   creating infinite loops).
        let children = std::fs::read_dir(dir)
            .context(format!("Failed reading {dir:?}"))?
            .collect::<Vec<_>>();

        for child in children {
            let child = child.context(format!("Failed reading {dir:?}: Entry is error"))?;

            let child_path = &child.path();

            let child_bak = backup_path(child_path)
                // NOTE: If an error happens here, it aborts the backup
                //   restoration and reverts all changes made until then.
                .map_err(|err| {
                    anyhow::Error::new(err)
                        .context(format!("Failed backing up child {child_path:?}"))
                })?;

            (revert_guard.paths).push((PathBuf::clone(child_path), child_bak));
        }

        Ok(revert_guard)
    }

    /// Note that this is best-effort, meaning we’re already doing error recovery
    /// at this point so we can’t recover from subsequent internal errors.
    #[cold]
    fn revert<'a>(paths: impl Iterator<Item = &'a (PathBuf, PathBuf)>) {
        use std::fs;

        for (path, backup_path) in paths {
            if path.exists() {
                if let Err(err) = self::remove(path) {
                    tracing::error!("Could not delete created path {path:?}: {err:?}")
                }
            }

            if let Err(err) = fs::rename(&backup_path, &path) {
                tracing::error!("Could not recover {path:?}: {err:?}")
            };
        }
    }

    /// Deletes any path and its children.
    #[inline]
    pub fn remove(path: &std::path::Path) -> Result<(), std::io::Error> {
        use std::fs;

        if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        }
    }

    /// Feature `path_add_extension` isn’t stable in Rust edition 2021.
    /// See <https://github.com/rust-lang/rust/issues/127292>.
    /// We’d have to migrate to edition 2024, but it’s unnecessary as this
    /// project will disappear soon
    /// (see <https://github.com/prose-im/prose-pod-api/discussions/368>).
    fn with_added_extension(path: &Path, ext: impl AsRef<std::ffi::OsStr>) -> PathBuf {
        let extension = path
            .extension()
            .into_iter()
            .chain(std::iter::once(ext.as_ref()))
            .collect::<Vec<_>>()
            .join(std::ffi::OsStr::new("."));
        path.with_extension(extension)
    }

    pub fn backup_path(path: &std::path::Path) -> Result<std::path::PathBuf, std::io::Error> {
        use std::fs;
        use std::path::{Path, PathBuf};

        let mut backup_path = with_added_extension(path, "bak");

        // If file already exists, switch to a unique name.
        if fs::exists(&backup_path)? {
            #[cold]
            fn use_unique_name(backup_path: &mut PathBuf, path: &Path) {
                *backup_path = with_added_extension(path, format!("{}.bak", unix_timestamp()));
            }
            use_unique_name(&mut backup_path, &path)
        }

        fs::rename(path, &backup_path)?;

        Ok(backup_path)
    }

    #[inline]
    pub fn unix_timestamp() -> u64 {
        use std::time::{Duration, UNIX_EPOCH};

        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }
}
