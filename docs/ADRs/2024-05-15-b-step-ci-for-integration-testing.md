# ADR: Use Step CI for integration testing

- Date: **2024-05-15**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Accepted** via [#18](https://github.com/prose-im/prose-pod-api/pull/18) (2024-05-30)
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

[ADR: Write integration tests] introduced our plan to write integration tests, and detailed what kind of tool we will need. This ADR explores some options and decides the one we will use.

### Artillery

[Artillery] is a complete load testing platform. It is open-source and provides a very attractive solution for testing APIs. However, it is not designed to be an exhaustive testing tool. Indeed, scenarios defined in a test script are not guaranteed to be ran, as Artillery uses [probabilities](https://www.artillery.io/docs/reference/test-script#scenario-weights) to choose which scenario to run. It is similar to what a fleet of users would do, but doesn't correspond to our use case.

We *could* create one file per scenario but that would be a real bummer and even though we could reuse the `config` section (thanks to the `--config` CLI argument) it would be highly unpractical.

### Step CI

[Step CI] is a free and open-source CLI tool which uses a syntax very similar to [Artillery]'s but runs every test once. It provides:

- Reusability via [`$ref`](https://docs.stepci.com/guides/testing-http.html#reusables) and [`env`](https://docs.stepci.com/reference/templating.html#objects)
- [Fake data](https://docs.stepci.com/guides/using-fake-data.html) (from [Faker](https://fakerjs.dev/api/))
- [Performance](https://docs.stepci.com/guides/performance-testing.html), [load](https://docs.stepci.com/guides/load-testing.html), [contract](https://docs.stepci.com/guides/contract-testing.html) and [fuzz](https://docs.stepci.com/guides/fuzz-testing.html) testing
  - Contract testing can reference OpenAPI definitions directly!
- Plugins if we need to
- Good enough docs
- Debugging logs, only when a test fails (unlike [Artillery's verbose debugging mode](https://www.artillery.io/docs/reference/engines/http#debugging "Debugging – HTTP Engine – Artillery Docs"))

Step CI seems less mature than Artillery but if it covers our needs and allows us to implement what is missing ourselves (open-source runner + custom plugins), then it seems like a great solution.

> NOTE: We've already submitted [stepci/runner#117](https://github.com/stepci/runner/pull/117 "Fix: Render liquidless templates everywhere (not just in `tests`)") and [stepci/runner#118](https://github.com/stepci/runner/pull/118 "Add `before` and `after` sections") to fix a bug and add functionalities we were missing (which Artillery supported).

## Decision

<!--
This section describes our response to these forces. It is stated in full
sentences, with active voice. "We will …"
-->

> NOTE: See [ADR: Write integration tests] for general decisions about integration testing.

We will use [Step CI] to write integration tests, covering the most important routes.
Also, every time we encounter an integration issue, we will write at least one integration test to prevent regressions.

Since Step CI test files contain no particular information mentioning the name of the tool, we will put tests in a directory named `step-ci` under `tests/integration` for discoverability.

## Consequences

<!--
This section describes the resulting context, after applying the decision.
All consequences should be listed here, not just the "positive" ones.
A particular decision may have positive, negative, and neutral consequences,
but all of them affect the team and project in the future.
-->

Most consequences of integration testing are listed in [ADR: Write integration tests] but using Step CI instead of another tool will have additional consequences:

- Because it is still immature, we will run into bugs and suffer from missing features which will require efforts to work around or fix/implement.

However:

- If we want to do some load testing one day, we won't have to use yet another tool as Step CI supports it.
- Step CI's nice debug logs will help us fix issues quicker.
- Step CI's reusability features will help us write tests faster.

[ADR: Write integration tests]: ./2024-05-15-a-integration-testing.md "ADR: Write integration tests"
[Artillery]: https://www.artillery.io/ "Artillery homepage"
[Step CI]: https://stepci.com/ "Step CI homepage"
