# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

<!-- WARN: Do not move the next line and add changelog entries **under** it.
       It’s used by `task release` when updating the changelog. -->
[Unreleased]: https://github.com/prose-im/prose-pod-api/compare/v0.19.1...HEAD

### Changed

- chore(app-config): Use `figment` correctly instead of using `serde(default)` (in `4b1c13b7`)

### Added

- test(app-config): Add tests for smart default values (in `fd96cf88`)

### Fixed

- fix: Fix database locks by opening two connection pools (in `8d808920`)
- fix(local-run): Fix `task telemetry:start` not working when Docker network changed (in `cf3c6b46`)

## [0.19.1] - 2025-09-16

[0.19.1]: https://github.com/prose-im/prose-pod-api/compare/v0.19.0...v0.19.1

### Fixed

- Fix API crashing if config key `log` is missing (in `68e2813d`)

## [0.19.0] - 2025-09-16

[0.19.0]: https://github.com/prose-im/prose-pod-api/compare/v0.18.0...v0.19.0

### Changed

- feat(health-check)!: Move health check from `GET /` to `GET /health(z)` (in `1f68a081`)

## [0.18.0] - 2025-09-15

[0.18.0]: https://github.com/prose-im/prose-pod-api/compare/v0.17.3...v0.18.0

### Removed

- feat!: Stop exposing the OpenAPI docs (in `f639c0ff`)

### Changed

- feat(pod-config)!: Rename `GET /v1/pod/config/dashboard-url` to `GET /v1/dashboard/config/url` (in `61d105eb` + `624c72c2`)
- chore: Update Redoc (in `d555bc76`)
- chore(app-config): Set default `api.databases.main.max_connections` to `1` (in `559d4a4b`)

### Added

- feat(licensing): Add `GET /v1/licensing/license` (in `78f201a5`)
- feat(reports): Add `GET /v1/reports/cloud-api` (in `e99815e9`)
- feat(licensing): Add `GET /v1/licensing/status` (in `5fa0a1f7`)
- feat(members): Add `GET /v1/members/{jid}/email-address` (in `13146744`)
- feat: Add health check route (`GET /`) (in `c5d04861`)
- feat: Parse/validate all user input (in `3740e500`)
- docs(members): Document `PUT /v1/members/{jid}/email-address`, along with a ton of code improvements (in `19ea5027`)
- docs(members): Document `GET /v1/members`’s `q` query parameter (in `aedf0a05`)
- chore: Add `serde(deny_unknown_fields)` to all DTOs (in `de4a49d7`)
- feat(local-run): Allow running the Dashboard with `+dashboard` (in `4bae47a4`)

### Fixed

- fix(licensing): Fix license installation (in `8c8d5711`)
- fix(server-config): Fix reset routes not reloading the server (in `33ed647b`)
- fix(security): Never trust user-provided MIME type (in `e93a9d2a`)
- feat(app-config): Support sub-second durations in app config (in `e51aa913`)
- fix(invitations): `"You have been invited to {company}'s Prose server!"` isn't correct with company names ending with a “s” (in `196f47e2`)
- fix: Run tests from all workspace crates in `task test` (in `3ec78897`)
- perf: Attach user info to requests to avoid querying Prosody and the database multiple times (in `7dc524c7`)
- chore: Read log format from config/env before initializing tracing (in `358aa793`)

## [0.17.3] - 2025-09-06

[0.17.3]: https://github.com/prose-im/prose-pod-api/compare/v0.17.2...v0.17.3

### Fixed

- fix: Remove committed `dbg!` leaking license data (in `69181f71`)

## [0.17.2] - 2025-09-05

[0.17.2]: https://github.com/prose-im/prose-pod-api/compare/v0.17.1...v0.17.2

### Changed

- chore: Rename `guards` to `extractors` (in `b06b47b2`)
- chore(tests): Change integration tests domain to `test.local` (in `c962045d`)

### Added

