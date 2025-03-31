# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

<!-- WARN: Do not move the next line and add changelog entries **under** it.
       It’s used by `task release` when updating the changelog. -->
[Unreleased]: https://github.com/prose-im/prose-pod-api/compare/v0.10.0...HEAD

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
- feat: Add `PUT /v1/pod/config/dashboard-address` (@RemiBardon in #188).
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
