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