- feat(lifecycle): Reload on `SIGHUP` (in `0f2aa704`)
- feat(release): Update packaged license when running `task release` (in `602f27c9`)
- feat(version): Add a route which returns the Server version (in `8e2f361a`)
- feat(licensing): Add `PUT /v1/licensing/license` to remotely update the license (in `2e0fdbbb`)
- feat(local-run): Allow running without the `otel-collector` service (in `8c3e2fb3`)
- feat(app-config): Allow customizing public contacts (in `c80fbe10`)
- feat(app-config): Use the oldest admin as the admin contact by default (in `2fc39918`)

### Fixed

- fix(prosody): Do not set `http_interfaces` to `{ "*" }` (in `d1a413e0`)
- fix(prosody): Do not set `interfaces` to `{ "*" }` (in `eb015ae3`)
- fix(prosody): Do not set `https_interfaces` to `{}` (in `80fecd3d`)
- fix(prosody): Do not set `https_ports` to `{}` (in `67699858`)
- fix(prosody): Use default `c2s_port` implicitly (in `bcb51035`)
- ci: Fix `xh` installation (in `61142332`)
- fix(tests): Uncomment `members` integration tests (in `218d6573`)

## [0.17.1] - 2025-08-15

[0.17.1]: https://github.com/prose-im/prose-pod-api/compare/v0.17.0...v0.17.1

### Fixed

- fix(startup): Fix skipping one startup action would skip all actions (in `ba7499b2`)
- fix(prosody-config): Fix `Interface::AllIPv6` being parsed as `::1` (in `9ceab890`)

## [0.17.0] - 2025-08-14

[0.17.0]: https://github.com/prose-im/prose-pod-api/compare/v0.16.4...v0.17.0

### Changed

- feat(licensing): Replace constant user limit by a license system (in `818f4095`)

## [0.16.4] - 2025-08-14

[0.16.4]: https://github.com/prose-im/prose-pod-api/compare/v0.16.3...v0.16.4

### Added

- feat(startup): Load XMPP users at startup.

  If some are not in the Pod API database, create them.
  If there are open invitations for them, read the email address then delete the invitation.
  This improves the migration UX.

## [0.16.3] - 2025-06-27

[0.16.3]: https://github.com/prose-im/prose-pod-api/compare/v0.16.2...v0.16.3

### Changed

- feat(app-config): Make `notifiers.email` completely optional again (in `739cb33d`)

### Fixed

- fix(app-config): Fix default values overriding existing ones… (in `587ae74c`)
- fix(dns-setup): Do not mix DNS record types in the same step (in `53c5b62b`)
- fix: Fix API not exiting on `panic!` when startup actions fail on the first startup (in `22975367`)
- fix(prosody): Fix `http_external_url` (in `c49775f0`)

## [0.16.2] - 2025-06-26

[0.16.2]: https://github.com/prose-im/prose-pod-api/compare/v0.16.1...v0.16.2

### Changed

- feat(dns-setup): Rename `xmpp.{domain}` to `prose.{domain}` (in `3a86e811`)
- chore(dns-setup): “let servers connect” -> “let other servers connect” (in `043d7621`)
- chore(dns-setup): “let clients connect” -> “let users connect” (in `23bada88`)

### Added

- feat(dns-setup): Provide standard SRV records (e.g. `_xmpp-client._tcp.{domain}`) (in `3e36d360`)
- feat(dns-setup): Provide `CNAME` records for the Web app and Dashboard (in `0caf5247`)

## [0.16.1] - 2025-06-24

[0.16.1]: https://github.com/prose-im/prose-pod-api/compare/v0.16.0...v0.16.1

### Changed

- ci: Create GitHub release AFTER the image is released (in `5db36736`)

### Added

- chore(app-config)!: Make `skip_startup_actions` available in release builds (in `1304bb6b`)

## [0.16.0] - 2025-06-24

[0.16.0]: https://github.com/prose-im/prose-pod-api/compare/v0.15.0...v0.16.0

### Changed

- feat!: Change `prose.org.local` for `prose.local` (in `c7bd0ea5`)
- chore(app-config)!: Rename some configuration keys (in `a1e92e72`):

  | Before | After |
  | --- | --- |
  | `dashboard_url` | `dashboard.url` |
  | `address` | `api.address` |
  | `port` | `api.port` |
  | `default_response_timeout` | `api.default_response_timeout` |
  | `default_retry_interval` | `api.default_retry_interval` |
  | `notify.*` | `notifiers.*` |
  | `database.*` | `api.databases.*` |
  | `branding.app_name` | `branding.api_app_name` |
