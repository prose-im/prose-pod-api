// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::model::JID;
use crate::prosody_config::ProsodyConfig;

/// Abstraction over ProsodyCtl in case we want to switch to another server.
/// Also facilitates testing.
trait ServerCtl {
    type Config;

    fn config(&self) -> &Self::Config;

    fn add_admin(&mut self, jid: JID);
    fn remove_admin(&mut self, jid: &JID);

    fn add_enabled_module(&mut self, module_name: String);
    fn remove_enabled_module(&mut self, module_name: &String);

    fn add_disabled_module(&mut self, module_name: String);
    fn remove_disabled_module(&mut self, module_name: &String);

    fn set_rate_limit(&mut self, conn_type: ConnectionType, value: DataRate);
    fn set_burst_limit(&mut self, conn_type: ConnectionType, value: Duration);
    /// Sets the time that an over-limit session is suspended for
    /// (`limits_resolution` in Prosody).
    ///
    /// See <https://prosody.im/doc/modules/mod_limits> for Prosody
    /// and <https://docs.ejabberd.im/admin/configuration/basic/#shapers> for ejabberd.
    fn set_timeout(&mut self, value: Duration);
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

#[derive(Debug)]
pub enum Duration {
    Seconds(u32),
}

impl Duration {
    pub fn seconds(&self) -> &u32 {
        match self {
            Duration::Seconds(n) => n,
        }
    }
}

struct ProsodyCtl {
    config: ProsodyConfig,
}

impl ServerCtl for ProsodyCtl {
    type Config = ProsodyConfig;

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn add_admin(&mut self, jid: JID) {
        self.config.admins.insert(jid);
    }
    fn remove_admin(&mut self, jid: &JID) {
        self.config.admins.remove(jid);
    }

    fn add_enabled_module(&mut self, module_name: String) {
        self.config.enabled_modules.insert(module_name);
    }
    fn remove_enabled_module(&mut self, module_name: &String) {
        self.config.enabled_modules.remove(module_name);
    }

    fn add_disabled_module(&mut self, module_name: String) {
        self.config.disabled_modules.insert(module_name);
    }
    fn remove_disabled_module(&mut self, module_name: &String) {
        self.config.disabled_modules.remove(module_name);
    }

    fn set_rate_limit(&mut self, conn_type: ConnectionType, value: DataRate) {
        self.config
            .limits
            .entry(conn_type.into())
            .or_insert_with(Default::default)
            .rate = Some(value)
    }
    fn set_burst_limit(&mut self, conn_type: ConnectionType, value: Duration) {
        self.config
            .limits
            .entry(conn_type.into())
            .or_insert_with(Default::default)
            .burst = Some(value)
    }
    /// Sets the time that an over-limit session is suspended for
    /// (`limits_resolution` in Prosody).
    ///
    /// See <https://prosody.im/doc/modules/mod_limits> for Prosody
    /// and <https://docs.ejabberd.im/admin/configuration/basic/#shapers> for ejabberd.
    fn set_timeout(&mut self, value: Duration) {
        self.config.limits_resolution = Some(value.seconds().clone());
    }
}
