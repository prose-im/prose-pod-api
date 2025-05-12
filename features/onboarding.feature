@onboarding
Feature: Onboarding steps

  Background:
    Given the Prose Pod has been initialized for prose.org
      And the Prose Pod API has started
      And Valerian is an admin

  Rule: One can know if DNS checks all passed once

    Scenario: All checks pass
      Given onboarding step "all_dns_checks_passed_once" is false
        And the Prose Pod is publicly accessible via a hostname
        And the XMPP server domain is test.prose.org
        And prose.org’s DNS zone has a SRV record for test.prose.org redirecting port 5222 to cloud-provider.com.
        And prose.org’s DNS zone has a SRV record for test.prose.org redirecting port 5269 to cloud-provider.com.
       When Valerian checks the DNS records configuration
        And Valerian queries onboarding steps statuses
       Then onboarding step "all_dns_checks_passed_once" should be true

    Scenario: All checks pass but one
      Given onboarding step "all_dns_checks_passed_once" is false
        And the Prose Pod is publicly accessible via a hostname
        And the XMPP server domain is test.prose.org
        And prose.org’s DNS zone has a SRV record for test.prose.org redirecting port 5222 to cloud-provider.com.
       When Valerian checks the DNS records configuration
        And Valerian queries onboarding steps statuses
       Then onboarding step "all_dns_checks_passed_once" should be false

  Rule: One can know if one invitation has already been sent

    Scenario: No invitation sent yet
      Given onboarding step "at_least_one_invitation_sent" is false
       When Valerian queries onboarding steps statuses
       Then onboarding step "at_least_one_invitation_sent" should be false

    Scenario: After sending an invitation
      Given onboarding step "at_least_one_invitation_sent" is false
       When Valerian invites <remi@prose.org> as a MEMBER
        And Valerian queries onboarding steps statuses
       Then onboarding step "at_least_one_invitation_sent" should be true
