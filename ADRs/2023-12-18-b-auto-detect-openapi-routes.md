# ADR: Automatically detect OpenAPI routes

- Date: **2023-12-18**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Proposed**
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Relates to:
  - [Describe Prose Pod API using the OpenAPI Specification](./2023-12-18-a-describe-with-openapi.md) (2023-12-18)
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Superseded by: ø
<!-- "No" or "Yes" with the deprecation date -->
- Deprecated: No

## Context

<!--
This section describes the forces at play, including technological, political, social, and project local. These forces are probably in tension, and should be called out as such. The language in this section is value-neutral. It is simply describing facts.
-->

The [Rocket] web framework [requires us to maintain an exhaustive list of all routes][rocket-routes-list].
The [utoipa] crate, which we use to generate an [OpenAPI Specification], requires us to maintain
another exhaustive list. This will inevitably lead to desynchronization and thus incoherence
between the code and the [OpenAPI Specification].

## Decision

<!--
This section describes our response to these forces. It is stated in full sentences, with active voice. "We will …"
-->

We will use the [utoipauto] crate,
downloaded 1,418 times on [crates.io] at the time of writing this ADR
and weighing only 9.08 KiB (on `v0.1.8`).

This crate is in its very early stages, with multiple new releases in the past few weeks.
However, it already has the features we need so we can use it without worrying about breaking changes.

## Consequences

<!--
This section describes the resulting context, after applying the decision. All consequences should be listed here, not just the "positive" ones. A particular decision may have positive, negative, and neutral consequences, but all of them affect the team and project in the future.
-->

Using [utoipauto] will allow us to not have to maintain a second exhaustive list of routes,
which will increase productivity and avoid inconsistencies.

As always when adding dependencies, this decision will increase compile time and attack surface.
Fortunately, the [utoipauto] crate is very small so it is negligeable.

Finally, as the crate is very recent, we cannot be sure that it will be maintained in the long run.
However, it isn't a dependency of the API core, so in case we have to get rid of it
for some reason in the future, we can always do it and list routes manually.

[crates.io]: <https://crates.io/> "crates.io: Rust Package Registry"
[OpenAPI Specification]: <https://en.wikipedia.org/wiki/OpenAPI_Specification> "OpenAPI Specification | Wikipedia"
[Rocket]: <https://rocket.rs/> "Rocket homepage"
[rocket-routes-list]: <https://rocket.rs/v0.5/guide/overview/#mounting> "Overview - Rocket Programming Guide"
[utoipa]: <https://crates.io/crates/utoipa> "utoipa | crates.io"
[utoipauto]: <https://crates.io/crates/utoipauto> "utoipauto | crates.io"