- feat(app-config): Provide defaults for more config keys (in `cb558ab9`)
  - `dashboard.url` (`dashboard_url`) is now optional
  - `pod.address.domain` is now optional
  - `notifiers.email.pod_address` (`motify.email.pod_addess`) is now optional if `pod.address.ipv4` and `pod.address.ipv6` are undefined
- chore(app-config)!: Move app config from `/etc/prose-pod-api` to `/etc/prose` (in `84c0d03d`)
- chore(app-config)!: Rename `Prose.toml` to `prose.toml` (in `240810f9`)
  - `/etc/prose-pod-api/Prose.toml` is now `/etc/prose/prose.toml`
- feat(local-run): Update `prose-pod-dashboard` (in `dd117278`)

### Added

- feat(app-config)!: Add support for more keys
  - ```diff
    + log
    + use_libevent
    + restrict_room_creation
    + default_archive_policy
    - prosody
    + prosody.*.defaults
    + prosody.*.overrides
    ```
  - All keys for `mod_c2s`, `mod_s2s`, `mod_http_file_upload`, `mod_disco`
  - More keys for timeouts and limits (in `26a2480f`)
  - Allow deserializing `Bytes` (in `50edb36a`)
  - Allow deserializing durations (in `226870be`)
  - Allow deserializing `DataRate` (in `6b0973d1`)
  - Allow per-host defaults and overrides for Prosody (in `3e986bf7`)
- feat(server-config): Enable `muc_public_affiliations` in `muc` component (in `e217343b`)
- feat(server-config): Add `disco_items` in main host (in `9e47b8dc`)
- feat(server-config): Enable `reload_modules` (in `d9595285`)
- feat(startup): Add `update_rosters` startup action (in `27acbb97`)
- feat(tests): Allow logging app config in smoke tests (in `7f02bd4d`)

### Fixed

- fix(server-config): Fix `http_file_share` component (in `c429b0c2`)
- fix(server-config): Fix `contact_info` (in `7948b28a`)
- fix(local-run): Fix `make-demo-scenario` (in `3ee725d3`)
- fix(tests): Make tests more consistent (in `f2f105f9`)
- fix(app-config): Allow skipping any startup action using `debug_only.skip_startup_actions` (in `52b0e7a8`)

## [0.15.0] - 2025-06-22

[0.15.0]: https://github.com/prose-im/prose-pod-api/compare/v0.14.1...v0.15.0

### Removed

- feat(app-config)!: Replace server config initialization step by a static configuration key (in `44ac2f1`)
- feat(app-config)!: Replace pod config initialization step by static configuration keys (in `f469a60`)

### Changed

- feat: Set `push_notification_with_*` to `true` by default (in `5449ac5`)
- feat!: Lower the 100 members limit to 20 (in `2838a6e`)
- feat(local-run): Speed up local builds by using a different Dockerfile (in `02c3263`)
- feat(invitations): Do not return 200 OK when getting details for an expired invitation token (in `df09f8a`)
- feat(invitations)!: Return 410 Gone instead of 404 Not Found on `invitation_not_found` (in `a9d9653`)
- A ton of improvements to the code.

### Added

- feat(profile): Add `PUT /v1/members/{jid}/email-address` (in `2709119`)
- feat(auth): Implement password reset (in `dd2ebcd`)
- feat(invitations): Store email addresses when accepting invitations (in `0a401e8`)
- feat(local-run): Allow running the Dashboard locally too (in `8347293`)
- feat: Add support for all keys from `ProsodySettings` in `PUT /v1/server/config/prosody-overrides` (in `bfac9a7`)
- feat(invitations): Return whether or not a Workspace invitation is expired when getting its details (in `2b581d0`)

### Fixed

