@workspace-name @workspace-details
Feature: Workspace name

  Background:
    Given the Prose Pod API has started

  Rule: The API should warn if the workspace has not been initialized when getting the workspace name

    Scenario: XMPP server initialized but not the workspace
      Given the XMPP server has been initialized
        And the workspace has not been initialized
       When an unauthenticated user gets the workspace name
       Then the user should receive 'Workspace not initialized: No vCard'

  """
  When logging into a workspace, we want to show the name of the workspace the person
  is logging into, which means an unauthenticated user must be able to query this information.
  It's a data leak but we'll find a workaround only if it becomes a real issue.
  """
  Rule: Anyone can request the workspace name

    Scenario: Get workspace name after initializing
      Given the Prose Pod has been initialized
        And the workspace is named "Prose"
       When an unauthenticated user gets the workspace name
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace name should be "Prose"

  Rule: Admins can change the workspace name

    Scenario: Valerian changes the workspace name
      Given the Prose Pod has been initialized
        And the workspace is named "Prose"
        And Valerian is an admin
       When Valerian sets the workspace name to "Prose IM"
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace name should be "Prose IM"
        And the workspace should be named "Prose IM"

  Rule: Regular members can't change the workspace name

    Scenario: Rémi tries to change the workspace name
      Given the Prose Pod has been initialized
        And the workspace is named "Prose"
        And Rémi is a regular member
       When Rémi sets the workspace name to "Prose IM"
       Then the HTTP status code should be Forbidden
        And the workspace should be named "Prose"
