Feature: Members list

  Background:
    Given the Prose Pod has been initialized

  """
  In the Prose Pod Dashboard, admins should be able to see members.
  """
  Rule: Admins can list members

    Scenario: Small number of members
      Given Valerian is an admin
        And the workspace has 2 members
       When Valerian lists members
       Then the HTTP status code should be OK
        And the response content type should be JSON
        And they should see 2 members

    Scenario: Large number of members, first page
      Given Valerian is an admin
        And the workspace has 42 members
       When Valerian lists members by pages of 20
       Then the HTTP status code should be Partial Content
        And they should see 20 members
        And the "Pagination-Current-Page" header should contain "1"
        And the "Pagination-Page-Size" header should contain "20"
        And the "Pagination-Page-Count" header should contain "3"

    Scenario: Large number of members, last page
      Given Valerian is an admin
        And the workspace has 42 members
       When Valerian gets page 3 of members by pages of 20
       Then the HTTP status code should be OK
        And they should see 2 members
        And the "Pagination-Current-Page" header should contain "3"
        And the "Pagination-Page-Size" header should contain "20"
        And the "Pagination-Page-Count" header should contain "3"

  Rule: Listing members should not interact with the XMPP server

    Scenario: XMPP server offline
      Given Valerian is an admin
        And the workspace has 2 members
        And the XMPP server is offline
       When Valerian lists members
       Then the call should succeed

  Rule: Admins can lazily load more information about users
