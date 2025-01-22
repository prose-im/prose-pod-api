@nicknames @profiles
Feature: Member nicknames

  Background:
    Given the Prose Pod has been initialized
      And the Prose Pod API has started

  Rule: One can change their own nickname

    Scenario: Rémi changes their nickname
      Given Rémi is a regular member
        And Rémi’s nickname is "Rémi"
       When Rémi sets their nickname to "Rémi B."
       Then the call should succeed
        And Rémi’s nickname should be "Rémi B."

  Rule: One cannot change someone else’s nickname

    Scenario: An admin tries to change Rémi’s nickname
      Given Rémi is a regular member
        And Valerian is an admin
        And Rémi’s nickname is "Rémi"
       When Valerian sets Rémi’s nickname to "Rémi B."
       Then the HTTP status code should be Forbidden
        And Rémi’s nickname should be "Rémi"

    Scenario: A regular member tries to change Rémi’s nickname
      Given Rémi is a regular member
        And Marc is a regular member
        And Rémi’s nickname is "Rémi"
       When Marc sets Rémi’s nickname to "Rémi B."
       Then the HTTP status code should be Forbidden
        And Rémi’s nickname should be "Rémi"
