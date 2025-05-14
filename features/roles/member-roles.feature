@member-roles @roles
Feature: Member roles

  Background:
    Given the Prose Pod has been initialized
      And Valerian is an admin
      And the Prose Pod API has started

  Rule: Admins can change member roles

    Scenario: Valerian (admin) makes Marc an admin
      Given Marc is a regular member
       When Valerian makes Marc an admin
       Then the HTTP status code should be OK
        And Marc should have the ADMIN role
        And Marc should have the "prosody:admin" role in Prosody

    Scenario: Valerian (admin) makes Marc an admin while it already is one
      Given Marc is an admin
       When Valerian makes Marc an admin
       Then the HTTP status code should be OK
        And Marc should have the ADMIN role
        And Marc should have the "prosody:admin" role in Prosody

  Rule: One cannot downgrade someone with a higher role

    Scenario: Rémi (not admin) makes Valerian a regular member
      Given Rémi is a regular member
        And Valerian is an admin
       When Rémi makes Valerian a regular member
       Then the HTTP status code should be Forbidden
        And Valerian should have the ADMIN role
        And Valerian should have the "prosody:admin" role in Prosody

  """
  An admin cannot downgrade themselves as a regular member.
  If they could, the API could end up having no admin and the API would be locked up.
  """
  Rule: One cannot downgrade their own role

    Scenario: Valerian (admin) makes Valerian a regular member
      Given Valerian is an admin
       When Valerian makes Valerian a regular member
       Then the HTTP status code should be Forbidden
        And Valerian should have the ADMIN role
        And Valerian should have the "prosody:admin" role in Prosody
