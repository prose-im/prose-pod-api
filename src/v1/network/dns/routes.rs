// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::{get, serde::json::Json};
use serde::{Deserialize, Serialize};
use service::model::ServerConfig;

use crate::{error::Error, guards::LazyGuard};

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
pub async fn get_dns_records(
    server_config: LazyGuard<ServerConfig>,
) -> Result<Json<GetDnsRecordsResponse>, Error> {
    let server_config = server_config.inner?;

    let steps = vec![
        DnsSetupStep {
            purpose: "specify your server IP address".to_string(),
            records: vec![
                DnsRecord::A {
                    hostname: format!("xmpp.{}", server_config.domain),
                    ttl: 600,
                    value: "90.105.205.180".to_string(),
                },
                DnsRecord::AAAA {
                    hostname: format!("xmpp.{}", server_config.domain),
                    ttl: 600,
                    value: "2a01:cb05:899c:c200::1".to_string(),
                },
            ]
            .into_iter()
            .map(DnsRecordWithStringRepr::from)
            .collect(),
        },
        DnsSetupStep {
            purpose: "let clients connect to your server".to_string(),
            records: vec![DnsRecord::SRV {
                hostname: server_config.domain.to_string(),
                ttl: 3600,
                priority: 0,
                weight: 5,
                port: 5222,
                target: format!("xmpp.{}.", server_config.domain),
            }]
            .into_iter()
            .map(DnsRecordWithStringRepr::from)
            .collect(),
        },
        DnsSetupStep {
            purpose: "let servers connect to your server".to_string(),
            records: vec![DnsRecord::SRV {
                hostname: server_config.domain.to_string(),
                ttl: 3600,
                priority: 0,
                weight: 5,
                port: 5269,
                target: format!("xmpp.{}.", server_config.domain),
            }]
            .into_iter()
            .map(DnsRecordWithStringRepr::from)
            .collect(),
        },
    ];
    let res = GetDnsRecordsResponse { steps };
    Ok(res.into())
}

#[get("/v1/network/dns/checks", format = "json")]
pub async fn check_dns_records() -> Json<GetDnsRecordsResponse> {
    todo!()
}
