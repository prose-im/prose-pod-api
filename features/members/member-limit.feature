# As [Prose’s pricing page](https://prose.org/pricing) states,
# the Community version of Prose is limited to 100 users.
# The API therefore needs to have a way to limit account creation.
#
# NOTE: We need `@serial` for this feature since it heavily depends on a static
#   value (not an instance-specific configuration key).
@member-limit @members
@serial
Feature: Member limit

  Background:
    Given the Prose Pod has been initialized
      And the Prose Pod API has started

  Rule: New members cannot join the Workspace if the member limit is reached

    Example: Adding a member while under the limit
      Given the member limit is 5
        And the Workspace has 4 members
       When a new member joins the Workspace
       Then the call should succeed

    Example: Adding a member while at the limit
      Given the member limit is 3
        And the Workspace has 3 members
       When a new member joins the Workspace
       Then the call should fail
        And the error code should be "member_limit_reached"
        And there should be 3 members in the database

    # This shouldn’t happen, but just in case we lower the limit at some point,
    # we have to make sure the API doesn’t use `==` (who’d do that anyway?).
    Example: Adding a member while above the limit
      Given the member limit is 3
        And the Workspace has 5 members
       When a new member joins the Workspace
       Then the call should fail
        And the error code should be "member_limit_reached"
        And there should be 5 members in the database

  Rule: The member limit counts how many members are currently in the Workspace, not how many have been created over the lifetime of the Prose Pod

    # This scenario ensures the API doesn’t use an auto-incrementing counter
    # (which would be equal to the number of members ever created).
    # See [SQLite Autoincrement](https://sqlite.org/autoinc.html).
    Example: The API uses an auto-incrementing counter
      Given the member limit is 3
        And Valerian is an admin
        And the Workspace has 3 members
       When Valerian deletes a member
        And a new member joins the Workspace
       Then the call should succeed

    # This scenario ensures the API doesn’t use `ROWID` (which would
    # be equal to the largest `ROWID` currently in use plus one).
    # See [SQLite Autoincrement](https://sqlite.org/autoinc.html).
    Example: The API uses SQLite’s `ROWID`
      Given the member limit is 4
        And Valerian is an admin
        And the Workspace has 4 members
       When Valerian deletes member 2
        And a new member joins the Workspace
       Then the call should succeed
