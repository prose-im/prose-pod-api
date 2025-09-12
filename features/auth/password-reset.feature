@password-reset @auth
Feature: Password reset

  Background:
    Given the Prose Pod has been initialized
      And the Prose Pod API has started

  Rule: Admins can send a password reset email

    Example: Valerian (admin) requests a password reset for Rémi
      Given Valerian is an admin
        And Rémi is a member
       When Valerian requests a password reset for Rémi
       Then the HTTP status code should be Accepted
        And the response body should be empty

  Rule: Only admins can send a password reset emails

    Example: Rémi (not admin) tries to request a password reset for Marc
      Given Rémi is not an admin
        And Marc is not an admin
       When Rémi requests a password reset for Marc
       Then the call should fail
        And the HTTP status code should be Forbidden

  Rule: Password reset tokens can be used to change someone’s password

    Example: Rémi resets his password
      Given Valerian is an admin
        And Rémi is a member
        And Rémi’s password is "12345678"
        And Valerian requested a password reset for Rémi
       When an unauthenticated user uses Rémi’s password reset token with password "new-password"
       Then the call should succeed
        And Rémi’s password should be "new-password"

  Rule: Password reset tokens can only be used once

    Example: Valerian (admin) requests a password reset for Rémi
      Given Valerian is an admin
        And Rémi is not an admin
        And Valerian requested a password reset for Rémi
       When an unauthenticated user uses Rémi’s password reset token with password "new-password"
        And an unauthenticated user uses Rémi’s password reset token with password "new-password-2"
       Then the HTTP status code should be Not Found
        And the error code should be "password_reset_token_expired"

  Rule: Password reset tokens expire

    Example: Rémi tries to reset his password after the token has expired
      Given config "auth.password_reset_token_ttl" is set to "PT0S"
        And the Prose Pod API has restarted
        And Valerian is an admin
        And Rémi is not an admin
        And Valerian requested a password reset for Rémi
       When an unauthenticated user uses Rémi’s password reset token with password "new-password"
       Then the HTTP status code should be Gone
        And the error code should be "password_reset_token_expired"

  """
  If a member has a slow email server, they might say “I haven’t received” the
  email to an admin who’d send a new password reset request. If we canceled
  existing tokens, the member would receive the first email, click on the link
  but it would be invalid.

  Tokens expire quickly so adding logic for this would just be a footgun (no
  real security benefit).
  """
  Rule: A member can request multiple password resets at the same time

    Example: Rémi requests a password reset twice
      Given Valerian is an admin
        And Rémi is not an admin
        And Valerian requested a password reset for Rémi
       When Valerian requests a password reset for Rémi
       Then the call should succeed
        And there should be 2 valid password reset tokens for Rémi

  """
  Password reset tokens expire quickly so there is no real reason to cancel
  existing ones when a member uses a token and more than one exist for them.
  This situation is already very rare and we might just footgun ourselves trying
  to implement this. Let’s just let unused tokens expire.
  """
  Rule: Using a password reset token doesn’t cancel existing ones for the same user

    Example: Rémi resets his password twice at the same time
      Given Valerian is an admin
        And Rémi is not an admin
        And Valerian requested a password reset for Rémi
        And Valerian requested a password reset for Rémi
       When an unauthenticated user uses Rémi’s first password reset token with password "new-password"
        And an unauthenticated user uses Rémi’s second password reset token with password "new-password-2"
       Then the call should succeed
        And Rémi’s password should be "new-password-2"
