@workspace-accent-color @workspace-details
Feature: Workspace accent color

  Background:
    Given the Prose Pod API has started

  Rule: The API should warn if the workspace has not been initialized when getting the workspace accent color

    Scenario: XMPP server and workspace not initialized
      Given the server config has not been initialized
       When an unauthenticated user gets the workspace accent color
       Then the user should receive 'Server config not initialized'

    Scenario: XMPP server initialized but not the workspace
      Given the server config has been initialized
        And the workspace has not been initialized
       When an unauthenticated user gets the workspace accent color
       Then the user should receive 'Workspace not initialized'

  """
  When logging into a workspace, we want to show the accent color of the workspace the person
  is logging into, which means an unauthenticated user must be able to query this information.
  It's a data leak but we'll find a workaround only if it becomes a real issue.
  """
  Rule: Anyone can request the workspace accent color

    Scenario: Get workspace accent color after initializing
      Given the Prose Pod has been initialized
        And the workspace accent color is "#2062da"
       When an unauthenticated user gets the workspace accent color
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace accent color should be "#2062da"

  Rule: Admins can change the workspace accent color

    Scenario: Valerian changes the workspace accent color
      Given the Prose Pod has been initialized
        And the workspace accent color is "#2062da"
        And Valerian is an admin
       When Valerian sets the workspace accent color to "#3458ad"
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace accent color should be "#3458ad"
        And the workspace accent color should be "#3458ad"

  Rule: Regular members can't change the workspace accent color

    Scenario: Rémi tries to change the workspace accent color
      Given the Prose Pod has been initialized
        And the workspace accent color is "#2062da"
        And Rémi is a regular member
       When Rémi sets the workspace accent color to "#3458ad"
       Then the HTTP status code should be Forbidden
        And the workspace accent color should be "#2062da"
