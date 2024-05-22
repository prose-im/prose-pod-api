# ADR: Use Step CI for integration testing

- Date: **2024-05-15**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Proposed**
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Relates to:
  - [Write integration tests](./2024-05-15-a-integration-testing.md) (2024-05-15)
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

Why not automatic testing from the description file? (Dredd)

- Because some routes need to be used in a very specific order (init, invite…)

[Artillery]

- All scenarios are not guaranteed to be ran, because of [probabilities](https://www.artillery.io/docs/reference/test-script#scenario-weights)
- We could create one file per scenario but that would be a real bummer and even though we could reuse the `config` section (thanks to the `--config` CLI argument) it would be highly unpractical

[Circle CI]

- Free, open-source, CLI tool
- Reusability via [`$ref`](https://docs.stepci.com/guides/testing-http.html#reusables) and [`env`](https://docs.stepci.com/reference/templating.html#objects)
- [Fake data](https://docs.stepci.com/guides/using-fake-data.html) (from [Faker](https://fakerjs.dev/api/))
- [Performance](https://docs.stepci.com/guides/performance-testing.html), [load](https://docs.stepci.com/guides/load-testing.html), [contract](https://docs.stepci.com/guides/contract-testing.html) and [fuzz](https://docs.stepci.com/guides/fuzz-testing.html) testing
  - Contract testing can reference OpenAPI definitions directly!
- Plugins if we need to
- Good docs
- Debbugging? (Like [Debugging – HTTP Engine – Artillery Docs](https://www.artillery.io/docs/reference/engines/http#debugging))

Load testing?

- On GET operations?

## Decision

<!--
This section describes our response to these forces. It is stated in full
sentences, with active voice. "We will …"
-->

<TODO:Describe>

## Consequences

<!--
This section describes the resulting context, after applying the decision.
All consequences should be listed here, not just the "positive" ones.
A particular decision may have positive, negative, and neutral consequences,
but all of them affect the team and project in the future.
-->

<TODO:Describe>

[Artillery]: https://www.artillery.io/ "Artillery homepage"
