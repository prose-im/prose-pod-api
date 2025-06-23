@rotating-service-passwords @startup-actions
Feature: Rotating service account passwords

  Rule: The "super admin" account password is rotated

    Scenario: At API startup
       When the Prose Pod API starts
       Then <prose-pod-api@admin.prose.local>'s password is changed

  Rule: The workspace account password is rotated

    Scenario: At API startup
      Given the Prose Pod has been initialized for test.org
       When the Prose Pod API starts
       Then <prose-workspace@test.org>'s password is changed