- fix(network-checks): Do not generate S2S DNS record if federation is disabled (in `a5163be`)
- fix(dns-setup): Do not check S2S network config if federation is disabled (in `a5163be`)
- fix(app-config): Fix `server.log_level` config not being taken into account (in `a77d07d`)
- fix(network-checks): Fix tokio panic by using hickory’s `TokioAsyncResolver` (in `fb2598f`)

## [0.14.1] - 2025-05-16

[0.14.1]: https://github.com/prose-im/prose-pod-api/compare/v0.14.0...v0.14.1

### Changed

- chore(local-run): Improve logs when running locally (in `3e05670`, `d7b7fde`, `2152a00`)
- feat(errors): Return error code `invalid_auth_token` when auth token is invalid (in `41d47dc`)
- chore(auth): Improve `AuthService` errors (in `41d47dc`)

### Fixed

- fix: Fix #248 and #249 (in `71b0ee4`)

## [0.14.0] - 2025-05-15

[0.14.0]: https://github.com/prose-im/prose-pod-api/compare/v0.13.0...v0.14.0

### Removed

- chore(server-config): Remove hard-coded `ssl` Prosody config (in `cd3c053`)
- feat(local-run)!: Remove `--no-update` from `task local:run` (in `34c7290`)

### Changed

Breaking:

- feat(app-config)!: Rename `server.prosody_config_file_path` into `prosody.config_file_path` (in `ec4beff`)
- chore(local-run)!: Rename `prosody.initial.cfg.lua` into `prosody.bootstrap.cfg.lua` (in `49cb15d`)
- feat(server-config)!: Use `default_storage` instead of `storage` (in `6e1914c`, `9bde5b7`)
- feat(init)!: Change `HEAD /v1/init/first-account`’s semantics from `PUT` to `GET` (in `0ccc2e7`)
- feat(factory-reset)!: Require a password confirmation on first step of a factory reset (in `7ca76ab`)
- feat(workspace-details)!: Accept content types when setting and getting the Workspace icon (in `30401bd`)

Non-breaking:

- feat(network-checks): Add a default timeout of 5 minutes for SSE streams (in `d607603`)
- feat(members): Return content type along with base64 member avatars (in `235d67a`)
- feat(errors): Return code `cannot_remove_self` when a member tries to remove themselves (in `8d73945`)
- feat(errors): Create error codes `cannot_change_own_role` and `cannot_assign_role` (in `a6043c7`)
- chore: Move a ton of logic from `rest-api` to `service`
- chore: Refactor and simplify a lot of code
- chore(observability): Greatly improve how errors are propagated internally
- chore(observability): Improve logging of errors debug info when developing the API
- chore(observability): Improve observability
- chore(run): Switch from `panic = "abort"` to `panic = "unwind"` (in `02daf87`)
- feat(local-run): Delete all ephemeral scenarios instead of just the last one in `task local:stop` (in `f84274c`)
- chore(local-run): Use bootstrap Prosody configuration now packaged in the Server (in `0d0d186`)

### Added

- `Prose.toml` additions:
  - Add `prosody.*` which supports a lot of Prosody keys (in `0b672b4`, `eea7608`)
  - Add `prosody_ext.additional_modules_enabled` (in `e563c54`, `001dddd`)
  - Add `prosody_ext.config_file_path`
  - Allow configuring log levels in a target-agnostic manner (in `085e362`)
    - Add `log.level`
  - Allow configuring log format (in `0bc28a2`, `e2cb6b5`, `d102e11`)
    - Add `log.format`, `log.timer`, `log.with_ansi`, `log.with_file`, `log.with_level`, `log.with_target`, `log.with_thread_ids`, `log.with_line_number`, `log.with_span_events`, `log.with_thread_names`
- `prosody-config` improvements:
  - Add support for `mod_storage_sql` (in `d3a661f`, `565555f`)
  - Add support for `mod_reload_modules` (in `6d911dd`)
  - Derive `Serialize` and `Deserialize` for `ProsodySettings` (not all keys) (in `4a33424`, `14d5bbe`)
  - Allow merging two `ProsodySettings` (in `42b12a4`)
- Members-related improvements:
  - Allow searching for members using the `q` and `until` query parameters (in `b1ed8a1`, `e2d2bd2`)
  - Add support for `HEAD /v1/members` without authentication (in `68b4105`)
  - Cache values when enriching members (in `66e855c`, `59bf4fb`, `da8a0be`)
