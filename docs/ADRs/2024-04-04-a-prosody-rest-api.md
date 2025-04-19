# ADR: Interact with Prosody using a REST API

- Date: **2024-04-04**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Accepted** via [#10](https://github.com/prose-im/prose-pod-api/pull/10) (2024-04-28)
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

This REST API needs to interact with a running [Prosody] server for
at least four reasons (at the time of writing this ADR):

1. Add and remove users (when joining / leaving the workspace)
2. Hot-reload the server configuration (when changing the settings)
3. Create a user's [vCard] (to initialize their nickname when joining)
4. Test a user's password (to allow logging into the Prose Pod Dashboard)

To perform those actions, we have found different possibilities:

1. Use [`prosodyctl`], the official Prosody Command-Line Interface (CLI)
   - This CLI provides basic commands covering our first 2 use cases
     (with `register`[^1], `deluser` and `reload`)
   - Our third use case is made possible by the community plugin
     [`mod_vcard_command`], which allows getting, setting and deleting
     a user's [vCard].
     - As it is published, the plugin doesn't compile, but we have published
       a fix in [prose-pod-server#1]
   - Our fourth use case isn't supported by built-in commands,
     but we could write a custom plugin for it.
   - `prosodyctl` reads configuration files locally so in a Dockerized context
     we'd need a shared filesystem
   - `prosodyctl` reads the [PID] of the running Prosody server
     in the configuration file
     - If we have a `prosodyctl` in the Prose Pod API Docker container,
       it will not find the process as it is running in a different container
     - We can't access the `prosodyctl` CLI installed in the Prose Pod Server
       container, for security reasons
       - Maybe we could, using a `docker exec`, but that seems very
         inefficient and insecure
2. Use the bundled [Admin Console][Console] ([mod_admin_telnet])
   - This plugin starts a [telnet] console allowing one to communicate
     with a running Prosody instance
     - Because there is no authentication, this console blocks all connections
       from the outside, even when using a Docker network and exposed ports
   - There is only a fixed set of instructions
     (documented in [prosody.im/doc/console][Console]),
     which does not contain the ability to set a user's [vCard]
     - The ["Advanced usage" section of the Console documentation] says
       we could access server internals at runtime by prefixing a line
       with the `>` character. However, we found no server internal allowing us
       to set a user's vCard without reimplementing all of the logic present in
       `datamanager.lua`, `mod_vcard_command.lua` and the rest of `core`
       and `util` libraries. Reimplementing all of this would be highly
       inefficient, error-prone and against the idea of modularity in Prosody.
3. Expose a REST API via a Prosody plugin
   - [wltsmrz/mod_admin_rest] is a RESTful admin interface to Prosody,
     supporting operations like adding and removing users, changing passwords,
     etc. It does not support setting user vCards but by integrating
     the logic from [`mod_vcard_command`] in `mod_admin_rest.lua`
     we would have all the features we need.
   - This REST API is accessible via the [standard Prosody HTTP port]
     (`5280/tcp` by default). It uses [Basic authentication] and
     [Prosody permissions] to restrict access to Prosody administrators.

## Decision

<!--
This section describes our response to these forces. It is stated in full
sentences, with active voice. "We will …"
-->

Since the [Admin Console][Console] does not solve all our needs and we cannot
use `prosodyctl` from another Docker container, we will use the third option
(a REST API as a Prosody plugin). [wltsmrz/mod_admin_rest] does not support
all of our requirements, so we will fork it and add routes as needed.

We will use [the `reqwest` crate] to perform HTTP calls from the Prose Pod API
to the Prose Pod Server, as it is lightweight (based on [`hyper`])
and well-known in the Rust community. This choice is not definitive,
we could choose this HTTP client for another one if we find a better fit.

[wltsmrz/mod_admin_rest] requires [Basic authentication] using
an XMPP account, but the Prose Pod API does not save user passwords anywhere
(everything is handled by the XMPP server — Prosody as of today).
Since we wouldn't want the Prose Pod API to use [Basic authentication]
on every route (for security reasons), we will create an XMPP account
for the Prose Pod API. It will be created automatically at Prosody's launch
using `module:hook_global("server-started", …);` in a custom plugin[^2].

The XMPP account used by the Prose Pod API will be given the `prosody:admin`
role, and its password will be shared via an environment variable to both
the Prose Pod API and the Prose Pod Server. For higher security we could change
this password on every deployment of [prose-pod-system], to make sure no one
can find it even in a `.env` file. Out of simplicity, we will make it
user-defined for now, and improve the process later if we need to.

## Consequences

<!--
This section describes the resulting context, after applying the decision.
All consequences should be listed here, not just the "positive" ones.
A particular decision may have positive, negative, and neutral consequences,
but all of them affect the team and project in the future.
-->

We could not find a built-in or community-maintained solution answering
all of our needs, therefore we will have to maintain a fork of
[wltsmrz/mod_admin_rest].

In terms of security, this solution seems to be the safest, as it adds a layer
of authentication which other solutions don't have. Even if this internal port
happened to be accessed from outside of the Docker network, it would still
require one to have the `prosody:admin` role to do anything harmful.

Having an XMPP account for the Prose Pod API means we don't NEED the user to be
authenticated in order to make a call to the Prose Pod Server REST API.
Because of this, we could introduce a [privilege escalation] flaw if we don't
pay attention. We MUST design our [request guards] in such way that it becomes
impossible to make a call to the Prose Pod Server REST API if the source
Prose Pod API request is not authenticated by an admin (as defined by Prose).

We will create an XMPP account for the Prose Pod API, for internal purposes.
However, this account is likely to appear when listing users in the XMPP server,
which is undesired. We might be able to go around it by defining a scoped
[virtual host] in the Prosody configuration and using it for our internal usage,
but we will solve this consequent issue later.

[^1]: The `register` command is hidden from the `prosodyctl` command listing by default but exists as a compatibility with `ejabberdctl`. It acts like `adduser JID` but allows passing the password as an argument instead of reading user input (which is annoying to automate).
[^2]: A working example can be found at <https://github.com/prose-im/prose-pod-server/commit/3b54d071880dff669f0193a8068733b089936751>.

["Advanced usage" section of the Console documentation]: https://prosody.im/doc/console#advanced_usage "Console > Advanced usage – Prosody IM"
[Basic authentication]: https://en.wikipedia.org/wiki/Basic_access_authentication "Basic access authentication - Wikipedia"
[Console]: https://prosody.im/doc/console "Console – Prosody IM"
[mod_admin_telnet]: https://prosody.im/doc/modules/mod_admin_telnet "mod_admin_telnet – Prosody IM"
[`hyper`]: https://crates.io/crates/hyper "hyper - crates.io"
[PID]: https://en.wikipedia.org/wiki/Process_identifier "Process identifier - Wikipedia"
[privilege escalation]: https://en.wikipedia.org/wiki/Privilege_escalation "Privilege escalation - Wikipedia"
[prose-pod-server#1]: https://github.com/prose-im/prose-pod-server/pull/1 "feat: Add `mod_vcard_command`"
[prose-pod-system]: https://github.com/prose-im/prose-pod-system "prose-im/prose-pod-system: Prose Pod system configurations and build rules. Used to package everything together."
[Prosody permissions]: https://prosody.im/doc/developers/permissions "Roles and permissions – Prosody IM"
[Prosody]: https://prosody.im "Welcome – Prosody IM"
[`prosodyctl`]: https://prosody.im/doc/prosodyctl "prosodyctl – Prosody IM"
[request guards]: https://rocket.rs/guide/v0.5/requests/#request-guards "Requests > Request Guards - Rocket Web Framework"
[standard Prosody HTTP port]: https://prosody.im/doc/ports#default-ports "Port and network configuration – Prosody IM"
[telnet]: https://wikipedia.org/wiki/Telnet "Telnet - Wikipedia"
[the `reqwest` crate]: https://crates.io/crates/reqwest "reqwest - crates.io"
[vCard]: https://www.rfc-editor.org/rfc/rfc6350 "RFC 6350: vCard Format Specification"
[virtual host]: https://prosody.im/doc/configure#adding_a_host "Configuring Prosody > Adding a host – Prosody IM"
[wltsmrz/mod_admin_rest]: https://github.com/wltsmrz/mod_admin_rest "wltsmrz/mod_admin_rest: RESTful admin interface to Prosody XMPP server."
