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
       When Valerian sends an invitation to "remi@prose.org"
       Then the call should succeed
        And the HTTP status code should be OK

  """
  For security reasons, we don't want members to invite other people.
  Therefore only members with the ADMIN role should be allowed to do it.
  """
  Rule: Only admins can invite new members

    Scenario: Rémi (not admin) invites Marc
      Given Rémi is not an admin
       When Valerian sends an invitation to "marc@prose.org"
       Then the call should not succeed
        And the response content type should be JSON
        And the HTTP status code should be Unauthorized
        And the response should contain a "WWW-Authenticate" HTTP header

  """
  In the Prose Pod Dashboard, admins should be able to see
  pending invitations.
  """
  Rule: Admins can list invites

  """
  Admins should be able to see the status of an invite
  (to act if needed).
  """
  Rule: Admins can see the status of an invite

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
  A invite can never find its recipient (e.g. typo in email address,
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

  """
  If the admin made a mistake in the email address for example,
  a random person might receive the invite.
  It is their right to reject it and make sure they will never
  receive a follow-up email.
  """
  Rule: Someone invited by mistake can reject an invite

  """
  Access logs already store this kind of operation,
  there is no need to clutter the database with such data.
  Also, someone rejecting an invite probably doesn't want
  their email address staying around.
  """
  Rule: An invite disappears after it's rejected
