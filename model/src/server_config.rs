// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

/// Values from <https://prosody.im/doc/modules/mod_limits>.
/// Probably abstract enough to be used in non-Prosody APIs.
///
/// See also <https://docs.ejabberd.im/admin/configuration/basic/#shapers> for ejabberd.
pub enum ConnectionType {
    /// "c2s"
    ClientToServer,
    /// "s2sin"
    ServerToServerInbounds,
    /// "s2sout"
    ServerToServerOutbounds,
}

/// Data-transfer rate (kB/s, MB/s…).
/// See <https://en.wikipedia.org/wiki/Data-rate_units> for Prosody
/// and <https://docs.ejabberd.im/admin/configuration/basic/#shapers> for ejabberd.
#[derive(Debug)]
pub enum DataRate {
    BytesPerSec(u32),
    KiloBytesPerSec(u32),
    MegaBytesPerSec(u32),
}

pub trait Duration {}

#[derive(Debug)]
pub enum DurationTime {
    Seconds(u32),
    Minutes(u32),
    Hours(u32),
}

impl DurationTime {
    pub fn seconds(&self) -> u32 {
        match self {
            Self::Seconds(n) => n.clone(),
            Self::Minutes(n) => n * Self::Seconds(60).seconds(),
            Self::Hours(n) => n * Self::Minutes(60).seconds(),
        }
    }
}

impl Duration for DurationTime {}

#[derive(Debug)]
pub enum DurationDate {
    Days(u32),
    Weeks(u32),
    Months(u32),
    Years(u32),
}

impl Duration for DurationDate {}

#[derive(Debug)]
pub enum PossiblyInfinite<D: Duration> {
    Infinite,
    Finite(D),
}
