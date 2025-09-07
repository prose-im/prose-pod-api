// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Display;

use axum::response::sse::Event;
use serde_with::SerializeDisplay;
use serdev::Serialize;
use service::network_checks::*;

// ===== JSON RESPONSES =====

#[derive(Debug)]
#[derive(Serialize)]
pub struct NetworkCheckResult {
    id: String,
    event: NetworkCheckEvent,
    data: serde_json::Value,
}

impl NetworkCheckResult {
    pub fn new<'a, Id, Check, Status>(check: &'a Check, status: Status) -> Self
    where
        Id: From<&'a Check> + Display,
        Check: NetworkCheck,
        Status: Serialize,
        NetworkCheckEvent: From<&'a Check>,
    {
        let data = CheckResultData {
            description: check.description(),
            status,
        };
        Self {
            event: NetworkCheckEvent::from(&check),
            id: Id::from(check.to_owned()).to_string(),
            data: serde_json::to_value(&data).unwrap_or_default(),
        }
    }
}

#[macro_export]
macro_rules! impl_network_check_result_from {
    ($check:ty, $result:ty, $status:ty, $id:ty) => {
        impl From<($check, $result)> for crate::features::network_checks::NetworkCheckResult {
            fn from((check, result): ($check, $result)) -> Self {
                Self::new::<$id, $check, $status>(&check, <$status>::from(result))
            }
        }
    };
}

// ===== EVENTS =====

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(SerializeDisplay)]
#[derive(strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum NetworkCheckEvent {
    DnsRecordCheckResult,
    PortReachabilityCheckResult,
    IpConnectivityCheckResult,
}

#[macro_export]
macro_rules! impl_network_check_event_from {
    ($check:ty, $result:expr) => {
        impl From<&$check> for crate::features::network_checks::NetworkCheckEvent {
            fn from(_: &$check) -> Self {
                $result
            }
        }
    };
}

#[derive(Debug)]
#[derive(Serialize, serdev::Deserialize)]
pub struct CheckResultData<Status> {
    pub description: String,
    pub status: Status,
}

pub fn end_event() -> Event {
    Event::default()
        .event("end")
        .id("end")
        .comment("End of stream")
}
