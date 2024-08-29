Feature: Updating the Prose Pod address

  Rule: One can change from IP addresses to hostname and vice versa

    Scenario: User had given IP addresses, but wants to switch to a hostname
      Given the Prose Pod is publicly accessible via an IPv4
        And the Prose Pod is publicly accessible via an IPv6
       When Valerian

    #Scenario: User had given a hostname, but wants to switch to IP addresses
