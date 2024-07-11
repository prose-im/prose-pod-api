# ADR: Enrich member data in a separate HTTP API call

- Date: **2024-05-27**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Accepted** via [#19](https://github.com/prose-im/prose-pod-api/pull/19) (2024-06-29)
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Relates to:
  - [Interact with Prosody using a REST API](./2024-04-04-a-prosody-rest-api.md) (2024-04-04)
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

The Prose Pod API is used to add members to the workspace. When joining, people have to choose a nickname, and the Prose Pod API is responsible for initializing their XMPP profile (vCard). However, once they have access to their account, they are able to edit their XMPP profile from any client software (it's the very essence of XMPP). For this reason, the Prose Pod API isn't a source of truth and thus it can't return information like member nicknames and avatars when listing members (based on the information it has internally).

One solution would be to interact with the XMPP server to retrieve this data when listing members. However, doing so would increase the response time and cost of the request while the API client might not need the additional information. In addition, unexpected cases like the user not having a vCard or Prose Pod API not being able to communicate with the XMPP server would also have to be taken into consideration, and this would complicate the route.

[@valeriansaliou](https://github.com/valeriansaliou) proposed that we create a separate route which allows one to lazily "enrich" a member's data. This way we'd have one blazing fast route for retrieving simple data and a slower one for retrieving only the additional data we need.

To enrich a member's [JID], we have to communicate with the XMPP server, and this can be done two ways. At the time of writing this ADR, the Prose Pod API only interacts with Prosody (the XMPP server) [using a REST API](./2024-04-04-a-prosody-rest-api.md "ADR: Interact with Prosody using a REST API") able to access Prosody's internals. Naturally, it is also possible to send [stanzas] to the XMPP server. To enrich a member's [JID] using the former method, we'd have to implement a [REST] route for every action we'd need (e.g. get the nickname, get the avatar, get the presence…), which is very time-consuming and error-prone amongst many other negative aspects. It would be a lot better to send stanzas directly, as we'd make sure it behaves exactly like when using any XMPP client.

## Decision

<!--
This section describes our response to these forces. It is stated in full
sentences, with active voice. "We will …"
-->

The Prose Pod API will keep its "List members" route as is (returning only the data stored in its database) and we will create a separate route for lazily loading more information about members. This route will send stanzas to the XMPP server to retrieve the data.

To avoid flooding the Prose Pod API with HTTP requests, the route will accept `1..n` [JID]s. The inherent asynchronous nature of this route makes it a good candidate for using [server-sent events (SSE)][SSE]. While the XMPP server should be able to handle a large number of requests at once, using [SSE]s will ensure the API client doesn't hang while waiting for the HTTP response. All of the enriched data should arrive very quickly but streaming allows for a better [user experience][UX] on resource-constrained devices (the enriched data contains every member's avatar in [Base64]).

While it's not efficient in terms of memory usage for both the client and the HTTP API server, the Prose Pod API will by default return the results as a single JSON object. Developers are more used to it and it will be coherent with the rest of the API.

## Consequences

<!--
This section describes the resulting context, after applying the decision.
All consequences should be listed here, not just the "positive" ones.
A particular decision may have positive, negative, and neutral consequences,
but all of them affect the team and project in the future.
-->

The Prose Pod API isn't yet able to send stanzas to the XMPP server, which means implementing this "enriching" feature will require substantial changes to the API's internal structure. However, once that done correctly, it will allow us to implement all sorts of advanced features in Prose Pod API so it's a worth investment.

[Rocket] —the web framework we use for the Prose Pod API— already supports [SSE]s natively[^sse-rocket] which means it won't require a lot of work to develop the streaming route. Both the non-streaming and the streaming routes will share the same logic therefore by factoring it we will greatly reduce the maintenance cost of the streaming counterpart[^streaming-vs-non-streaming].

On the API clients' end, it will be a little more work to display a rich list of members but we believe any sufficiently well developed software should be handle to make and consume the two sequential HTTP calls without any trouble.

It is not the case yet, but we might want to continue streaming enriched data updates as long as the client displays it. For example, if a member was offline by the time its data was enriched but it turns online while a user of the Prose Pod Dashboard is looking at the members list, we might want to change the "presence dot" color. By allowing the Prose Pod API to subscribe to changes and not closing the [server-sent events] stream on the server's side, we could easily implement such feature without breaking API clients.

[Base64]: https://en.wikipedia.org/wiki/Base64 "Base64 | Wikipedia"
[JID]: https://datatracker.ietf.org/doc/html/rfc7622 "RFC 7622 - Extensible Messaging and Presence Protocol (XMPP): Address Format"
[LLOC]: https://en.wikipedia.org/wiki/Source_lines_of_code "Source lines of code | Wikipedia"
[REST]: https://en.wikipedia.org/wiki/REST "REST | Wikipedia"
[Rocket]: https://rocket.rs "Rocket homepage"
[SSE]: https://en.wikipedia.org/wiki/Server-sent_events "Server-sent events | Wikipedia"
[UX]: https://en.wikipedia.org/wiki/User_experience "User experience | Wikipedia"
[stanzas]: https://www.rfc-editor.org/rfc/rfc6120.html#section-1.3 "RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core - Section 1.3. (Functional Summary)"

[^sse-rocket]: See [Responses > Async Streams - Rocket Web Framework](https://rocket.rs/guide/v0.5/responses/#async-streams) and [`EventStream` in `rocket::response::stream` - Rust](https://api.rocket.rs/v0.5/rocket/response/stream/struct.EventStream).
[^streaming-vs-non-streaming]: See [prose-pod-api/src/v1/members/routes.rs L58-L95](https://github.com/prose-im/prose-pod-api/blob/bea93054334974ed3008ebe9deedf095aef549bf/src/v1/members/routes.rs#L58-L95), by factoring the logic in `MemberController::enrich_member`, both the non-streaming and the streaming routes takes 5 logical lines of code ([LLOC]).
