# ADR: Use SeaORM to interact with the SQLite database

- Date: **2023-12-21**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Accepted** in person (2023-12-22)
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Relates to: ø
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Superseded by: ø
<!-- "No" or "Yes" with the deprecation date -->
- Deprecated: No

## Context

<!--
This section describes the forces at play, including technological, political, social, and project local. These forces are probably in tension, and should be called out as such. The language in this section is value-neutral. It is simply describing facts.
-->

Prose Pod API uses a SQLite database to store data, for example the server settings.
To avoid SQL mistakes and simplify development, it had been decided to use an [ORM].

To avoid regressions, catch errors sooner and because it's generally a good pratice,
we would like this project to contain [tests].
Therefore, we need an [ORM] which provides test tooling.

At first, we started developing Prose Pod API using the [Diesel] [ORM].
However, we quickly hit a wall when trying to implement testing as the library
did not provide any tooling in that regard.

## Decision

<!--
This section describes our response to these forces. It is stated in full sentences, with active voice. "We will …"
-->

We will use [SeaORM] which [provides helpers for writing tests][sea-orm-testing],
in addition to supporting `async`, migrations and other important features.

For information about how [SeaORM] compares to [Diesel],
see ["Compare with Diesel" on the SeaORM Docs][compare-with-diesel].

We will use the same structure as [SeaORM]'s [Rocket with SeaORM example app][rocket-example]
since it is the way the library is intended to be used.

## Consequences

<!--
This section describes the resulting context, after applying the decision. All consequences should be listed here, not just the "positive" ones. A particular decision may have positive, negative, and neutral consequences, but all of them affect the team and project in the future.
-->

In addition to the issues it solves, [SeaORM] has helpers for [Rocket].
It is also worth mentioning that [SeaORM] has [a good documentation][sea-orm-docs] with built-in search.

As always when adding dependencies, this decision will increase compile time and attack surface.
The [sea-orm] crate has been downloaded 2,080,859 times on [crates.io] at the time of writing this ADR
and has [frequent updates][sea-orm-releases] which is reassuring in that regard.
The crate weighs 746 KiB (on `v0.12.12`), which is quite a lot, but it's also a central piece of the API
and all the features it provides (`async`, migrations, testing…) explain this weight.

Finally, using a new [ORM] unknown to the team is not a light decision, but since it's the beginning
of the project, this decision is acceptable.
We will learn how to use this crate as we build Prose Pod API and document what we learn along the way.

[compare-with-diesel]: <https://www.sea-ql.org/SeaORM/docs/internal-design/diesel/> "Compare with Diesel | SeaORM"
[crates.io]: <https://crates.io/> "crates.io: Rust Package Registry"
[Diesel]: <https://diesel.rs/> "Diesel homepage"
[ORM]: <https://en.wikipedia.org/wiki/Object%E2%80%93relational_mapping> "Object–relational mapping | Wikipedia"
[Rocket]: <https://rocket.rs/> "Rocket homepage"
[rocket-example]: <https://github.com/SeaQL/sea-orm/tree/131f9f11230b7fd2d318870fd0c2d441c80ed734/examples/rocket_example> "SeaQL/sea-orm/examples/rocket_example | GitHub"
[SeaORM]: <https://www.sea-ql.org/SeaORM/> "SeaORM homepage"
[sea-orm]: <https://crates.io/crates/sea-orm> "sea-orm | crates.io"
[sea-orm-docs]: <https://www.sea-ql.org/SeaORM/docs/index/> "Docs | SeaORM"
[sea-orm-releases]: <https://github.com/SeaQL/sea-orm/releases> "Releases · SeaQL/sea-orm"
[sea-orm-testing]: <https://www.sea-ql.org/SeaORM/docs/write-test/mock/> "Mock Interface | SeaORM"
[tests]: <https://en.wikipedia.org/wiki/Software_testing> "Software testing | Wikipedia"
