@test-connections @preflight-checks
Feature: Test required services are reachable

  Rule: The API doesn't start if the SMTP server isn't reachable

    Scenario: SMTP server not reachable
      Given the SMTP server isn't reachable
       When the Prose Pod API starts
       Then the Prose Pod API isn't running

    Scenario: SMTP server reachable
      Given the SMTP server is reachable
       When the Prose Pod API starts
       Then the Prose Pod API is running
