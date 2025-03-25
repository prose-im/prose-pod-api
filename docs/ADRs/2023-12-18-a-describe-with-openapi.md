# ADR: Describe Prose Pod API using the OpenAPI Specification

- Date: **2023-12-18**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Accepted** via [#3](https://github.com/prose-im/prose-pod-api/pull/3) (2024-02-09)
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Relates to:
  - [Automatically detect OpenAPI routes](./2023-12-18-b-generate-openapi-description.md) (2023-12-18)
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Superseded by: ø
<!-- "No" or "Yes" with the deprecation date -->
- Deprecated: No

## Context

<!--
This section describes the forces at play, including technological, political, social, and project local. These forces are probably in tension, and should be called out as such. The language in this section is value-neutral. It is simply describing facts.
-->

The Prose Pod API is meant to be used by the [Prose Pod Dashboard], but also to be
used by automations to administrate a [Prose Pod Server].
For this reason, we would like to document the API so anyone can understand how to use it.

The [OpenAPI Specification] is nowadays the most widely used API description standard.
Citing the [OpenAPI Initiative homepage]:

> The OpenAPI Specification provides a formal standard for describing HTTP APIs.
> This allows people to understand how an API works, generate client code, create tests, apply design standards, and much, much more.

## Decision

<!--
This section describes our response to these forces. It is stated in full sentences, with active voice. "We will …"
-->

We will provide an OpenAPI description file, following the [OpenAPI Specification v3.1.0],
latest version at the time of this ADR.
For this, we will use the [utoipa] crate, developped since October 2021,
downloaded 1,450,715 times on [crates.io] at the time of writing this ADR
and weighing only 53.3 KiB (on `v4.2.0`).

We will also provide a [Swagger UI] route to help people understand the API and play with its routes.
For this, we will use the [utoipa-swagger-ui] crate, developped as part of [utoipa],
downloaded 1,054,711 times on [crates.io] at the time of writing this ADR
and weighing 4.17 MiB (on `v6.0.0`).

## Consequences

<!--
This section describes the resulting context, after applying the decision. All consequences should be listed here, not just the "positive" ones. A particular decision may have positive, negative, and neutral consequences, but all of them affect the team and project in the future.
-->

Providing an OpenAPI description file and a [Swagger UI] route will greatly improve
routes discoverability and API user experience. However, this comes with drawbacks.

First, we will have to maintain a separate documentation (the [utoipa] macros).
Fortunately, [utoipa] can automatically infer data from [Rocket],
which will save us from having too much duplicate documentation.

Second, this decision will add 4.22 MiB of dependencies, which will increase compile time
and attack surface. Fortunately, 98.8% of this size comes from the [utoipa-swagger-ui] crate,
which we could hide behind a [feature flag] to make it opt-in.

[crates.io]: <https://crates.io/> "crates.io: Rust Package Registry"
[feature flag]: <https://doc.rust-lang.org/cargo/reference/features.html> "Features | The Cargo Book"
[OpenAPI Initiative homepage]: <https://www.openapis.org/> "OpenAPI Initiative homepage"
[OpenAPI Specification]: <https://en.wikipedia.org/wiki/OpenAPI_Specification> "OpenAPI Specification | Wikipedia"
[OpenAPI Specification v3.1.0]: <https://spec.openapis.org/oas/v3.1.0.html> "OpenAPI Specification v3.1.0 | OpenAPI Initiative"
[Prose Pod Dashboard]: <https://github.com/prose-im/prose-pod-dashboard> "prose-im/prose-pod-dashboard | GitHub"
[Prose Pod Server]: <https://github.com/prose-im/prose-pod-server> "prose-im/prose-pod-server | GitHub"
[Rocket]: <https://rocket.rs/> "Rocket homepage"
[Swagger UI]: <https://swagger.io/tools/swagger-ui/> "Swagger UI | Swagger"
[utoipa]: <https://crates.io/crates/utoipa> "utoipa | crates.io"
[utoipa-swagger-ui]: <https://crates.io/crates/utoipa-swagger-ui> "utoipa-swagger-ui | crates.io"
