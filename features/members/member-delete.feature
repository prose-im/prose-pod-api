@member-delete @members
Feature: Deleting a member

  Background:
    Given config "server.domain" is set to "prose.org"
      And the Prose Pod has been initialized
      And the Prose Pod API has started

  Rule: Member accounts can be deleted

    Scenario: Valerian (admin) deletes Rémi’s account
      Given Valerian is an admin
       When Valerian deletes remi@prose.org’s account
       Then the HTTP status code should be NoContent

  Rule: Only admins can delete member accounts

    Scenario: Rémi (not admin) deletes Marc’s account
      Given Rémi is not an admin
       When Rémi deletes marc@prose.org’s account
       Then the HTTP status code should be Forbidden
        And the response content type should be JSON

  """
  1. It doesn't make sense
  2. If the only admin removes themselves, the workspace is locked

  See <https://github.com/prose-im/prose-pod-api/issues/140>.
  """
  Rule: Members cannot delete their own account

    Scenario: Valerian (admin) deletes Rémi’s account
      Given Valerian is an admin
       When Valerian deletes Valerian’s account
       Then the HTTP status code should be Forbidden
