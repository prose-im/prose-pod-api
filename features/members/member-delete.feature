@member-delete @members
Feature: Deleting a member

  Background:
    Given the Prose Pod has been initialized
      And the XMPP server domain is prose.org
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
