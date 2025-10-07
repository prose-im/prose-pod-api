@invitations
Feature: Inviting members

  Background:
    Given config "server.domain" is set to "prose.org"
      And the Prose Pod has been initialized
      And the Prose Pod API has started

  """
  The intended way of inviting a new member to a Prose server
  is by sending an email to their professional email address.
  """
  Rule: New members can be invited via email

    Scenario: Valerian (admin) invites Rémi
      Given Valerian is an admin
       When Valerian invites <remi@prose.org> as a MEMBER
       Then the HTTP status code should be Created
        And the response should contain a "Location" HTTP header
        And 1 email should have been sent
        And the email body should match "/invitations/accept/[a-zA-Z0-9]{32}"
        And the email body should match "/invitations/reject/[a-zA-Z0-9]{32}"

  """
  For security reasons, we don't want members to invite other people.
  Therefore only members with the ADMIN role should be allowed to do it.
  """
  Rule: Only admins can invite new members

    Scenario: Rémi (not admin) invites Marc
      Given Rémi is not an admin
       When Rémi invites <marc@prose.org> as a MEMBER
       Then the HTTP status code should be Forbidden
        And the response content type should be JSON

  """
  In the Prose Pod Dashboard, admins should be able to see
  pending invitations.
  """
  Rule: Admins can list invitations

    Scenario: Small number of invitations
      Given <marc@prose.org> has been invited via email
        And <remi@prose.org> has been invited via email
        And Valerian is an admin
       When Valerian lists pending invitations
       Then the HTTP status code should be OK
        And the response content type should be JSON
        And they should see 2 pending invitations

    Scenario: Large number of invitations, first page
      Given 42 people have been invited via email
        And Valerian is an admin
       When Valerian lists pending invitations by pages of 20
       Then the HTTP status code should be Partial Content
        And they should see 20 pending invitations
        And the "Pagination-Current-Page" header should contain "1"
        And the "Pagination-Page-Size" header should contain "20"
        And the "Pagination-Page-Count" header should contain "3"

    Scenario: Large number of invitations, last page
      Given 42 people have been invited via email
        And Valerian is an admin
       When Valerian gets page 3 of pending invitations by pages of 20
       Then the HTTP status code should be OK
        And they should see 2 pending invitations
        And the "Pagination-Current-Page" header should contain "3"
        And the "Pagination-Page-Size" header should contain "20"
        And the "Pagination-Page-Count" header should contain "3"

  """
  Admins should be able to see when an invitation has been created
  (to spot languishing ones for example).
  """
  Rule: Admins can see the creation timestamp of an invitation

  """
  For many reasons (e.g. a typo in the email address),
  admins should be able to cancel invitations.
  """
  Rule: Admins can cancel an invitation

    Scenario: Valerian (admin) cancels an invitation
      Given <marc@prose.org> has been invited via email
        And Valerian is an admin
       When Valerian cancels the invitation
       Then the HTTP status code should be No Content
        And there should not be any invitation for <marc@prose.org> in the database

    Scenario: Rémi (not admin) tries to cancel an invitation
      Given <marc@prose.org> has been invited via email
        And Rémi is not an admin
       When Rémi cancels the invitation
       Then the HTTP status code should be Forbidden
        And there should be an invitation for <marc@prose.org> in the database

  """
  Instead of having to wait for a member to accept their invitation
  before being able to assign them a role, admins should be able to
  pre-assign a role to an invited member.
  """
  Rule: An invited member can be pre-assigned a role upon invitation

    Scenario: Rémi is invited to be an admin
      Given <remi@prose.org> has been invited via email
        And <remi@prose.org> is pre-assigned the ADMIN role
       When <remi@prose.org> accepts their invitation
       Then <remi@prose.org> should have the ADMIN role

  """
  At the time of writing this test, only two roles exist: ADMIN and MEMBER.
  As admins are the only ones who can invite new members, we won't
  be able to test a failing scenario.
  However, with this rule a security mechanism should be implemented
  forcing a developer to break the test as soon as a higher role is created.
  """
  Rule: An admin can only pre-assign a role lower or equal to theirs

  """
  For security reasons, invitations should expire after some time.
  """
  Rule: Invitations expire after 3 days by default

    Scenario: Rémi did not accept the invitation in time (he always does this)
      Given <remi@prose.org> has been invited via email
        And the invitation has already expired
       When <remi@prose.org> accepts their invitation
       Then the HTTP status code should be Gone
        And the error code should be "invitation_not_found"

  """
  By default, an invitation is valid for 3 days.
  Admins should be able to override this setting when inviting someone.
  """
  Rule: Admins can choose the lifetime of an invitation upon creation

  """
  Because invitations expire after some time or they can never find their recipient
  (e.g. typo in email address, email server down…), we must provide a way
  to resend them.
  """
  Rule: An admin can resend an invitation

    Scenario: Valerian (admin) resends an invitation
      Given <marc@prose.org> has been invited via email
        And Valerian is an admin
       When Valerian resends the invitation
       Then the HTTP status code should be No Content
        And 1 email should have been sent

    Scenario: Rémi (not admin) tries to resend an invitation
      Given <marc@prose.org> has been invited via email
        And Rémi is not an admin
       When Rémi resends the invitation
       Then the HTTP status code should be Forbidden
        And 0 email should have been sent

  """
  For security reasons, if an invitation is sent again, the previous accpet link
  becomes useless.
  """
  Rule: After resending an invitation, the previous accept token becomes invalid

    Scenario: Rémi has been invited twice but uses the first link
      Given <remi@prose.org> has been invited via email
        And an admin resent the invitation
       When <remi@prose.org> uses the previous invitation accept link they received
       Then the HTTP status code should be Gone
        And the error code should be "invitation_not_found"

  """
  Access logs already store this kind of operation,
  there is no need to clutter the database with such data.
  Also, the email address used for contacting the user
  might not be a professional email address, and therefore
  should be treated as a sensitive information.
  """
  Rule: An invitation disappears after it's accepted

    Scenario: Rémi accepts an invitation
      Given <remi@personal.name> has been invited via email
       When <remi@personal.name> accepts their invitation
       Then there should not be any invitation for <remi@personal.name> in the database

  """
  If the admin made a mistake in the email address for example,
  a random person might receive the invitation.
  It is their right to reject it and make sure they will never
  receive a follow-up email.
  """
  Rule: Someone invited by mistake can reject an invitation

    Scenario: Rémi rejects an invitation
      Given <remi@personal.name> has been invited via email
       When <remi@personal.name> rejects their invitation
       Then the HTTP status code should be No Content

  """
  Access logs already store this kind of operation,
  there is no need to clutter the database with such data.
  Also, someone rejecting an invitation probably doesn't want
  their email address staying around.
  """
  Rule: An invitation disappears after it's rejected

    Scenario: Rémi rejects an invitation
      Given <remi@personal.name> has been invited via email
       When <remi@personal.name> rejects their invitation
       Then the call should succeed
        And there should not be any invitation for <remi@personal.name> in the database

  """
  The invitation accept and reject links look like `/invitations/(accept|reject)/{uuid}`,
  with the invide ID not directly accessible. However, to follow the HTTP standard,
  the Prose Pod API requires the full path to the invitation resource, which includes its ID.
  Therefore, the Prose Pod Dashboard needs a way to retrieve the invitation ID in order to
  make the REST API call.

  The answer should not contain sensitive information such as the invitation creator,
  creation timestamp and accept/reject tokens, as the route needs to be publicly accessible
  (the member is joining and doesn't have a JID and role yet). Since accept/reject tokens
  are never shared outside of invitation notifications, someone with a token should be allowed
  to see this basic information, as it proves they have been invited.

  In the result we can share the accept token expiration timestamp, so it can be displayed
  in the Prose Pod Dashboard if we wanted to (e.g. "This invites expires in {countdown}").
  """
  Rule: Basic info about an invitation can be retrieved from the accept and reject tokens

    Scenario: Retrieving from an accept token
      Given <remi@prose.org> has been invited via email
       When <remi@prose.org> requests the invitation associated to their accept token
       Then the call should succeed
        And the response content type should be JSON

    Scenario: Retrieving from a reject token
      Given <remi@prose.org> has been invited via email
       When <remi@prose.org> requests the invitation associated to their reject token
       Then the call should succeed
        And the response content type should be JSON

    Scenario: Retrieving from an old accept token
      Given <remi@prose.org> has been invited via email
        And an admin resent the invitation
       When <remi@prose.org> requests the invitation associated to their previous accept token
       Then the HTTP status code should be Gone
        And the error code should be "invitation_not_found"

  Rule: Invited members can choose their nickname when joining

    Scenario: Rémi joins using a custom nickname
      Given <remi@prose.org> has been invited via email
          # NOTE: config "server.domain" is set to "prose.org"
       When <remi@prose.org> accepts their invitation using the nickname "Rémi B."
       Then the call should succeed
        And remi@prose.org’s nickname should be "Rémi B."
