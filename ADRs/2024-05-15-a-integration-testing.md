# ADR: Write integration tests

- Date: **2024-05-15**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Accepted** via [#18](https://github.com/prose-im/prose-pod-api/pull/18) (2024-05-30)
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

### Detect integration issues

The Prose Pod API is covered with unit tests [written in Gherkin](./2024-01-11-a-write-tests-in-gherkin.md).
However, those tests use [mocks](https://en.wikipedia.org/wiki/Mock_object) and therefore some integration issues can go into production.
To avoid this, we have to write integration tests which start the Docker containers.

We don't need to test *everything*, since it should already be covered in unit tests.
We only need to make sure the Prose Pod starts correctly and the most important routes work as expected.

We *could* reuse some BDD tests written in Gherkin to run both unit and integration tests but it would take a lot of efforts to setup so it's better if we use an existing tool made specifically for integration testing.
Such tool could also be used for load testing, not because the API will be under a lot of pressure but just to detect mishandled race conditions.

### Make sure the API conforms to its description file

To simplify the maintenance of unit tests, we use strongly typed Rust data structures to create the HTTP requests, and especially their bodies.
This is guaranteed to work with the API as [serde] will do both serialization and deserialization, but it also means we are not testing how data structures are encoded and if they conform to their API description.

One solution would be to use a tool like [Dredd] to automatically validate [the API description file] against the API implementation.
However, a lot of routes (the most critical ones in addition) need to be used in a very specific order (think about the initialization process or workspace invitations for example), and an automatic tool wouldn't be able to understand this.

Therefore, we need to use an API testing framework which allow us to write sequences of HTTP requests and perform checks on the response.

## Decision

<!--
This section describes our response to these forces. It is stated in full
sentences, with active voice. "We will …"
-->

We will write integration tests which will:

- Start the Prose Pod Server and the Prose Pod API
- Initialize the Prose Pod using the Prose Pod API
- Send HTTP requests to Prose Pod API's most important routes and assert that the response conforms to [the API description file]

Those integration tests will be in the `prose-pod-api` repository for now, as it will make it easier to run a CI which detects issues.
One could argue that it should be in `prose-pod-system` and run on `prose-pod-server` changes, we'll do it later if we feel the need.

## Consequences

<!--
This section describes the resulting context, after applying the decision.
All consequences should be listed here, not just the "positive" ones.
A particular decision may have positive, negative, and neutral consequences,
but all of them affect the team and project in the future.
-->

Since we won't reuse BDD tests, we will have to maintain another set of tests, written in another non-standard format.
They will require the installation of at least one additional tool, which may not be maintained in the future.

[Dredd]: https://dredd.org/en/latest/index.html "Dredd homepage"
[serde]: https://serde.rs/ "serde homepage"
[the API description file]: ./2023-12-18-a-describe-with-openapi.md "ADR: Describe Prose Pod API using the OpenAPI Specification"