- New routes:
  - Unauthenticated `GET /v1/server/config` now returns the Server domain (in `a1a5445`)
  - Add `GET /v1/onboarding-steps` (in `8662d2c`)
- chore(run): Declare `VOLUME`s in the Dockerfile (in `ca6d6cd`)
- feat(run): Implement the 100 members limit
- feat(run): Provide a default bootstrap password (in `ef18583`)
- chore: Create a global key/value store (in `b6f2a55`)
- feat(benchmarks): Add benchmark of Prosody storages (in `13d5e9c`)
- feat(scripts): Create `task changelog:prepare` (in `9b4d492`)

### Fixed

- fix(network-checks): Fix API crash when client closes network checks SSE (in `71d7b3c`)
- fix(run): Fix `ConcurrentTaskRunner` so it doesn’t panic when passed 0 elements (in `23b251e`)
- fix(auth): Fix API returning 500 instead of 401 when logging in with a bad password (in `10c4c16`)
- fix(network-checks): Fix network checks not stopping after the SSE connection is closed (in `c528791`)
- fix(local-run): Fix uncommitted change when running a scenario without `--no-update` (in `70849a9`)
- fix(run): Fix API being broken after being suspended for too long (in `9b11d89`)
- fix(members): Make adding members a O(1) operation (≈150ms) (in `229fb4d`)
- fix(prosody-config): Fix (de)serialization of `StorageBackend` and support custom values (in `e4fb9f1`)
- fix(members): Remove member from Prosody team when deleting it (in `8a50e91`)

## [0.13.0] - 2025-04-10

[0.13.0]: https://github.com/prose-im/prose-pod-api/compare/v0.12.0...v0.13.0

### Changed

- chore: Remove unused `IpConnectivityStatus::Missing` case (@RemiBardon in 9ec5ec9).
- chore: Update Redoc (@RemiBardon in 946fd0e).
- chore(local-run): Update `demo` scenario (@RemiBardon in ae4e5df).
- feat!: Move Pod versions to `GET /pod/version` and rename `self` to `api` (@RemiBardon in 13036b3).
- feat!: Use `If-Match` instead of `If-None-Match` (@RemiBardon in 07b1725).

### Added

