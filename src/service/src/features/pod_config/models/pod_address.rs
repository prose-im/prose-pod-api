// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::rr::Name as DomainName;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone)]
#[derive(serdev::Serialize, serdev::Deserialize)]
#[serde(validate = "Self::validate")]
pub struct PodAddress {
    pub domain: Option<DomainName>,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
    /// NOTE: Here to prevent the creation of an invalid value.
    #[serde(skip)]
    _private: (),
}

impl PodAddress {
    fn validate(&self) -> Result<(), &'static str> {
        if (&self.domain, &self.ipv4, &self.ipv6) == (&None, &None, &None) {
            return Err("The Pod address must contain at least an IPv4, an IPv6 or a domain.");
        }

        Ok(())
    }
}
