# ADR: Interact with Prosody using `prosodyctl`

- Date: **2024-04-04**
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

This REST API needs to interact with a running [Prosody] server for at least three reasons (at the time of writing this ADR):

1. Add and remove users (when joining / leaving the workspace)
2. Hot-reload the server configuration (when changing the settings)
3. Create a user's [vCard] (to initialize their nickname when joining)

To perform those actions, we have found different possibilities:

1. Use [`prosodyctl`], the official Prosody Command-Line Interface (CLI)
   - This CLI provides basic commands covering our first 2 use cases (with `register`[^1], `deluser` and `reload`)
   - Our third use case is made possible by the community plugin [`mod_vcard_command`], which allows getting, setting and deleting a user's [vCard].
     - As it is published, the plugin doesn't compile, but we have published a fix in [prose-pod-server#1]
   - `prosodyctl` reads configuration files locally so in a Dockerized context we'd need a shared filesystem
2. Use the bundled [Admin Console][Console] ([`mod_admin_telnet`])
   - This plugin starts a [telnet] console allowing one to communicate with a running Prosody instance
     - Because there is no authentication, security must be thoroughly thought about
   - There is only a fixed set of instructions (documented in [prosody.im/doc/console][Console]), which does not contain the ability to set a user's [vCard]
3. Develop our own Prosody plugin

## Decision

<!--
This section describes our response to these forces. It is stated in full sentences, with active voice. "We will …"
-->

Since the [Admin Console][Console] does not solve all our needs and existing `prosodyctl` community plugins do, we will use `prosodyctl`.

TODO

## Consequences

<!--
This section describes the resulting context, after applying the decision. All consequences should be listed here, not just the "positive" ones. A particular decision may have positive, negative, and neutral consequences, but all of them affect the team and project in the future.
-->

TODO

[^1]: The `register` command is hidden from the `prosodyctl` command listing by default but exists as a compatibility with `ejabberdctl`. It acts like `adduser JID` but allows passing the password as an argument instead of reading user input (which is annoying to automate).

[Console]: https://prosody.im/doc/console "Console – Prosody IM"
[`mod_admin_telnet`]: https://prosody.im/doc/modules/mod_admin_telnet "mod_admin_telnet – Prosody IM"
[prose-pod-server#1]: https://github.com/prose-im/prose-pod-server/pull/1 "feat: Add `mod_vcard_command`"
[Prosody]: https://prosody.im
[`prosodyctl`]: https://prosody.im/doc/prosodyctl "prosodyctl – Prosody IM"
[telnet]: https://wikipedia.org/wiki/Telnet "Telnet - Wikipedia"
[vCard]: https://www.rfc-editor.org/rfc/rfc6350 "RFC 6350: vCard Format Specification"
