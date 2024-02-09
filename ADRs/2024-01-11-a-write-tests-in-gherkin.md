# ADR: Write tests with the Gherkin syntax

- Date: **2024-01-11**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Proposed**
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

Writing tests is an important part of the long-term development of Prose Pod API.
For this reason, we must make it easy to add new tests, especially ones for complex scenarios.
Usually, those tests are long to write and very hard to read later.

To reduce the amount of similar code reused in all test definitions, it is a common practice to use
[the xUnit architecture], which [Rust] does not follow natively.

## Decision

<!--
This section describes our response to these forces. It is stated in full sentences, with active voice. "We will …"
-->

To make tests easier to read, we will use [the Gherkin syntax].
It allows us to write tests in natural language, improving a ton how easy it is to understand
what a test does and express complex scenarios.

To run tests from their Gherkin definition, we will use [Cucumber step definitions],
which is the intended way of running Gherkin.

We will use the [cucumber] crate,
downloaded 1,057,583 times on [crates.io] at the time of writing this ADR
and weighing 129 KiB (on `v0.20.2`).
It is developped by the Cucumber team, which means it is continuously updated to align
with the Gherkin and Cucumber specifications.

For more information about Gherkin and Cucumber, please read ["Introduction" on the Cucumber Documentation].
Also, the [`Background` keyword] might be worth checking out, as it could remove some redundancies
in the feature files.

## Consequences

<!--
This section describes the resulting context, after applying the decision. All consequences should be listed here, not just the "positive" ones. A particular decision may have positive, negative, and neutral consequences, but all of them affect the team and project in the future.
-->

Writing tests in natural language has a lot of advantages. First, it makes tests easy to read by anyone,
and act as an additional documentation garanteed to be up-to-date.
It also makes tests implementation-agnostic, allowing easy migrations if we decide to change
the implementation of Prose Pod API.
In addition, once [Cucumber step definitions][step definitions] have been written,
it is very easy to add more tests and reuse steps from other scenarios.

However, this comes with important challenges.
Mainly, it requires the extra step of writing [step definitions], which slows us down in the short term.
In addition, it can be hard to implement those [step definitions] as state is shared between steps
through a single [World] implementation.
Because of concurrency, this implementation needs to be thread-safe, which adds ever more overhead.
However, this ensures that tests are safe and makes it easier to write correct tests.

[`Background` keyword]: <https://cucumber-rs.github.io/cucumber/current/writing/background.html> "Background keyword - Cucumber Rust Book"
[crates.io]: <https://crates.io/> "crates.io: Rust Package Registry"
[cucumber]: <https://crates.io/crates/cucumber> "cucumber | crates.io"
[step definitions]: <https://cucumber.io/docs/cucumber/step-definitions/> "Step Definitions - Cucumber Documentation"
["Introduction" on the Cucumber Documentation]: <https://cucumber.io/docs/guides/overview/> "Introduction - Cucumber Documentation"
[Rust]: <https://www.rust-lang.org/> "Rust Programming Language homepage"
[the Gherkin syntax]: <https://cucumber.io/docs/gherkin/reference/> "Gherkin Reference - Cucumber Documentation"
[the xUnit architecture]: <https://en.wikipedia.org/wiki/XUnit> "xUnit | Wikipedia"
[World]: <https://docs.rs/cucumber/0.20.2/cucumber/trait.World.html> "World in cucumber - Rust"
