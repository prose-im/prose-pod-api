# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs).
For more information about what it is, see [Documenting Architecture Decisions][adr]
([permalink][adr-permalink]).

To write a new ADR for Prose Pod API, use [YYYY-MM-DD-z-template.md](./YYYY-MM-DD-z-template.md)
as a template and replace every occurrence of `<TODO:Whatever>` by whatever it should be.

## Accepted ADRs

- [Describe Prose Pod API using the OpenAPI Specification](./2023-12-18-a-describe-with-openapi.md) (2023-12-18)
- [Use SeaORM to interact with the SQLite database](./2023-12-21-a-use-sea-orm.md) (2023-12-21)
- [Write tests with the Gherkin syntax](./2024-01-11-a-write-tests-in-gherkin.md) (2024-01-11)
- [Interact with Prosody using a REST API](./2024-04-04-a-prosody-rest-api.md) (2024-04-04)
- [Write the OpenAPI description file manually](./2024-04-25-a-write-openapi-manually.md) (2024-04-25)
- [Write integration tests](./2024-05-15-a-integration-testing.md) (2024-05-15)
- [Use Step CI for integration testing](./2024-05-15-b-step-ci-for-integration-testing.md) (2024-05-15)
- [Enrich member data in a separate HTTP API call](./2024-05-27-a-lazily-enriching-member-data.md) (2024-05-27)
- [Use `prose-xmpp` and `mod_rest` to send stanzas to Prosody](./2024-06-05-a-prose-xmpp-and-mod_rest-to-send-stanzas.md) (2024-06-05)
- [Store workspace data in a vCard](./2024-07-14-a-store-workspace-data-in-xmpp-vcard.md) (2024-07-14)
- [Rotate service accounts passwords at every startup](./2024-07-16-a-rotate-service-passwords.md) (2024-07-16)
- [Use Axum instead of Rocket](./2025-01-01-a-use-axum-instead-of-rocket.md) (2025-01-01)
- [Stop exposing the OpenAPI docs](./2025-09-13-a-stop-exposing-openapi-docs.md) (2025-09-13)

## Proposed ADRs

- [Maintain TypeScript and Rust Prose Pod API SDKs](2025-09-13-b-prose-pod-api-sdks.md) (2025-09-13)

## Superseded ADRs

- [Automatically detect OpenAPI routes](./2023-12-18-b-auto-detect-openapi-routes.md) (2023-12-18)
- [Expose API documentation route using Redoc (instead of Swagger UI)](./2024-04-25-b-use-redoc-instead-of-swagger-ui.md) (2024-04-25)

## Deprecated ADRs

- ø

## Rejected ADRs

- ø

[adr]: https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions "Documenting Architecture Decisions | Cognitect"
[adr-permalink]: https://web.archive.org/web/20240104230549/https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions "Documenting Architecture Decisions | Wayback Machine"
