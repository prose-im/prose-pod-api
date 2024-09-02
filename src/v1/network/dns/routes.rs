// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::{get, serde::json::Json};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::{
    model::{PodAddress, ServerConfig},
    prose_xmpp::BareJid,
    repositories::{MemberRepository, PodConfigRepository},
};

use crate::{
    error::{self, Error},
    guards::{Db, LazyGuard},
};

#[derive(Debug, Serialize, Deserialize, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::EnumString))]
#[serde(tag = "type")]
pub enum DnsRecord {
    A {
        hostname: String,
        ttl: u32,
        value: String,
    },
    AAAA {
        hostname: String,
        ttl: u32,
        value: String,
    },
    SRV {
        hostname: String,
        ttl: u32,
        priority: u32,
        weight: u32,
        port: u32,
        target: String,
    },
}

/// NOTE: Only used in tests.
#[cfg(debug_assertions)]
impl DnsRecord {
    pub fn record_type(&self) -> DnsRecordDiscriminants {
        DnsRecordDiscriminants::from(self)
    }
}

impl ToString for DnsRecord {
    fn to_string(&self) -> String {
        match self {
            Self::A {
                hostname,
                ttl,
                value,
            } => format!("{hostname} {ttl} IN A {value}"),
            Self::AAAA {
                hostname,
                ttl,
                value,
            } => format!("{hostname} {ttl} IN AAAA {value}"),
            Self::SRV {
                hostname,
                ttl,
                priority,
                weight,
                port,
                target,
            } => format!("{hostname} {ttl} IN SRV {priority} {weight} {port} {target}"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DnsRecordWithStringRepr {
    #[serde(flatten)]
    pub inner: DnsRecord,
    pub string_repr: String,
}

impl Deref for DnsRecordWithStringRepr {
    type Target = DnsRecord;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<DnsRecord> for DnsRecordWithStringRepr {
    fn from(dns_record: DnsRecord) -> Self {
        Self {
            string_repr: dns_record.to_string(),
            inner: dns_record,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DnsSetupStep {
    /// The purpose of this step.
    ///
    /// Example: "specify your server IP address".
    ///
    /// Note that it always starts with a lowercase letter.
    pub purpose: String,
    pub records: Vec<DnsRecordWithStringRepr>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDnsRecordsResponse {
    pub steps: Vec<DnsSetupStep>,
}

#[get("/v1/network/dns/records", format = "json")]
pub async fn get_dns_records<'r>(
    conn: Connection<'r, Db>,
    jid: LazyGuard<BareJid>,
    server_config: LazyGuard<ServerConfig>,
) -> Result<Json<GetDnsRecordsResponse>, Error> {
    // === Boilerplate 1 ===

    let db = conn.into_inner();
    let server_domain = server_config.inner?.domain;

    // === Access control ===

    let jid = jid.inner?;
    // TODO: Use a request guard instead of checking in the route body if the user can invite members.
    if !MemberRepository::is_admin(db, &jid).await? {
        return Err(error::Forbidden(format!("<{jid}> is not an admin")).into());
    }

    // === Boilerplate 2 ===

    let Some(pod_config) = PodConfigRepository::get(db).await? else {
        return Err(error::PodAddressNotInitialized.into());
    };
    let pod_address = PodAddress::try_from(pod_config)?;

    // === Helpers to create DNS records ===

    let a = |ipv4: String| {
        DnsRecordWithStringRepr::from(DnsRecord::A {
            hostname: format!("xmpp.{server_domain}"),
            ttl: 600,
            value: ipv4,
        })
    };
    let aaaa = |ipv6: String| {
        DnsRecordWithStringRepr::from(DnsRecord::AAAA {
            hostname: format!("xmpp.{server_domain}"),
            ttl: 600,
            value: ipv6,
        })
    };
    let srv = |port: u32, target: String| {
        DnsRecordWithStringRepr::from(DnsRecord::SRV {
            hostname: server_domain.to_string(),
            ttl: 3600,
            priority: 0,
            weight: 5,
            port,
            target,
        })
    };
    let srv_c2s = |target: String| srv(5222, target);
    let srv_s2s = |target: String| srv(5269, target);

    // === Helpers to create DNS setup steps ===

    let step_static_ip = |ipv4: String, ipv6: Option<String>| DnsSetupStep {
        purpose: "specify your server IP address".to_string(),
        records: vec![
            Some(a(ipv4)),
            ipv6.map(aaaa),
        ]
        .into_iter()
        .flatten()
        .collect(),
    };
    let step_c2s = |target: String| DnsSetupStep {
        purpose: "let clients connect to your server".to_string(),
        records: vec![srv_c2s(target)],
    };
    let step_s2s = |target: String| DnsSetupStep {
        purpose: "let servers connect to your server".to_string(),
        records: vec![srv_s2s(target)],
    };

    // === Main logic ===

    let steps = match pod_address {
        PodAddress::Static { ipv4, ipv6 } => vec![
            step_static_ip(ipv4, ipv6),
            step_c2s(format!("xmpp.{server_domain}.")),
            step_s2s(format!("xmpp.{server_domain}.")),
        ],
        PodAddress::Dynamic { hostname } => vec![
            step_c2s(format!("{hostname}.")),
            step_s2s(format!("{hostname}.")),
        ],
    };

    let res = GetDnsRecordsResponse { steps };
    Ok(res.into())
}

#[get("/v1/network/checks/dns", format = "json")]
pub async fn check_dns_records() -> Json<GetDnsRecordsResponse> {
    todo!()
}
