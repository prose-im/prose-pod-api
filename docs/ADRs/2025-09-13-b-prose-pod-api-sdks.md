# ADR: Maintain TypeScript and Rust Prose Pod API SDKs

- Date: **2025-09-13**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Proposed**
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Relates to:
  - [ADR: Write tests with the Gherkin syntax](./2024-01-11-a-write-tests-in-gherkin.md) (2024-01-11)
  - [ADR: Stop exposing the OpenAPI docs](./2025-09-13-a-stop-exposing-openapi-docs.md) (2025-09-13)
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

Currently, the Prose Pod API is used by two projects: the Prose Pod Dashboard and the
Prose Cloud API. The former is written in TypeScript, while the latter is written in Rust.
While I — Rémi Bardon, maintainer of the Prose Pod API — have the opportunity to contribute
to those projects (and already have), I must mention that both of them are developed by
different people. Consequently, every change/addition to the Prose Pod API means at least
two people will have to understand it and implement the client code for it.

In addition, said client implementations can easily get outdated and break silently,
given they have no automated testing ensuring every route works as expected.
This brittleness means changes are harder to make in the Prose Pod API and require a lot of
planning.

Finally, now that [ADR: Stop exposing the OpenAPI docs][2025-09-13-a] has been implemented,
it will make the situation even a bit worse as Prose Pod API client developers won’t have the
comfort of having the `/api-docs/redoc` route.
This means they will have to clone the `prose-pod-api` repository, install [Task] and the
[Redocly CLI], check the appropriate tag/commit out and then finally run
`task openapi:preview-docs` to see the visual documentation.
All steps won’t be necessary every time, but it’s a lot more time consuming than what the
previous process offered (not to mention the need to install tools).

## Decision

<!--
This section describes our response to these forces. It is stated in full
sentences, with active voice. "We will …"
-->

We will internalize the TypeScript and Rust client code here in the `prose-pod-api` repository,
making it available in the form of SDKs. I highly recommend watching the talk
[Building HTTP API SDKs that Really Are a Kit • Darrel Miller • GOTO 2019][sdk-talk]
before starting work on said SDKs.

While we might not do it from the very beginning by lack of time, we will set integration tests
up for those SDKs. Fortunately, we wrote most of the API tests in a language-agnostic manner
(see [ADR: Write tests with the Gherkin syntax][2024-01-11-a]), which means we wouldn’t need
three separate integration test suites. That old decision will soon pay off!

## Consequences

<!--
This section describes the resulting context, after applying the decision.
All consequences should be listed here, not just the "positive" ones.
A particular decision may have positive, negative, and neutral consequences,
but all of them affect the team and project in the future.
-->

Most notably, integrating new features in the Prose Pod Dashboard will be easier than ever.
We will have the opportunity to make changes to the Prose Pod API while being sure no client is
silently broken, and no addition/change will ever feel annoying to integrate in client apps.

On the flip side, developing features in `prose-pod-api` will be slightly slower, as we will
have to update at least the TypeScript SDK (the Rust SDK only needs a small subset of the
Pod API’s features, until someone else will want to use it). However this small cost is by far
smaller to the time someone else would spend integrating the additions/changes.

It is worth mentionning that Prose Pod API developers will need to know about TypeScript, but
this doesn’t seem to be a notable requirement. In addition, once the TypeScript SDK will be
scaffolded correctly, making changes to it will likely only be a matter of copy-pasting some
code and/or making a few tweaks to it.

One possible downside is that the proposed SDKs might not integrate as well as a custom solution
might have, but since we have existing code to look at the possibility to have feedback easily
I don’t think it’s too much of an issue.
Also, if said SDKs are well designed, it shouldn’t cause problems.

[2024-01-11-a]: ./2024-01-11-a-write-tests-in-gherkin.md
[2025-09-13-a]: ./2025-09-13-a-stop-exposing-openapi-docs.md
[Task]: https://taskfile.dev/ "Task homepage"
[Redocly CLI]: https://redocly.com/docs/cli "Redocly CLI docs"
[sdk-talk]: https://youtu.be/dzaVtAZBnsA?feature=shared "“Building HTTP API SDKs that Really Are a Kit • Darrel Miller • GOTO 2019” on YouTube"
