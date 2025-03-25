# ADR: Use Axum instead of Rocket

- Date: **2025-01-01**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Accepted** via [#95](https://github.com/prose-im/prose-pod-api/pull/95) (2025-01-01)
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Relates to: ø
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

Up until today, we have been using [Rocket] to power the Prose Pod API.
However, it had multiple drawbacks:

- Its lack of middlewares (see [`tower`’s documentation]) made authorization
  laborious and error-prone (which is something we really don’t want on that topic).
- Because of how errors are managed in [request guards], we had to create a [`LazyGuard`]
  (see the request guard code for more explanations). Consequently, we’d often have about
  30% of boilerplate in route definitions, just to “work around” a limitation of [Rocket].
  See [`check_dns_records_route`], which was 4 LLOCs[^lloc] long while it could have been
  a one-liner, or [`init_first_account_route`], which had 50% of boilerplate LLOCs[^lloc].
- [Observability][observability] would have been very complex to implement, if not impossible
  to apply on all routes automatically. In addition, [Rocket]’s logging wasn’t very useful
  and silently prevented tracing logs from being printed for a long time (until [`2267262`]).
- Its usage of macros would sometimes prevent auto-completion or context-sensitive
  navigation in IDEs, which would impact developer productivity.
- [Rocket] uses a `Rocket.toml` config file, which people would have had to create
  and maintain in addition to the `Prose.toml` config file.
  It would be a lot better to have all the configuration we support in `Prose.toml`.
- Release builds stayed broken for a long period because of a `#[cfg(debug_assertions)]`
  annotation on a route function parameter. To fix it, we had to duplicate the whole route,
  making more room for mistakes.
- Also, see [“Rocket is dead. (?)” on r/rust] about Rocket having a bus factor of 1[^needs].

For this reason, we searched for alternatives which would fit our needs better[^needs].
The most notable Rust HTTP server libraries we found (apart from [Rocket]) are:

- [Axum]
  - Developed by the [Tokio] team (see [“Announcing Axum” on the Tokio Blog]),
    which means it’s performant, well integrated and will be maintained.
  - Fastest full-stack web application framework (all programming languages considered),
    [according to TechEmpower][techempower-r22].
  - Takes full advantage of the [`tower`] and [`tower-http`] ecosystem of middlewares,
    services, and utilities. Also enables sharing middlewares with applications written
    using [`hyper`] or [`tonic`].
- [Actix Web]
  - Is unmaintained (see [“A sad day for Rust”] by its original author).
    It was brought back to life just after the announcement (see [`actix-web#1289`]),
    but it has been slowing down since then (see [the Contributors Insight]).
- [`warp`]
  - Seems a bit immature, and we found better alternatives.
- [`hyper`]
  - > hyper is a relatively low-level library, meant to be a building block for libraries
    > and applications.
    - We need a higher-level library.

## Decision

<!--
This section describes our response to these forces. It is stated in full
sentences, with active voice. "We will …"
-->

[Axum] being objectively the best alternative, we will rewrite the Prose Pod API using
this library. Since the core logic is decoupled from the HTTP [REST] interface, this rewrite
should only affect the `rest-api` crate. By leveraging [Axum]’s ecosystem of middlewares,
the migration should be pretty quick and go smoothly.

We put a lot of effort into writing tests first
(see [ADR: Write tests with the Gherkin syntax] and [ADR: Write integration tests]),
but now is the moment it pays off. We can rewrite the [REST] API stress-free, with the security
of having tests in place to ensure stability and non-regression.

## Consequences

<!--
This section describes the resulting context, after applying the decision.
All consequences should be listed here, not just the "positive" ones.
A particular decision may have positive, negative, and neutral consequences,
but all of them affect the team and project in the future.
-->

This migration will have a positive impact on developer productivity, code readability
and conciseness, maintainability, security, and many other aspects. It will also simplify
the addition of planned features like fine-grained authorization and [observability].

[Axum]’s macro-free API should also reduce compile times, thus improving developer experience.

Edit: After the migration, we noticed a large 32% reduction of compile time in release mode,
and a negligibly smaller binary size
(see the full benchmark in [`prose-pod-api#95` “Migrate from Rocket to Axum”][pr-95]).

On the other side, it’s important to note that:

- [Axum] doesn’t support routing by [`Accept`] header like [Rocket does][rocket-ct-routing]
  (see [`axum#1654` “Enable Routing by Content-Type Header”][axum#1654]).
  - [The workaround][axum-ct-routing] is quite simple.
- [Rocket] has a nicer API for working with [Server-Sent Events][SSE].
  - We’d have to create our own helper functions if we wanted this level of expressivity.

[`2267262`]: https://github.com/prose-im/prose-pod-api/commit/22672622cb31fbd1eb82fb460515a514e1ae9b71
[`Accept`]: https://developer.mozilla.org/docs/Web/HTTP/Headers/Accept "Accept - HTTP | MDN"
[`actix-web#1289`]: https://github.com/actix/actix-web/issues/1289 "Project future · Issue #1289 · actix/actix-web"
[`check_dns_records_route`]: https://github.com/prose-im/prose-pod-api/blob/e3e6bbba82fa0d1934990f878c1db376fc35f7d8/crates/rest-api/src/features/network_checks/check_dns_records.rs#L8-L18
[`hyper`]: https://crates.io/crates/hyper "hyper | crates.io"
[`init_first_account_route`]: https://github.com/prose-im/prose-pod-api/blob/e3e6bbba82fa0d1934990f878c1db376fc35f7d8/crates/rest-api/src/features/init/init_first_account.rs#L31-L50
[`LazyGuard`]: https://github.com/prose-im/prose-pod-api/blob/e3e6bbba82fa0d1934990f878c1db376fc35f7d8/crates/rest-api/src/guards/mod.rs#L35-L47
[`tonic`]: https://crates.io/crates/tonic "tonic | crates.io"
[`tower-http`]: https://crates.io/crates/tower-http "tower-http | crates.io"
[`tower`]: https://crates.io/crates/tower "tower | crates.io"
[`tower`’s documentation]: https://docs.rs/tower/0.5.2/tower/#overview "tower | docs.rs"
[`warp`]: https://crates.io/crates/warp "warp | crates.io"
[Actix Web]: https://crates.io/crates/actix-web "actix-web | crates.io"
[ADR: Write integration tests]: ./2024-05-15-a-integration-testing.md
[ADR: Write tests with the Gherkin syntax]: ./2024-01-11-a-write-tests-in-gherkin.md
[axum#1654]: https://github.com/tokio-rs/axum/issues/1654
[axum-ct-routing]: https://github.com/tokio-rs/axum/issues/1654#issuecomment-1454769195
[Axum]: https://crates.io/crates/axum "axum | crates.io"
[observability]: https://en.wikipedia.org/wiki/Observability_(software) "Observability (software) | Wikipedia"
[pr-95]: https://github.com/prose-im/prose-pod-api/pull/95 "Migrate from Rocket to Axum"
[request guards]: https://rocket.rs/guide/v0.5/requests/#request-guards "Requests > Request Guards - Rocket Web Framework"
[REST]: https://en.wikipedia.org/wiki/REST "REST | Wikipedia"
[rocket-ct-routing]: https://rocket.rs/guide/v0.5/requests/#format "Requests > Format | The Rocket Programming Guide"
[Rocket]: https://rocket.rs/ "Rocket homepage"
[SSE]: https://en.wikipedia.org/wiki/Server-sent_events "Server-sent events | Wikipedia"
[techempower-r22]: https://www.techempower.com/benchmarks/#hw=ph&test=fortune&section=data-r22 "Round 22 results - TechEmpower Framework Benchmarks"
[the Contributors Insight]: https://github.com/actix/actix-web/graphs/contributors?from=31/12/2022 "Contributors to actix/actix-web"
[Tokio]: https://tokio.rs/ "Tokio - An asynchronous Rust runtime"
[“A sad day for Rust”]: https://steveklabnik.com/writing/a-sad-day-for-rust "A sad day for Rust | steveklabnik.com"
[“Announcing Axum”]: https://tokio.rs/blog/2021-07-announcing-axum "Announcing Axum | Tokio - An asynchronous Rust runtime"
[“Rocket is dead. (?)” on r/rust]: https://www.reddit.com/r/rust/comments/zvvrl7/rocket_is_dead/

[^lloc]: Logical source lines of code. See [“Source lines of code” on Wikipedia](https://en.wikipedia.org/wiki/Source_lines_of_code).
[^needs]: I’m not saying Rocket is a bad framework, it really is a great one, but it doesn’t fit **our** needs _at the moment_.
