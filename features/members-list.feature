Feature: Members list

  Background:
    Given the Prose Pod has been initialized
      And the XMPP server domain is prose.org
      And Valerian is an admin
      And the Prose Pod API has started

  """
  In the Prose Pod Dashboard, admins should be able to see members.
  """
  Rule: Admins can list members

    Scenario: Small number of members
      Given the workspace has 2 members
       When Valerian lists members
       Then the HTTP status code should be OK
        And the response content type should be JSON
        And they should see 2 members

    Scenario: Large number of members, first page
      Given the workspace has 42 members
       When Valerian lists members by pages of 20
       Then the HTTP status code should be Partial Content
        And they should see 20 members
        And the "Pagination-Current-Page" header should contain "1"
        And the "Pagination-Page-Size" header should contain "20"
        And the "Pagination-Page-Count" header should contain "3"

    Scenario: Large number of members, last page
      Given the workspace has 42 members
       When Valerian gets page 3 of members by pages of 20
       Then the HTTP status code should be OK
        And they should see 2 members
        And the "Pagination-Current-Page" header should contain "3"
        And the "Pagination-Page-Size" header should contain "20"
        And the "Pagination-Page-Count" header should contain "3"

  Rule: One needs to be authenticated in order to list members

    Scenario: Not authenticated
      When someone lists members without authenticating
      Then the HTTP status code should be Unauthorized

    Scenario Outline: Bad token
      When someone lists members using <token> as Bearer token
      Then the HTTP status code should be Forbidden

    Examples:
      | token |
      | "undefined" |
      | "" |
      | "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJqaWQiOiJqb2huLmRvZUBleGFtcGxlLm9yZyJ9.C5mKZHGQhjTv8Td43yLcZ9S3-MQlDHexhaxlLWr6ixg" |

  Rule: Listing members should not interact with the XMPP server

    Scenario: XMPP server offline
      Given the workspace has 2 members
        And the XMPP server is offline
       When Valerian lists members
       Then the call should succeed

  Rule: Admins can lazily load more information about users

    Scenario: Heterogeneous information
      Given Rémi is a regular member
        And Rémi is online
        And Rémi's avatar is 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
        And John is a regular member
        And John is offline
        And John has no avatar
       When Valerian gets detailed information about Rémi and John
       Then the response is a SSE stream
        And one SSE event is "id:rémi@prose.org\nevent:enriched-member\ndata:{\"jid\":\"rémi@prose.org\",\"role\":\"MEMBER\",\"online\":true,\"nickname\":\"Rémi\",\"avatar\":\"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\"}"
        And one SSE event is "id:john@prose.org\nevent:enriched-member\ndata:{\"jid\":\"john@prose.org\",\"role\":\"MEMBER\",\"online\":false,\"nickname\":\"John\",\"avatar\":null}"
