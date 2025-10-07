@rotating-service-passwords @startup-actions
Feature: Rotating service account passwords

  # When the Prose Pod Server API was introduced, it became responsible for
  # rotating service account passwords.
  Rule: The Workspace account password is not rotated

    Scenario: At API startup
      Given config "server.domain" is set to "test.org"
        And the Prose Pod has been initialized
       When the Prose Pod API starts
       Then <prose-workspace@test.org>'s password is not changed
