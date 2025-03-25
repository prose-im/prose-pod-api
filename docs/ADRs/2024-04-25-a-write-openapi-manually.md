# ADR: Write the OpenAPI description file manually

- Date: **2024-04-25**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Accepted** via [#13](https://github.com/prose-im/prose-pod-api/pull/13) (2024-04-28)
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Relates to:
  - [Describe Prose Pod API using the OpenAPI Specification](./2023-12-18-a-describe-with-openapi.md) (2023-12-18)
  - [Automatically detect OpenAPI routes](./2023-12-18-b-generate-openapi-description.md) (2023-12-18)
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Superseded by: ø
<!-- "No" or "Yes" with the deprecation date -->
- Deprecated: No

## Context

<!--
This section describes the forces at play, including technological, political,
social, and project local. These forces are probably in tension, and should be
called out as such. The language in this section is value-neutral. It is simply
describing facts.
-->

In [Automatically detect OpenAPI routes](./2023-12-18-b-generate-openapi-description.md)
we decided to use [`utoipa`] to generate the OpenAPI description file instead of
writing it manually. However, it did not integrate well with [Rocket]'s
uncommon way of defining routes. For example, to define a route which accepts
multiple content types, we could not use `utoipa` macros and had to build
the definition ourselves using `utoipa`'s API (see [openapi_extensions.rs]).

We also realized that the output of `utoipa` wasn't good at all. Some models
were defined but never used, some were missing and we get no warning,
values using `FromStr` for parsing instead of `serde` were not understood
automatically… among other problems.

In addition, we wouldn't even be able to hide `utoipa` behind a feature flag
because of a bug in `cfg_attr` (see [rust-lang/rust-analyzer#13360] and
[ProbablyClem/utoipauto#13]).

At the time of writing this ADR, Prose Pod API is still in early stages
and we already see these issues appear. It will only get worse and we will
spend too much time "fixing" `utoipa`'s output and learning its syntax.
In the end, it might make more sense to spend all of this time learning the
OpenAPI Specification (a well-known standard) and writing a qualitative
API documentation.

[Rocket] has built-in tools for serving static files (see [`FileServer`] and
[`NamedFile`]) so it would be very easy to have the OpenAPI description file
stored in the repository.

## Decision

<!--
This section describes our response to these forces. It is stated in full
sentences, with active voice. "We will …"
-->

We will copy the current output of [`utoipa`], clean it, fix it and then
get rid of [`utoipa`] altogether.

Instead of reflecting the current state of the API, this manual OpenAPI
description file will now be used as a "contract" defining how routes SHOULD
behave even if some are not fully implemented. This is why specifications like
this exist in the first place and it will allow other people to start building
on top of the API (like building the Prose Pod Dashboard) while it's
in development.

## Consequences

<!--
This section describes the resulting context, after applying the decision.
All consequences should be listed here, not just the "positive" ones.
A particular decision may have positive, negative, and neutral consequences,
but all of them affect the team and project in the future.
-->

Writing and maintaining the OpenAPI description file manually will directly
require a lot of time and efforts but at the same time remove the indirect work
needed to maintain the `utoipa`-generated file.

Since the schemas won't be generated automatically and the documentation won't
lie next to the code anymore, it's inevitable that the OpenAPI description file
will get out-of-sync with the API. We have to see it as an opportunity to focus
on quality and write a better documentation.

[`FileServer`]: https://api.rocket.rs/v0.5/rocket/fs/struct.FileServer "FileServer in rocket::fs - Rust"
[`NamedFile`]: https://api.rocket.rs/v0.5/rocket/fs/struct.NamedFile "NamedFile in rocket::fs - Rust"
[openapi_extensions.rs]: https://github.com/prose-im/prose-pod-api/blob/3f65904a3bd22d5b7d8f7065b76f21e6b8b1e6b4/src/v1/workspace/openapi_extensions.rs
[ProbablyClem/utoipauto#13]: https://github.com/ProbablyClem/utoipauto/issues/13 "Detecting `ToSchema` fails when behind a `cfg_attr` flag · Issue #13 · ProbablyClem/utoipauto"
[Rocket]: https://rocket.rs/ "Rocket homepage"
[rust-lang/rust-analyzer#13360]: https://github.com/rust-lang/rust-analyzer/issues/13360 "proc_macro attribue fails to expand when combined with `cfg_attr` · Issue #13360 · rust-lang/rust-analyzer"
[`utoipa`]: https://crates.io/crates/utoipa "utoipa | crates.io"
