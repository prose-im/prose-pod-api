@workspace-vcard @workspace-details
Feature: Workspace vCard

  Background:
    Given the Prose Pod API has started

  Rule: The API should warn if the workspace has not been initialized when getting the workspace vCard

    Scenario: XMPP server and workspace not initialized
      Given the server config has not been initialized
       When an unauthenticated user gets the workspace vCard
       Then the user should receive 'Server config not initialized'

    Scenario: XMPP server initialized but not the workspace
      Given the server config has been initialized
        And the workspace has not been initialized
       When an unauthenticated user gets the workspace vCard
       Then the user should receive 'Workspace not initialized'

  """
  When logging into a workspace, we want to show some details of the workspace the person
  is logging into, which means an unauthenticated user must be able to query this information.
  It's a data leak but we'll find a workaround only if it becomes a real issue.
  """
  Rule: Anyone can request the workspace vCard

    Scenario: Someone tries to get the workspace vCard without authenticating
      Given the Prose Pod has been initialized
       When an unauthenticated user gets the workspace vCard
       Then the call should succeed
        And the response content type should be "text/vcard"

  Rule: Admins can change the workspace vCard

    Scenario: Valerian changes the workspace vCard
      Given the Prose Pod has been initialized
        And Valerian is an admin
        And the workspace is named "Prose"
       When Valerian sets the workspace vCard to "BEGIN:VCARD\nVERSION:4.0\nFN:Prose IM\nEND:VCARD"
       Then the call should succeed
        And the response content type should be "text/vcard"
        And the workspace should be named "Prose IM"

  Rule: Regular members can't change the workspace vCard

    Scenario: Rémi tries to change the workspace vCard
      Given the Prose Pod has been initialized
        And the workspace is named "Prose"
        And Rémi is a regular member
       When Rémi sets the workspace vCard to "BEGIN:VCARD\nVERSION:4.0\nFN:Prose IM\nEND:VCARD"
       Then the HTTP status code should be Forbidden
        And the workspace should be named "Prose"
