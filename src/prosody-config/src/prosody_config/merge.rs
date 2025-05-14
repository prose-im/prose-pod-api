// prosody-config
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::ProsodySettings;

impl ProsodySettings {
    /// Merge two [`ProsodySettings`], where values in `self` take precedence
    /// over values in `other`.
    pub fn shallow_merged_with(&self, other: Self) -> Self {
        let mut res = self.clone();
        res.shallow_merge(other);
        res
    }

    // TODO: Find a way to use a macro instead…
    /// Merge two [`ProsodySettings`], where values in `self` take precedence
    /// over values in `other`.
    pub fn shallow_merge(&mut self, other: Self) {
        // NOTE: Destructuring `other` ensures we don’t forget to merge one key.
        let Self {
            pidfile,
            admins,
            authentication,
            default_storage,
            storage,
            storage_archive_item_limit,
            storage_archive_item_limit_cache_size,
            sql,
            sql_manage_tables,
            sqlite_tune,
            log,
            interfaces,
            c2s_ports,
            s2s_ports,
            http_ports,
            http_interfaces,
            https_ports,
            https_interfaces,
            plugin_paths,
            modules_enabled,
            modules_disabled,
            ssl,
            allow_registration,
            c2s_require_encryption,
            s2s_require_encryption,
            s2s_secure_auth,
            c2s_stanza_size_limit,
            s2s_stanza_size_limit,
            s2s_whitelist,
            limits,
            consider_websocket_secure,
            cross_domain_websocket,
            contact_info,
            archive_expires_after,
            default_archive_policy,
            max_archive_query_results,
            upgrade_legacy_vcards,
            groups_file,
            http_file_share_size_limit,
            http_file_share_daily_quota,
            http_file_share_expires_after,
            http_host,
            http_external_url,
            restrict_room_creation,
            muc_log_all_rooms,
            muc_log_by_default,
            muc_log_expires_after,
            tls_profile,
            reload_modules,
            reload_global_modules,
            custom_settings,
        } = other;

        macro_rules! replace_if_none {
            ($key:ident) => {
                if self.$key == None {
                    self.$key = $key;
                }
            };
        }

        replace_if_none!(pidfile);
        replace_if_none!(admins);
        replace_if_none!(authentication);
        replace_if_none!(default_storage);
        replace_if_none!(storage);
        replace_if_none!(storage_archive_item_limit);
        replace_if_none!(storage_archive_item_limit_cache_size);
        replace_if_none!(sql);
        replace_if_none!(sql_manage_tables);
        replace_if_none!(sqlite_tune);
        replace_if_none!(log);
        replace_if_none!(interfaces);
        replace_if_none!(c2s_ports);
        replace_if_none!(s2s_ports);
        replace_if_none!(http_ports);
        replace_if_none!(http_interfaces);
        replace_if_none!(https_ports);
        replace_if_none!(https_interfaces);
        replace_if_none!(plugin_paths);
        replace_if_none!(modules_enabled);
        replace_if_none!(modules_disabled);
        replace_if_none!(ssl);
        replace_if_none!(allow_registration);
        replace_if_none!(c2s_require_encryption);
        replace_if_none!(s2s_require_encryption);
        replace_if_none!(s2s_secure_auth);
        replace_if_none!(c2s_stanza_size_limit);
        replace_if_none!(s2s_stanza_size_limit);
        replace_if_none!(s2s_whitelist);
        replace_if_none!(limits);
        replace_if_none!(consider_websocket_secure);
        replace_if_none!(cross_domain_websocket);
        replace_if_none!(contact_info);
        replace_if_none!(archive_expires_after);
        replace_if_none!(default_archive_policy);
        replace_if_none!(max_archive_query_results);
        replace_if_none!(upgrade_legacy_vcards);
        replace_if_none!(groups_file);
        replace_if_none!(http_file_share_size_limit);
        replace_if_none!(http_file_share_daily_quota);
        replace_if_none!(http_file_share_expires_after);
        replace_if_none!(http_host);
        replace_if_none!(http_external_url);
        replace_if_none!(restrict_room_creation);
        replace_if_none!(muc_log_all_rooms);
        replace_if_none!(muc_log_by_default);
        replace_if_none!(muc_log_expires_after);
        replace_if_none!(reload_modules);
        replace_if_none!(reload_global_modules);
        replace_if_none!(tls_profile);

        // NOTE: We cannot simply use `Vec::extend`, as it would make values
        //   in `other` take precedence over values in `self` (which is the
        //   opposite of what happens for other keys). For consistency, we have
        //   to prepend values of `other` (Prosody settings appearing last
        //   taking precedence over previous ones).
        // NOTE: This implementation isn’t the most efficient, but who cares?
        self.custom_settings = [
            custom_settings,
            self.custom_settings.clone(),
        ]
        .concat();
    }
}
