# ADR: Use `prose-xmpp` and `mod_rest` to send stanzas to Prosody

- Date: **2024-06-05**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Accepted** via [#19](https://github.com/prose-im/prose-pod-api/pull/19) (2024-06-29)
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Relates to:
  - [Enrich member data in a separate HTTP API call](./2024-05-27-a-lazily-enriching-member-data.md) (2024-05-27)
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

As explained in [ADR: Enrich member data in a separate HTTP API call](./2024-05-27-a-lazily-enriching-member-data.md), the Prose Pod API needs to send [stanzas][stanza] to Prosody to retrieve data about members (e.g. nicknames, avatars, presence…). Since it doesn't have an XMPP account that's in every member's roster (and also to prevent escalation of privileges), the Prose Pod API needs to send stanzas "as" the logged in user. For obvious security reasons, it's not as easy as changing the "from" property of an IQ stanza: the Prose Pod API needs to be authorized by the user.

The most straighforward solution would be to open an XMPP connection for every user who logs into the Prose Pod Dashboard, however it would be both greatly inefficient and cumbersome to maintain. To help in situations like this, the Prosody community has created the [`mod_rest`] module, which exposes a simple HTTP RESTful API for sending and receiving stanzas. To authenticate users, [`mod_rest`] uses the official but undocumented [`mod_tokenauth`] module (which provides token management for use by other modules). [`mod_tokenauth`] doesn't expose a way for the Prose Pod API to generate tokens, the [`mod_http_oauth2`] community module is the only module which provides such functionality.

To generate a token using [`mod_http_oauth2`], one solution is to make an HTTP call with the user's bare JID and password as Basic authentication. However, the Prose Pod API doesn't store user passwords (and will never do so), which means it has to generate the token when the user logs in and keep it somewhere. While it's not as critical as an account password because Prosody's HTTP ports are not exposed outside of the containers' environment, we should still avoid storing it in the Prose Pod API's database (better safe than sorry).

Finally, the Prose Pod API needs to know how to serialize XMPP stanzas and which ones to send to get the desired results. While we could use the [`xmpp-parsers`] crate and implement every feature we need in `prose-pod-api`, we'd end up rewriting most of the logic that's in [`prose-core-client`]. Luckily, [`prose-core-client`] was developed in such a way tha all of the logic we need is exposed by the [`prose-xmpp`] crate. By implementing its `Connector` and `Connection` traits to send stanzas via [`mod_rest`] instead of a regular XMPP connection, we'd be able to make the XMPP queries we need in a completely transparent way.

## Decision

<!--
This section describes our response to these forces. It is stated in full
sentences, with active voice. "We will …"
-->

To send stanzas to Prosody, we will enable [`mod_rest`] and [`mod_http_oauth2`] in Prosody. When a user logs in, we will use their credentials with [`mod_http_oauth2`] to get a token we'd use to authenticate calls to [`mod_rest`]. We will save this token in the [JWT] returned by the Prose Pod API when a user logs in. Since this [JWT] doesn't need to be read by API clients, it will be encrypted to prevent one from "stealing" a Prosody token. Finally, to construct and send stanzas we will use the [`prose-xmpp`] crate, which abstracts away all of the XMPP logic the Prose Pod API doesn't need to know about.

## Consequences

<!--
This section describes the resulting context, after applying the decision.
All consequences should be listed here, not just the "positive" ones.
A particular decision may have positive, negative, and neutral consequences,
but all of them affect the team and project in the future.
-->

Since [`mod_rest`] and [`mod_http_oauth2`] are maintained by the Prosody community and [`prose-xmpp`] is maintained as part of the Prose app, once implemented this setup won't require any maintenance in `prose-pod-api`. It will automatically benefit from updates and fixes in all of its dependencies, and ensure a full compatibility with the Prose app at any time[^compat].

As always, adding dependencies to a project increases the attack surface and binary size of the program but the said dependencies are targetted on a single use-case so we could hardly do better "by hand".

Finally, one downside of using tokens to authenticate calls to Prosody is that they might expire before the Prose Pod API token ([JWT]) does, resulting in a situation where the API user has to log in again (given this situation is correctly handled by the Prose Pod API). Fortunately, Prose Pod API tokens are critical and thus have a very short lifetime. This situation will therefore never happen.

[JWT]: https://jwt.io/ "JSON Web Tokens - jwt.io"
[`mod_http_oauth2`]: https://modules.prosody.im/mod_http_oauth2 "mod_http_oauth2 - Prosody Community Modules"
[`mod_rest`]: https://modules.prosody.im/mod_rest "mod_rest - Prosody Community Modules"
[`mod_tokenauth`]: https://hg.prosody.im/0.12/file/997d9ad12477/plugins/mod_tokenauth.lua "Prosody IM 0.12: 997d9ad12477 plugins/mod_tokenauth.lua"
[`prose-core-client`]: https://github.com/prose-im/prose-core-client "prose-im/prose-core-client: Prose core XMPP client manager & protocols."
[`prose-xmpp`]: https://github.com/prose-im/prose-core-client/tree/master/crates/prose-xmpp "prose-core-client/crates/prose-xmpp at master · prose-im/prose-core-client"
[`xmpp-parsers`]: https://docs.rs/xmpp-parsers/0.20.0/xmpp_parsers/ "xmpp_parsers - Rust"
[stanza]: https://www.rfc-editor.org/rfc/rfc6120.html#section-1.3 "RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core - Section 1.3. (Functional Summary)"

[^compat]: For example, there won't be a moment where the app uses [XEP-0292: vCard4 Over XMPP](https://xmpp.org/extensions/xep-0292.html) and the Prose Pod API uses [XEP-0054: vcard-temp](https://xmpp.org/extensions/xep-0054.html).
