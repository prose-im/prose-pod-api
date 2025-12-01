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
            access_control_allow_credentials,
            access_control_allow_headers,
            access_control_allow_methods,
            access_control_allow_origins,
            access_control_max_age,
            admin_socket,
            admins,
            allow_registration,
            archive_cleanup_date_cache_size,
            archive_expires_after,
            archive_store,
            authentication,
            c2s_close_timeout,
            c2s_direct_tls_ports,
            c2s_ports,
            c2s_require_encryption,
            c2s_stanza_size_limit,
            c2s_tcp_keepalives,
            c2s_timeout,
            consider_websocket_secure,
            contact_info,
            cross_domain_websocket,
            mut custom_settings,
            default_archive_policy,
            default_storage,
            disco_expose_admins,
            disco_hidden,
            disco_items,
            dont_archive_namespaces,
            groups_file,
            http_default_cors_enabled,
            http_default_host,
            http_external_url,
            http_file_share_access,
            http_file_share_allowed_file_types,
            http_file_share_base_url,
            http_file_share_daily_quota,
            http_file_share_expires_after,
            http_file_share_global_quota,
            http_file_share_safe_file_types,
            http_file_share_secret,
            http_file_share_size_limit,
            http_host,
            http_interfaces,
            http_legacy_x_forwarded,
            http_max_buffer_size,
            http_max_content_size,
            http_paths,
            http_ports,
            https_interfaces,
            https_ports,
            interfaces,
            limits,
            limits_resolution,
            log,
            mam_include_total,
            mam_smart_enable,
            max_archive_query_results,
            modules_disabled,
            modules_enabled,
            muc_log_all_rooms,
            muc_log_by_default,
            muc_log_expires_after,
            pidfile,
            plugin_paths,
            reload_global_modules,
            reload_modules,
            restrict_room_creation,
            s2s_allow_encryption,
            s2s_close_timeout,
            s2s_direct_tls_ports,
            s2s_insecure_domains,
            s2s_ports,
            s2s_require_encryption,
            s2s_secure_auth,
            s2s_secure_domains,
            s2s_send_queue_size,
            s2s_stanza_size_limit,
            s2s_tcp_keepalives,
            s2s_timeout,
            s2s_whitelist,
            sql,
            sql_manage_tables,
            sqlite_tune,
            ssl,
            ssl_compression,
            ssl_ports,
            storage,
            storage_archive_item_limit,
            storage_archive_item_limit_cache_size,
            tls_profile,
            trusted_proxies,
            unlimited_jids,
            upgrade_legacy_vcards,
            use_libevent,
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

        merge!(access_control_allow_credentials);
        merge!(access_control_allow_headers);
        merge!(access_control_allow_methods);
        merge!(access_control_allow_origins);
        merge!(access_control_max_age);
        merge!(admin_socket);
        merge!(admins);
        merge!(allow_registration);
        merge!(archive_cleanup_date_cache_size);
        merge!(archive_expires_after);
        merge!(archive_store);
        merge!(authentication);
        merge!(c2s_close_timeout);
        merge!(c2s_direct_tls_ports);
        merge!(c2s_ports);
        merge!(c2s_require_encryption);
        merge!(c2s_stanza_size_limit);
        merge!(c2s_tcp_keepalives);
        merge!(c2s_timeout);
        merge!(consider_websocket_secure);
        merge!(contact_info);
        merge!(cross_domain_websocket);
        merge!(default_archive_policy);
        merge!(default_storage);
        merge!(disco_expose_admins);
        merge!(disco_hidden);
        merge!(disco_items);
        merge!(dont_archive_namespaces);
        merge!(groups_file);
        merge!(http_default_cors_enabled);
        merge!(http_default_host);
        merge!(http_external_url);
        merge!(http_file_share_access);
        merge!(http_file_share_allowed_file_types);
        merge!(http_file_share_base_url);
        merge!(http_file_share_daily_quota);
        merge!(http_file_share_expires_after);
        merge!(http_file_share_global_quota);
        merge!(http_file_share_safe_file_types);
        merge!(http_file_share_secret);
        merge!(http_file_share_size_limit);
        merge!(http_host);
        merge!(http_interfaces);
        merge!(http_legacy_x_forwarded);
        merge!(http_max_buffer_size);
        merge!(http_max_content_size);
        merge!(http_paths);
        merge!(http_ports);
        merge!(https_interfaces);
        merge!(https_ports);
        merge!(interfaces);
        merge!(limits);
        merge!(limits_resolution);
        merge!(log);
        merge!(mam_include_total);
        merge!(mam_smart_enable);
        merge!(max_archive_query_results);
        merge!(modules_disabled);
        merge!(modules_enabled);
        merge!(muc_log_all_rooms);
        merge!(muc_log_by_default);
        merge!(muc_log_expires_after);
        merge!(pidfile);
        merge!(plugin_paths);
        merge!(reload_global_modules);
        merge!(reload_modules);
        merge!(restrict_room_creation);
        merge!(s2s_allow_encryption);
        merge!(s2s_close_timeout);
        merge!(s2s_direct_tls_ports);
        merge!(s2s_insecure_domains);
        merge!(s2s_ports);
        merge!(s2s_require_encryption);
        merge!(s2s_secure_auth);
        merge!(s2s_secure_domains);
        merge!(s2s_send_queue_size);
        merge!(s2s_stanza_size_limit);
        merge!(s2s_tcp_keepalives);
        merge!(s2s_timeout);
        merge!(s2s_whitelist);
        merge!(sql);
        merge!(sql_manage_tables);
        merge!(sqlite_tune);
        merge!(ssl);
        merge!(ssl_compression);
        merge!(ssl_ports);
        merge!(storage);
        merge!(storage_archive_item_limit);
        merge!(storage_archive_item_limit_cache_size);
        merge!(tls_profile);
        merge!(trusted_proxies);
        merge!(unlimited_jids);
        merge!(upgrade_legacy_vcards);
        merge!(use_libevent);

        // NOTE: We cannot simply use `Vec::extend` on `self.custom_settings`,
        //   as it would make values in `other` take precedence over values in
        //   `self` (which is the opposite of what happens for other keys).
        //   For consistency, we have to prepend values of `other` (Prosody
        //   settings appearing last taking precedence over previous ones).
        //   We cannot extend `custom_settings` with `self.custom_settings`
        //   either as `self.custom_settings` is behind a shared reference.
        let mut new_custom_settings =
            Vec::with_capacity(custom_settings.len() + self.custom_settings.len());
        new_custom_settings.append(&mut custom_settings);
        new_custom_settings.append(&mut self.custom_settings);
        self.custom_settings = new_custom_settings;
    }
}
