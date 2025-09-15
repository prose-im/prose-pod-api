# ADR: Stop exposing the OpenAPI docs

- Date: **2025-09-13**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Accepted** via [#316](https://github.com/prose-im/prose-pod-api/pull/316) (2025-09-11)
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Relates to:
  - [Describe Prose Pod API using the OpenAPI Specification](./2023-12-18-a-describe-with-openapi.md) (2023-12-18)
  - [Write the OpenAPI description file manually](./2024-04-25-a-write-openapi-manually.md) (2024-04-25)
  - [Expose API documentation route using Redoc (instead of Swagger UI)](./2024-04-25-b-use-redoc-instead-of-swagger-ui.md) (2024-04-25)
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

Since the very creation of the Prose Pod API, an OpenAPI description and graphical
documentation of the HTTP API has been exposed publicly at `/api-docs/openapi.json` and
`/api-docs/redoc` (historically `/api-docs/swagger-ui`) respectively.
This comes from historical reasons: at the very beginning of this project, the OpenAPI
description was automatically generated at build time. One could not see any documentation for
their running version of the Prose Pod API unless it exposed it itself at run time.

Now things are different. We got rid of this process and started writing the OpenAPI description
manually (see [ADR: Write the OpenAPI description file manually][2024-04-25-a] (2024-04-25)).
At the time we made this transition, we also switched from Swagger UI to [Redoc], for reasons
explained in [ADR: Expose API documentation route using Redoc (instead of Swagger UI)][2024-04-25-b]
(2024-04-25).
While the OpenAPI description is hand-written, it is split into multiple files to make it easier
to work with. Because we didn’t find a better tool around and were already using [Redoc], we
decided to use the [Redocly CLI] to generate the merged OpenAPI description file.

On 2025-09-09, our CI broke because [Redocly] (the company behind Redoc) had released the
version 2 of the Redocly CLI (see [prose-pod-api#309]), with breaking changes.
After initially just forcing the use of v1 in our CI, I decided to try to use the latest version
but realized the Redocly CLI now seems to be a paid product.
It’s nowhere near being crucial to our project and it’s not something we use to make any money
so I decided we should just stick to the v1 as long as it’s available.

This is what sparked this decision, but the real reason behind it is different.
Indeed, exposing the OpenAPI description and graphical documentation of the Prose Pod API
benefits the very few people who will ever try integrating the Prose Pod API with something
else, but also a ton of people with bad intentions.
In reality, describing all routes of an HTTP API publicly like that makes the task so much
easier for someone looking for flaws.
Now that it’s not _required_ anymore, we should get rid of those routes.

## Decision

<!--
This section describes our response to these forces. It is stated in full
sentences, with active voice. "We will …"
-->

We will stop exposing all `/api-docs/*` routes in public releases (even when `debug_assertions`
are enabled). Because of how integration tests work, locally-built images will still have to
expose the OpenAPI description file. We will hide this route behind a
`cfg(all(debug_assertions, feature = "openapi"))` and generate the OpenAPI description only in `local-run/Dockerfile` so it’s not even in `edge` images.

Note that if a company _really_ needs the OpenAPI description file to be exposed, we could
always make it opt-in via a configuration flag. Maybe even add authentication if needed.
But until then we’ll get rid of it entirely.

## Consequences

<!--
This section describes the resulting context, after applying the decision.
All consequences should be listed here, not just the "positive" ones.
A particular decision may have positive, negative, and neutral consequences,
but all of them affect the team and project in the future.
-->

Removing those routes will:

- Make it harder to reverse-engineer the Prose Pod API, the security corner stone of a
  Prose Pod. While it won’t make it impossible, at least automated tools won’t easily find and
  analyse the OpenAPI description.
- Remove ~900kB of JS + ~200kB of JSON OpenAPI description + ~600B of HTML from the
  `prose-pod-api` image (total: ~1.1MB), which is always a good thing!
  - FYI, the image is ~16MB at the moment
- Reduce Rust compile time (less routes + no more use of `tower_http::services::ServeDir`)
- Reduce CI run time and remove potential unexpected CI failures (no installation of the
  Redocly CLI means faster runs and no unexpected breaking changes)

Developing clients for the Prose Pod API will be a little more cumbersome, as one won’t have the
comfort of just opening `/api-docs/redoc`. It will now require them to get the commit hash using
`GET /version`, then check the repository out and run `task openapi:preview-docs`.
Note that this last command will require [Task] and the [Redocly CLI] to be installed locally.
While it is a negative consequence, this cost is very small compared to the security impact of
exposing the docs publicly.
In addition, along with this ADR I proposed
[ADR: Maintain TypeScript and Rust Prose Pod API SDKs][2025-09-13-b]
to completely counter this downside.

[2024-04-25-a]: ./2024-04-25-a-write-openapi-manually.md
[2024-04-25-b]: ./2024-04-25-b-use-redoc-instead-of-swagger-ui.md
[2025-09-13-b]: ./2025-09-13-b-prose-pod-api-sdks.md
[prose-pod-api#309]: https://github.com/prose-im/prose-pod-api/issues/309 "Migrate `openapi:*` tasks to Redocly CLI v2 · Issue #309 · prose-im/prose-pod-api"
[Redoc]: https://redocly.com/redoc/ "Redoc homepage"
[Redocly]: https://redocly.com/ "Redocly homepage"
[Redocly CLI]: https://redocly.com/docs/cli "Redocly CLI docs"
[Task]: https://taskfile.dev/ "Task homepage"