- feat: Add Workspace XMPP account to everyone’s rosters (@RemiBardon in #202).
- docs(openapi): Document `GET /version` (@RemiBardon in 76a6d9e).
- feat: Add `GET /v1/server/config/prosody` (@RemiBardon in 5b55708 in #206).
- feat: Add `PUT|GET|DELETE /v1/server/config/prosody-overrides` (@RemiBardon in #206).
- feat: Add `PUT|GET|DELETE /v1/server/config/prosody-overrides-raw` (@RemiBardon in #207).
- feat(app-config): Add `debug_use_at_your_own_risk.c2s_unencrypted` (@RemiBardon in #208).
- feat(app-config): Add `server.log_level` (@RemiBardon in #210).
- feat(startup): Add `migrate_workspace_vcard` startup action (@RemiBardon in #209).
- feat: Add `HEAD` routes for initialization steps (@RemiBardon in 07b1725).
- feat: Allow checking of one can send invitations in advance (@RemiBardon in 2b58c0f).

### Fixed

- fix(scripts): Fix scripts so they don’t require `declare -n`. Because of this, one couldn’t
  use the version of Bash that’s shipped by default on macOS. (@RemiBardon in 9bf1aa9).
- fix(local-run): Expose Prosody ports `5222`, `5269`, `5280` and `5281` when running locally
  to allow one to log into the local Prosody instance (@RemiBardon in a440982 and 3003b61).
- fix(openapi): `PUT /v1/server/config` accepts a JSON object (@RemiBardon in 4c68734).
- fix: Refresh service accounts OAuth tokens before they expire (@RemiBardon in #213).
- fix: Fix API hangs when running network checks after 5 minutes (@RemiBardon in #215).
- fix(invitations): Send emails as plain UTF-8 and HTML (@RemiBardon in #217).
- docs(openapi): Fix description of `PUT /v1/workspace/icon` (@RemiBardon in c6f0285).
- docs(openapi): Fix authentication of `init_pod_config` (@RemiBardon in 57cacae).
- fix(invitations): Return `204` when rejecting a used invitation (@RemiBardon in 1bac849).

## [0.12.0] - 2025-04-02

[0.12.0]: https://github.com/prose-im/prose-pod-api/compare/v0.11.0...v0.12.0

### Removed

- feat(pod-config)!: Remove `type: [Static, Dynamic]` from the Pod address
  (@RemiBardon in 58cb3bb).
- docs(openapi): Hide unimplemented `PUT /v1/server/config/file-storage-encryption-scheme` route
  (@RemiBardon in cb245ec).

### Changed

- feat: Set Workspace vCard `KIND` to `application` (@nesium in #198).
- feat!: Routes (i.e. `/v1/server/config/tls-profile`) now return primitive JSON types when
  they point to a primitive data type (e.g. just `"Prose (demo)"` for `GET /v1/workspace/name`)
  (@RemiBardon in [#200]).
  - See [#200 Remove API discrepancies and add missing CRUD routes][#200] for examples and explanations.
- feat(server-config)!: Use HTTP verb `DELETE` instead of `PUT …/reset` (@RemiBardon in 860b5c7).

  ```diff
  -    PUT /v1/server/config/messaging/reset
  + DELETE /v1/server/config/messaging
  -    PUT /v1/server/config/message-archive-retention/reset
  + DELETE /v1/server/config/message-archive-retention
  -    PUT /v1/server/config/files/reset
  + DELETE /v1/server/config/files
  -    PUT /v1/server/config/push-notifications/reset
  + DELETE /v1/server/config/push-notifications
  -    PUT /v1/server/config/push-notification-with-body/reset
  + DELETE /v1/server/config/push-notification-with-body
  -    PUT /v1/server/config/push-notification-with-sender/reset
  + DELETE /v1/server/config/push-notification-with-sender
  -    PUT /v1/server/config/network-encryption/reset
  + DELETE /v1/server/config/network-encryption
  -    PUT /v1/server/config/tls-profile/reset
  + DELETE /v1/server/config/tls-profile
  -    PUT /v1/server/config/server-federation/reset
  + DELETE /v1/server/config/server-federation
  -    PUT /v1/server/config/federation-enabled/reset
  + DELETE /v1/server/config/federation-enabled
  -    PUT /v1/server/config/federation-whitelist-enabled/reset
  + DELETE /v1/server/config/federation-whitelist-enabled
  -    PUT /v1/server/config/federation-friendly-servers/reset
  + DELETE /v1/server/config/federation-friendly-servers
  ```

[#200]: https://github.com/prose-im/prose-pod-api/pull/200 "Remove API discrepancies and add missing CRUD routes"

### Added

- `PATCH /v1/pod/config/address` (@RemiBardon in 75dad56).
- feat(server-config): Make sure all server configs have `GET`, `PUT` and `DELETE` routes (@RemiBardon in 1d26c3f).

  ```diff
  +    GET /v1/server/config/file-upload-allowed
  + DELETE /v1/server/config/file-upload-allowed
  +    GET /v1/server/config/file-storage-retention
  + DELETE /v1/server/config/file-storage-retention
  +    GET /v1/server/config/message-archive-enabled
  + DELETE /v1/server/config/message-archive-enabled
  +    GET /v1/server/config/message-archive-retention
  +    GET /v1/server/config/push-notification-with-body
  +    GET /v1/server/config/push-notification-with-sender
  +    GET /v1/server/config/tls-profile
  +    GET /v1/server/config/federation-enabled
  +    GET /v1/server/config/federation-whitelist-enabled
  +    GET /v1/server/config/federation-friendly-servers
  ```
- feat(server-config): Return header `Content-Location` in "Server config" group routes (@RemiBardon in dc10322).

### Fixed

- fix(openapi): Return `204 No Content` in "Resend an invitation" (@RemiBardon in 4bbda5c).

## [0.11.0] - 2025-04-01

[0.11.0]: https://github.com/prose-im/prose-pod-api/compare/v0.10.0...v0.11.0

### Removed

- feat(api-config): Do not require email notifier configuration at startup (@RemiBardon in #188).
  - Attention: this means routes requiring an email notifier (e.g. invitations) will now fail **at runtime**.
- feat(api-config)!: Configuration key `branding.page_url` has been removed (@RemiBardon in #188).
  - Breaking: This can now be set using `PUT /v1/pod/config/dashboard-address`.
  - Attention: the workspace now has to be initialized before sending invitations.
- feat(api-config): Configuration key `branding.company_name` isn’t required anymore (@RemiBardon in #188).
  - Attention: the workspace now has to be initialized before sending invitations.

### Changed

- chore: Move OpenAPI docs to `docs/openapi/` (@RemiBardon in 4702409).
- chore: Move tutorials to `docs/tutorials/` (@RemiBardon in ec803e9).
- chore: Move `crates/` to `src/` (@RemiBardon in 1756a86).
- chore: Move `ADRs/` to `docs/ADRs/` (@RemiBardon in 2e58e92).
- feat(tasks): Run Jaeger by default on `task telemetry:start` (@RemiBardon in 4fa9d94).
- docs(openapi): Improve errors (@RemiBardon in 3a7fe7f).
- chore(tasks): Add default tag message on `task release` (@RemiBardon).

### Added

- feat!: Implement factory reset (`DELETE /`) (@RemiBardon in #188).
- feat: Add `PUT /v1/pod/config/dashboard-url` (@RemiBardon in #188 and #197).
- feat(tasks): Update changelog in `task release` (@RemiBardon).
- feat: Enable the `register` module in Prosody (@RemiBardon in #196).

## [0.10.0] - 2025-03-25

[0.10.0]: https://github.com/prose-im/prose-pod-api/compare/v0.9.0...v0.10.0

### Removed

- feat(workspace-details)!: Remove routes consuming and emitting `text/vcard` data (@RemiBardon in #180).

### Changed

- feat(workspace-details): Move workspace accent color to workspace vCard (@RemiBardon in #180).

### Added

- feat: Add support for OpenTelemetry tracing (@RemiBardon in #177).
- feat(workspace-details): Add `KIND:org` to the workspace vCard (@RemiBardon in #180).

## [0.9.0] - 2025-02-13

[0.9.0]: https://github.com/prose-im/prose-pod-api/compare/v0.8.2...v0.9.0

### Changed

- feat!: Use status code `412` instead of `400` when things are not initialized (@RemiBardon in #175).

### Added

- feat(server-config): Add `HEAD /v1/server/config` (@RemiBardon in #176).

## [0.8.2] - 2025-01-31

[0.8.2]: https://github.com/prose-im/prose-pod-api/compare/v0.8.1...v0.8.2

### Fixed

- fix(auth): Return `401`s when tokens expire (not `403s`) (@RemiBardon in #172).

## [0.8.1] - 2025-01-28

[0.8.1]: https://github.com/prose-im/prose-pod-api/compare/v0.8.0...v0.8.1

### Added

- feat: Serve version information on `/v1/version` too (@RemiBardon in 1344c40).

### Fixed

- fix(local-run): Fix release script which wouldn’t change last version for local runs (@RemiBardon in 2b79a49).

## [0.8.0] - 2025-01-28

[0.8.0]: https://github.com/prose-im/prose-pod-api/compare/v0.7.0...v0.8.0

### Removed

- build: Stop using `cargo-chef` (@RemiBardon in #169).

### Changed

- ci: Speed up `edge` build (@RemiBardon in #135).
- chore(deps): Update `axum` to `0.8` (@RemiBardon in #145).
- chore(deps): Update Prosody (@RemiBardon in #158).
- chore(deps): Update dependencies (@RemiBardon in #157).

And other improvements which didn’t need a PR.

### Added

- feat(local-run): Add persistent storage for Mailpit (@RemiBardon in #139).
- feat: Allow returning detailed error responses in non-debug builds (@RemiBardon in #141).
- feat: Add a route to get the current API version information (@RemiBardon in #144).
- feat(invitations)!: Accept `auto_accept=true` on route `POST /v1/invitations` (@RemiBardon in #148).
- test: Run integration tests on non-empty workspaces (@RemiBardon in #149).
- test: Ensure no one can change someone else’s nickname (@RemiBardon in #152).

### Fixed

- fix(local-run): Fix data not persisted (@RemiBardon in #137).
- fix: Prevent one from deleting their own account (@RemiBardon in #142).
- fix(tests): Fix broken and unpredictable integration test results (@RemiBardon in #153).
- fix(pod-address): Fix socket hang up when setting pod address to an empty value (@RemiBardon in #156).
- fix(auth): Yield 401 Unauthorized when logging in with a bad JID (@RemiBardon in #159).
- fix(ci): Fix release builds (@RemiBardon in #167).

## [0.7.0] - 2025-01-15

[0.7.0]: https://github.com/prose-im/prose-pod-api/compare/v0.6.0...v0.7.0

### Added

- feat(server-config)!: Add “Network encryption” configuration routes (@RemiBardon in #127).
- feat(server-config): Add “Server federation” configuration routes (@RemiBardon in #128).
- feat(workspace-vcard): Add routes to get and set the workspace vCard4 (@RemiBardon in #134).

## [0.6.0] - 2025-01-11

[0.6.0]: https://github.com/prose-im/prose-pod-api/compare/v0.5.0...v0.6.0

### Changed

- feat(config)!: Change default address to `0.0.0.0` again (@RemiBardon in #123).
- test: Improve integration tests (@RemiBardon in #124).
- chore(local-run): Improve local run experience (@RemiBardon in #125).

### Added

- feat(config): Add a default value for `databases.main.url` (@RemiBardon in #126).

### Fixed

- fix(invitations): Send emails for real (@RemiBardon in #122).

## [0.5.0] - 2025-01-09

[0.5.0]: https://github.com/prose-im/prose-pod-api/compare/v0.4.0...v0.5.0

### Changed

- chore!: Migrate from Rocket to Axum (@RemiBardon in #95).
  - `Prose.toml` needs more keys, since we got rid of `Rocket.toml`:

    ```diff
    + address = "0.0.0.0"

    + [databases.main]
    + url = "sqlite://database.sqlite?mode=rwc"
    ```

    We’ll make those defaults in the future, we didn't notice these breaking changes at first.

## [0.4.0] - 2025-01-01

[0.4.0]: https://github.com/prose-im/prose-pod-api/compare/v0.3.1...v0.4.0

### Changed

- refactor!: Continue restructuring by feature (@RemiBardon in #100).
  - `GET /v1/invitations/{invitationIdOrToken}` was split in two: `GET /v1/invitations/{invitationId}` and `GET /v1/invitation-tokens/{token}/details`
    - `GET /v1/invitations/{invitationId}` don't need any query parameter (like before)
    - `GET /v1/invitation-tokens/{token}/details` expects `token_type=(accept|reject)` (like before)
  - `PUT /v1/invitations/{token}/accept` is now `/v1/invitation-tokens/{token}/accept`
  - `PUT /v1/invitations/{token}/reject` is now `/v1/invitation-tokens/{token}/reject`

## [0.3.1] - 2024-12-19

[0.3.1]: https://github.com/prose-im/prose-pod-api/compare/v0.3.0...v0.3.1

### Changed

- chore(deps): Bump dependencies (@RemiBardon in #98).
- build: Use `cargo-chef` to speed up builds (@RemiBardon in #99).
- build: Disable unnecessary cargo features to speed up builds (@RemiBardon in #102).
- ci: Do not run tests on draft PRs (@RemiBardon in #103).

### Added

- test(workspace): Add integration tests for `GET /v1/workspace(/*)?` routes (@RemiBardon in #94).
- feat: Add support for XEP-0357: Push Notifications (@RemiBardon in #97).

### Fixed

- ci: Fix release builds (@RemiBardon in #96).

## [0.3.0] - 2024-12-11

[0.3.0]: https://github.com/prose-im/prose-pod-api/releases/tag/v0.3.0

### Added

- First release.
