// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, Mutex};
use ::model::JID;

pub struct ServerCtl {
    pub implem: Arc<Mutex<dyn ServerCtlImpl>>,
}

impl ServerCtl {
    pub fn new(implem: Arc<Mutex<dyn ServerCtlImpl>>) -> Self {
        Self { implem }
    }
}

/// Abstraction over ProsodyCtl in case we want to switch to another server.
/// Also facilitates testing.
pub trait ServerCtlImpl: Sync + Send {
    fn save_config(&self);

    fn add_admin(&mut self, jid: JID);
    fn remove_admin(&mut self, jid: &JID);

    /// Returns whether or not the value was changed.
    fn add_enabled_module(&mut self, module_name: String) -> bool;
    /// Returns whether or not the value was changed.
    fn remove_enabled_module(&mut self, module_name: &String) -> bool;

    /// Returns whether or not the value was changed.
    fn add_disabled_module(&mut self, module_name: String) -> bool;
    /// Returns whether or not the value was changed.
    fn remove_disabled_module(&mut self, module_name: &String) -> bool;

    fn set_rate_limit(&mut self, conn_type: ConnectionType, value: DataRate);
    fn set_burst_limit(&mut self, conn_type: ConnectionType, value: DurationTime);
    /// Sets the time that an over-limit session is suspended for
    /// (`limits_resolution` in Prosody).
    ///
    /// See <https://prosody.im/doc/modules/mod_limits> for Prosody
    /// and <https://docs.ejabberd.im/admin/configuration/basic/#shapers> for ejabberd.
    fn set_timeout(&mut self, value: DurationTime);

    /// Returns whether or not the value was changed.
    fn enable_message_archiving(&mut self) -> bool;
    /// Returns whether or not the value was changed.
    fn disable_message_archiving(&mut self) -> bool;
}

impl dyn ServerCtlImpl {
    /// Returns whether or not the value was changed.
    pub fn set_message_archiving(&mut self, new_state: bool) -> bool {
        if new_state {
            self.enable_message_archiving()
        } else {
            self.disable_message_archiving()
        } 
    }
}

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
            Self::Hours(n)   => n * Self::Minutes(60).seconds(),
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
