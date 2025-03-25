# ADR: Store workspace data in a vCard

- Date: **2024-07-14**
- Author: **Rémi Bardon <[remi@remibardon.name](mailto:remi@remibardon.name)>**
<!-- Proposed|Accepted|Rejected, with date and channel if applicable -->
- Status: **Accepted** in a call (2024-06-29)
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

The Prose Pod API is the interface allowing one (e.g. the Prose Pod Dashboard) to configure a Prose Pod. It stores some information, like user roles and invitations, but most of the data lives in the XMPP server. To allow one to use any XMPP client with a Prose Pod (following the essence of XMPP), all the data needed by a Prose client must be served by the XMPP server. A Prose client application shouldn't need to discuss with a Prose Pod API.

Through the Prose Pod Dashboard, a workspace administrator can set and change the workspace name, which should be reflected in Prose client applications. Therefore, the workspace name must be stored by the XMPP server. As far as I know, XMPP has no concept of workspace, so the closest we could get would be to configure the vCard[^prose-vcard-xep] of the host itself (a bare JID with no local part).

Although PEP and pubsub both support addressing and subscribing to a host JID[^pep-host-jid][^pubsub-host-jid], it's not common for both clients or servers to support it. In addition, we would probably hit plenty of roadblocks as a lot of Prosody plugins expect JIDs with local parts.

## Decision

<!--
This section describes our response to these forces. It is stated in full
sentences, with active voice. "We will …"
-->

Out of simplicity, we will create an XMPP account for the workspace and name it `prose-workspace@<host>` (where `<host>` is the XMPP server's fully qualified domain name). This account will not be added to every member's roster, in order not to appear in their contacts. It will have a very strong random password only the Prose Pod API knows, to ensure no one logs into this account.

In the future, we *might* get rid of this account in favor of the host JID, and maybe even standardize it, but it will need to be thoroughly tested to ensure every part of Prose (and its underlying parts) supports it.

## Consequences

<!--
This section describes the resulting context, after applying the decision.
All consequences should be listed here, not just the "positive" ones.
A particular decision may have positive, negative, and neutral consequences,
but all of them affect the team and project in the future.
-->

The Prose Pod API won't store details about the workspace anymore, so retrieving data like the workspace name won't be as simple as making a database request. It will require sending an XMPP stanza, parsing it and possibly not finding the required information.

Similarly, if we decide to change the name of the workspace account in the future, we will have to setup a migration logic to read data from the old account and create a new one. Even if this data was stored in database, we would need a migration process to delete the old account so this decision doesn't have a huge impact.

Since the workspace will have an XMPP account on the server, listing all users will show this account. However, XMPP users won't see it as they list members using their contacts list (roster). In addition, we will have "bot" accounts in the future and it will cause the same result so it wouldn't be that surprising to see the workspace account listed among them.

[^prose-vcard-xep]: Prose supports both [XEP-0292: vCard4 Over XMPP](https://xmpp.org/extensions/xep-0292.html) and [XEP-0054: vcard-temp](https://xmpp.org/extensions/xep-0054.html).
[^pep-host-jid]: See [XEP-0163: Personal Eventing Protocol > 2.1 Every Account a Pubsub Service](https://xmpp.org/extensions/xep-0163.html#approach-everyjid).
[^pubsub-host-jid]: See [XEP-0060: Publish-Subscribe > 4.6 Addressing](https://xmpp.org/extensions/xep-0060.html#addressing).
