@onboarding
Feature: Onboarding steps

  Background:
    Given the Prose Pod has been initialized
      And the Prose Pod API has started
      And Valerian is an admin

  Rule: One can know if DNS checks all passed once

    Scenario: All checks pass
      Given config "server.domain" is set to "test.org"
        And config "pod.address.domain" is set to "prose.test.org"
        And the Prose Pod API has restarted
        And onboarding step "all_dns_checks_passed_once" is false
        And prose.org’s DNS zone has a CNAME record redirecting admin.prose.test.org. to prose.test.org.
        And prose.org’s DNS zone has a SRV record for _xmpp-client._tcp.test.org. redirecting port 5222 to cloud-provider.com.
       When Valerian checks the DNS records configuration
        And Valerian queries onboarding steps statuses
       Then onboarding step "all_dns_checks_passed_once" should be true

    Scenario: All checks pass but one
      Given config "server.domain" is set to "test.org"
        And config "pod.address.domain" is set to "prose.test.org"
        And the Prose Pod API has restarted
        And onboarding step "all_dns_checks_passed_once" is false
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
