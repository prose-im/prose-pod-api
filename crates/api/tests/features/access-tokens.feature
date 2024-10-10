@access-tokens
@authentication
Feature: Prose Pod API access tokens

  Background:
    Given the Prose Pod has been initialized
      And the Prose Pod API has started

  Rule: Access tokens expire after 3 hours

    Scenario: User logs in
      Given Alice is a member
       When Alice logs into the Prose Pod API
       Then their access token should expire after 3 hours

  """
  In order for the Prose Pod API to send stanzas to Prosody, it needs a Prosody access token.
  This token is generated when a user logs in and is saved into the returned Prose Pod API access token.
  Although this token is encrypted and only the Prose Pod API can read its contents, there is no need
  for the Prosody access token to be valid longer than the Prose Pod API access token.
  """
  @prosody
  Rule: Prosody access tokens expire after 3 hours

    Scenario: User logs in
      Given Alice is a member
       When Alice logs into the Prose Pod API
       Then their Prosody access token should expire after 3 hours
