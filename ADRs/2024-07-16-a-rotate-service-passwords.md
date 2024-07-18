# ADR: Rotate service accounts passwords at every startup

- Date: **2024-07-16**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Accepted** via [#44](https://github.com/prose-im/prose-pod-api/pull/44) (2024-07-19)
<!-- "ø" or a nested unordered list linking to other ADRs and their date -->
- Relates to:
  - [Store workspace data in a vCard](./2024-07-14-a-store-workspace-data-in-xmpp-vcard.md) (2024-07-14)
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

At the time of writing this ADR, the Prose Pod API needs two XMPP accounts to work properly. Those "service" accounts are made to be accessed only by the Prose Pod API and no other party should ever log into it.

The first account is the "super admin" account, `prose-pod-api@admin.prose.org.local`, which is used to make administration changes to the Prose Pod Server (e.g. reloading the server configuration or creating users). Because it needs to exist before the Prose Pod API can even send a request to the Prose Pod Servern, this account is created automatically by the XMPP server when it starts, and the password must be known by both parties. We use environment variables to share this secret information, but a malicious Prosody plugin or just a bad code logging environment variables could expose critical credentials.

The second account is the workspace account, `prose-workspace@<host>`, as detailed in [ADR: Store workspace data in a vCard](./2024-07-14-a-store-workspace-data-in-xmpp-vcard.md). This account is created by the Prose Pod API and has a random password. The Prose Pod API doesn't store the password anywhere, for obvious security reasons, therefore it cannot recover it after a restart. Since the "super admin" account can create XMPP users and change their passwords, a solution is to override the workspace account password with a new random one the Prose Pod API will once again store in memory.

This ADR is there to document this decision but also generalize the process to the "super admin" account and every "service" account we will create in the future.

## Decision

<!--
This section describes our response to these forces. It is stated in full
sentences, with active voice. "We will …"
-->

To ensure no one can log into XMPP accounts made to be used only by the Prose Pod API (to prevent [privilege escalation]), we will rotate the passwords of "service" accounts every time the Prose Pod API starts. The "super admin" account password will become a "bootstrapping password", used for a few seconds to allow the Prose Pod API to send a first request to the Prose Pod Server. asking it to change its own password.

By locking passwords access during this procedure, we *could* rotate all passwords more often if we feel the need. It's a very fast operation so we could transparently do it very often with virtually no impact for API users.

## Consequences

<!--
This section describes the resulting context, after applying the decision.
All consequences should be listed here, not just the "positive" ones.
A particular decision may have positive, negative, and neutral consequences,
but all of them affect the team and project in the future.
-->

Since passwords are random, not predefined nor stored, we won't be able to run two instances of the same Prose Pod API at the same time to scale horizontally. It's not something we plan on doing therefore this is not an issue.

Other than that, the password rotation is completely transparent so apart from preventing unwanted access I don't see more consequences.

[privilege escalation]: https://en.wikipedia.org/wiki/Privilege_escalation "Privilege escalation | Wikipedia"
