# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

<!-- WARN: Do not move the next line and add changelog entries **under** it.
       It’s used by `task release` when updating the changelog. -->
[Unreleased]: https://github.com/prose-im/prose-pod-api/compare/v0.13.0...HEAD

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
