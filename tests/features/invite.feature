@testing
Feature: Inviting members

  Background:
    Given the workspace has been initialized

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

  """
  For security reasons, we don't want members to invite other people.
  Therefore only members with the ADMIN role should be allowed to do it.
  """
  Rule: Only admins can invite new members

    Scenario: Rémi (not admin) invites Marc
      Given Rémi is not an admin
       When Rémi invites <marc@prose.org> as a MEMBER
       Then the HTTP status code should be Unauthorized
        And the response content type should be JSON
        And the response should contain a "WWW-Authenticate" HTTP header

  """
  In the Prose Pod Dashboard, admins should be able to see
  pending invitations.
  """
  Rule: Admins can list invites

    Scenario: Small number of invites
      Given <marc@prose.org> has been invited via email
        And <remi@prose.org> has been invited via email
        And Valerian is an admin
       When Valerian lists pending invitations
       Then the HTTP status code should be OK
        And the response content type should be JSON
        And they should see 2 pending invitations

    Scenario: Large number of invites, first page
      Given 42 people have been invited via email
        And Valerian is an admin
       When Valerian lists pending invitations by pages of 20
       Then the HTTP status code should be Partial Content
        And they should see 20 pending invitations
        And the "Pagination-Current-Page" header should contain "1"
        And the "Pagination-Page-Size" header should contain "20"
        And the "Pagination-Page-Count" header should contain "3"

    Scenario: Large number of invites, last page
      Given 42 people have been invited via email
        And Valerian is an admin
       When Valerian gets page 3 of pending invitations by pages of 20
       Then the HTTP status code should be OK
        And they should see 2 pending invitations
        And the "Pagination-Current-Page" header should contain "3"
        And the "Pagination-Page-Size" header should contain "20"
        And the "Pagination-Page-Count" header should contain "3"

  """
  Admins should be able to see the status of an invite
  (to act if needed).
  """
  Rule: Admins can see the status of an invite

    Scenario: Small number of invites
      Given <marc@prose.org> has been invited via email
        And <marc@prose.org> has received their invitation
        And <remi@prose.org> has been invited via email
        And Valerian is an admin
       When Valerian lists pending invitations
       Then the call should succeed
        And 1 invitation should be TO_SEND
        And 1 invitation should be RECEIVED

  """
  Admins should be able to see when an invite has been created
  (to spot languishing ones for example).
  """
  Rule: Admins can see the creation timestamp of an invite

  """
  For security reasons (e.g. a typo in the email address),
  admins should be able to cancel invites.
  """
  Rule: Admins can cancel an invitation

  """
  Instead of having to wait for a member to accept their invitation
  before being able to assign them a role, admins should be able to
  pre-assign a role to an invited member.
  """
  Rule: An invited member can be pre-assigned a role upon invitation

  """
  At the time of writing this test, only two roles exist: ADMIN and MEMBER.
  As admins are the only ones who can invite new members, we won't
  be able to test a failing scenario.
  However, with this rule a security mechanism should be implemented
  forcing a developer to break the test as soon as a higher role is created.
  """
  Rule: An admin can only pre-assign a role lower or equal to theirs

  """
  An invite can never find its recipient (e.g. typo in email address,
  email server down…), therefore we must provide a way to resend it
  once issues have been solved.
  """
  Rule: If the invite did not go through, an admin can resend it

  """
  Access logs already store this kind of operation,
  there is no need to clutter the database with such data.
  Also, the email address used for contacting the user
  might not be a professional email address, and therefore
  should be treated as a sensitive information.
  """
  Rule: An invite disappears after it's accepted

    Scenario: Rémi accpets an invitation
      Given <remi@personal.name> has been invited via email
       When <remi@personal.name> accepts their invitation
       Then there should not be any invitation for <remi@personal.name> in the database

  """
  If the admin made a mistake in the email address for example,
  a random person might receive the invite.
  It is their right to reject it and make sure they will never
  receive a follow-up email.
  """
  Rule: Someone invited by mistake can reject an invite

    Scenario: Rémi rejects an invitation
      Given <remi@personal.name> has been invited via email
       When <remi@personal.name> rejects their invitation
       Then the HTTP status code should be No Content

  """
  Access logs already store this kind of operation,
  there is no need to clutter the database with such data.
  Also, someone rejecting an invite probably doesn't want
  their email address staying around.
  """
  Rule: An invite disappears after it's rejected

    Scenario: Rémi rejects an invitation
      Given <remi@personal.name> has been invited via email
       When <remi@personal.name> rejects their invitation
       Then there should not be any invitation for <remi@personal.name> in the database
