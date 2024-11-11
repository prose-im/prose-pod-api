// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

/// Bytes.
///
/// See <https://en.wikipedia.org/wiki/Byte#Multiple-byte_units>.
#[derive(Debug, Eq, PartialEq)]
pub enum Bytes {
    Bytes(u32),
    KiloBytes(u32),
    KibiBytes(u32),
    MegaBytes(u32),
    MebiBytes(u32),
}

/// Data-transfer rate (kB/s, MB/s…).
///
/// See <https://en.wikipedia.org/wiki/Data-rate_units>
/// and <https://docs.ejabberd.im/admin/configuration/basic/#shapers> for ejabberd.
#[derive(Debug, Eq, PartialEq)]
pub enum DataRate {
    BytesPerSec(u32),
    KiloBytesPerSec(u32),
    MegaBytesPerSec(u32),
}
