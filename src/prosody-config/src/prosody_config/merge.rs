// prosody-config
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::ProsodySettings;

#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    KeepSelf,
    KeepOther,
}

impl ProsodySettings {
    /// Merge two [`ProsodySettings`], where values in `self` take precedence
    /// over values in `other`.
    pub fn shallow_merged_with(&self, other: Self, strategy: MergeStrategy) -> Self {
        let mut res = self.clone();
        res.shallow_merge(other, strategy);
        res
    }

    // TODO: Find a way to use a macro instead…
    /// Merge two [`ProsodySettings`], where values in `self` take precedence
    /// over values in `other`.
    pub fn shallow_merge(&mut self, other: Self, strategy: MergeStrategy) {
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
            http_file_share_secret,
            http_file_share_base_url,
            http_file_share_size_limit,
            http_file_share_allowed_file_types,
            http_file_share_safe_file_types,
            http_file_share_expires_after,
            http_file_share_daily_quota,
            http_file_share_global_quota,
            http_file_share_access,
        } = other;

        macro_rules! merge {
            ($key:ident) => {
                match (strategy, &self.$key, $key) {
                    (MergeStrategy::KeepSelf, None, b @ Some(_)) => self.$key = b,
                    (MergeStrategy::KeepOther, Some(_), b @ Some(_)) => self.$key = b,
                    _ => {}
                }
            };
        }

        merge!(pidfile);
        merge!(admins);
        merge!(authentication);
        merge!(default_storage);
        merge!(storage);
        merge!(storage_archive_item_limit);
        merge!(storage_archive_item_limit_cache_size);
        merge!(sql);
        merge!(sql_manage_tables);
        merge!(sqlite_tune);
        merge!(log);
        merge!(interfaces);
        merge!(c2s_ports);
        merge!(s2s_ports);
        merge!(http_ports);
        merge!(http_interfaces);
        merge!(https_ports);
        merge!(https_interfaces);
        merge!(plugin_paths);
        merge!(modules_enabled);
        merge!(modules_disabled);
        merge!(ssl);
        merge!(allow_registration);
        merge!(c2s_require_encryption);
        merge!(s2s_require_encryption);
        merge!(s2s_secure_auth);
        merge!(c2s_stanza_size_limit);
        merge!(s2s_stanza_size_limit);
        merge!(s2s_whitelist);
        merge!(limits);
        merge!(consider_websocket_secure);
        merge!(cross_domain_websocket);
        merge!(contact_info);
        merge!(archive_expires_after);
        merge!(default_archive_policy);
        merge!(max_archive_query_results);
        merge!(upgrade_legacy_vcards);
        merge!(groups_file);
        merge!(http_host);
        merge!(http_external_url);
        merge!(restrict_room_creation);
        merge!(muc_log_all_rooms);
        merge!(muc_log_by_default);
        merge!(muc_log_expires_after);
        merge!(reload_modules);
        merge!(reload_global_modules);
        merge!(tls_profile);
        merge!(http_file_share_secret);
        merge!(http_file_share_base_url);
        merge!(http_file_share_size_limit);
        merge!(http_file_share_allowed_file_types);
        merge!(http_file_share_safe_file_types);
        merge!(http_file_share_expires_after);
        merge!(http_file_share_daily_quota);
        merge!(http_file_share_global_quota);
        merge!(http_file_share_access);

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
