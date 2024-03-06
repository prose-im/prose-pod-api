mod model {
    use linked_hash_map::LinkedHashMap;

    pub struct LuaComment(pub String);

    impl<S: ToString> From<S> for LuaComment {
        fn from(value: S) -> Self {
            Self(value.to_string())
        }
    }

    /// When we want to group definitions together by topic for example,
    /// we can use groups to avoid printing empty lines in-between.
    pub struct Group<T> {
        pub comment: Option<LuaComment>,
        pub elements: Vec<T>,
    }

    impl<T> From<T> for Group<T> {
        fn from(value: T) -> Self {
            Self {
                comment: None,
                elements: vec![value],
            }
        }
    }

    impl<T> From<Vec<T>> for Group<T> {
        fn from(value: Vec<T>) -> Self {
            Self {
                comment: None,
                elements: value,
            }
        }
    }

    pub struct LuaDefinition {
        pub comment: Option<LuaComment>,
        pub key: String,
        pub value: LuaValue,
    }

    pub enum LuaNumber {
        Scalar(i32),
        Product(Box<LuaNumber>, Box<LuaNumber>),
    }

    impl From<i32> for LuaNumber {
        fn from(value: i32) -> Self {
            LuaNumber::Scalar(value)
        }
    }

    pub enum LuaValue {
        Bool(bool),
        Number(LuaNumber),
        String(String),
        List(Vec<LuaValue>),
        Map(LinkedHashMap<String, LuaValue>),
    }

    impl From<bool> for LuaValue {
        fn from(value: bool) -> Self {
            Self::Bool(value)
        }
    }

    impl From<LuaNumber> for LuaValue {
        fn from(value: LuaNumber) -> Self {
            LuaValue::Number(value)
        }
    }

    impl From<i32> for LuaValue {
        fn from(value: i32) -> Self {
            Self::Number(LuaNumber::Scalar(value))
        }
    }

    impl From<&str> for LuaValue {
        fn from(value: &str) -> Self {
            Self::String(value.to_string())
        }
    }

    impl<V: Into<LuaValue>> From<Vec<V>> for LuaValue {
        fn from(value: Vec<V>) -> Self {
            Self::List(value.into_iter().map(Into::into).collect())
        }
    }

    pub enum ProsodyConfigSection {
        VirtualHost {
            comments: Vec<LuaComment>,
            hostname: String,
            settings: Vec<Group<LuaDefinition>>,
        },
        Component {
            comments: Vec<LuaComment>,
            hostname: String,
            plugin: String,
            name: String,
            settings: Vec<Group<LuaDefinition>>,
        },
    }

    pub struct ProsodyConfigFile {
        pub header: Option<Group<LuaComment>>,
        pub global_settings: Vec<Group<LuaDefinition>>,
        pub additional_sections: Vec<ProsodyConfigSection>,
    }
}

mod print {
    use super::model::*;

    const INDENT: &'static str = "  ";

    trait Print {
        fn print(&self, acc: &mut String, indent: u8);
    }

    impl<T: Print> Print for Option<T> {
        fn print(&self, acc: &mut String, indent: u8) {
            if let Some(value) = self {
                value.print(acc, indent);
            }
        }
    }

    impl<T: Print> Print for Group<T> {
        fn print(&self, acc: &mut String, indent: u8) {
            self.comment.print(acc, indent);
            for element in self.elements.iter() {
                element.print(acc, indent);
            }
            // Add an empty line at the end of a group
            acc.push('\n');
        }
    }

    impl Print for LuaComment {
        fn print(&self, acc: &mut String, indent: u8) {
            for _ in 0..indent {
                acc.push_str(INDENT);
            }
            acc.push_str("-- ");
            acc.push_str(&self.0);
            acc.push('\n');
        }
    }

    impl Print for LuaDefinition {
        fn print(&self, acc: &mut String, indent: u8) {
            self.comment.print(acc, indent);
            for _ in 0..indent {
                acc.push_str(INDENT);
            }
            acc.push_str(&self.key);
            acc.push_str(" = ");
            self.value.print(acc, indent);
            acc.push('\n');
        }
    }

    impl Print for LuaNumber {
        fn print(&self, acc: &mut String, indent: u8) {
            match self {
                Self::Scalar(n) => acc.push_str(&format!("{n}")),
                Self::Product(lhs, rhs) => {
                    lhs.print(acc, indent);
                    acc.push_str(" * ");
                    rhs.print(acc, indent);
                }
            }
        }
    }

    impl Print for LuaValue {
        fn print(&self, acc: &mut String, indent: u8) {
            match self {
                Self::Bool(b) => acc.push_str(&format!("{b}")),
                Self::Number(n) => n.print(acc, indent),
                Self::String(s) => acc.push_str(&format!("{s:?}")),
                Self::List(list) => match list.len() {
                    0 => acc.push_str("{}"),
                    1 => {
                        acc.push_str("{ ");
                        list[0].print(acc, indent);
                        acc.push_str(" }");
                    }
                    _ => {
                        acc.push_str("{\n");
                        for element in list.iter() {
                            for _ in 0..=indent {
                                acc.push_str(INDENT);
                            }
                            element.print(acc, indent + 1);
                            acc.push_str(";\n");
                        }
                        for _ in 0..indent {
                            acc.push_str(INDENT);
                        }
                        acc.push('}');
                    }
                },
                Self::Map(map) => {
                    acc.push_str("{\n");
                    for (key, value) in map.iter() {
                        for _ in 0..=indent {
                            acc.push_str(INDENT);
                        }
                        acc.push_str(key);
                        acc.push_str(" = ");
                        value.print(acc, indent + 1);
                        acc.push_str(";\n");
                    }
                    for _ in 0..indent {
                        acc.push_str(INDENT);
                    }
                    acc.push('}');
                }
            }
        }
    }

    impl Print for ProsodyConfigSection {
        fn print(&self, acc: &mut String, indent: u8) {
            match self {
                Self::VirtualHost {
                    comments,
                    hostname,
                    settings,
                } => {
                    for comment in comments.iter() {
                        comment.print(acc, indent);
                    }
                    acc.push_str(&format!("VirtualHost {hostname:?}\n"));
                    for element in settings.iter() {
                        element.print(acc, indent + 1);
                    }
                    *acc = acc.trim_end().to_string();
                    acc.push_str("\n\n");
                }
                Self::Component {
                    comments,
                    hostname,
                    plugin,
                    name,
                    settings,
                } => {
                    for comment in comments.iter() {
                        comment.print(acc, indent);
                    }
                    acc.push_str(&format!("Component {hostname:?} {plugin:?}\n"));
                    Group {
                        comment: None,
                        elements: vec![LuaDefinition {
                            comment: None,
                            key: "name".to_string(),
                            value: name.as_str().into(),
                        }],
                    }
                    .print(acc, indent + 1);
                    for element in settings.iter() {
                        element.print(acc, indent + 1);
                    }
                    *acc = acc.trim_end().to_string();
                    acc.push_str("\n\n");
                }
            }
        }
    }

    impl Print for ProsodyConfigFile {
        fn print(&self, acc: &mut String, indent: u8) {
            self.header.print(acc, indent);
            for element in self.global_settings.iter() {
                element.print(acc, indent);
            }
            for section in self.additional_sections.iter() {
                section.print(acc, indent);
            }
            *acc = acc.trim_end().to_string();
            acc.push('\n');
        }
    }

    impl ToString for ProsodyConfigFile {
        fn to_string(&self) -> String {
            let mut acc = "".to_string();
            self.print(&mut acc, 0);
            acc.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::model::*;

    impl LuaComment {
        pub fn new<S: ToString>(s: S) -> Self {
            Self(s.to_string())
        }
    }

    impl<T> Group<T> {
        pub fn one<S: ToString>(comment: S, element: T) -> Self {
            Self {
                comment: Some(LuaComment::new(comment)),
                elements: vec![element],
            }
        }
        pub fn many<S: ToString>(comment: S, elements: Vec<T>) -> Self {
            Self {
                comment: Some(LuaComment::new(comment)),
                elements,
            }
        }
    }

    pub fn def<K: ToString, V: Into<LuaValue>>(key: K, value: V) -> LuaDefinition {
        LuaDefinition {
            comment: None,
            key: key.to_string(),
            value: value.into(),
        }
    }

    pub fn mult<LHS: Into<LuaNumber>, RHS: Into<LuaNumber>>(lhs: LHS, rhs: RHS) -> LuaNumber {
        LuaNumber::Product(Box::new(lhs.into()), Box::new(rhs.into()))
    }

    #[test]
    fn test_default_config() {
        let default_config = ProsodyConfigFile {
            header: Some(vec![
                "Prose Pod Server".into(),
                "XMPP Server Configuration".into(),
                "/!\\ This file has been automatically generated by Prose Pod API.".into(),
                "/!\\ Do NOT edit this file manually or your changes will be overriden during the next reload.".into(),
            ].into()),
            global_settings: vec![
                Group::one(
                    "Base server configuration",
                    def("pidfile", "/var/run/prosody/prosody.pid"),
                ),
                vec![
                    def("authentication", "internal_hashed"),
                    def("storage", "internal"),
                ].into(),
                def(
                    "log",
                    LuaValue::Map(vec![
                        ("info".to_string(), "*console".into()),
                        ("warn".to_string(), "*console".into()),
                        ("error".to_string(), "*console".into()),
                    ].into_iter().collect())
                ).into(),
                Group::one(
                    "Network interfaces/ports",
                    def("interfaces", vec!["*"]),
                ),
                vec![
                    def("c2s_ports", vec![5222]),
                    def("s2s_ports", vec![5269]),
                ].into(),
                vec![
                    def("http_ports", vec![5280]),
                    def("http_interfaces", vec!["*"]),
                ].into(),
                vec![
                    def("https_ports", LuaValue::List(vec![])),
                    def("https_interfaces", LuaValue::List(vec![])),
                ].into(),
                LuaDefinition {
                    comment: Some(LuaComment::new("Enabled modules")),
                    key: "modules_enabled".to_string(),
                    value: vec![
                        "roster",
                        "groups",
                        "saslauth",
                        "tls",
                        "dialback",
                        "disco",
                        "posix",
                        "smacks",
                        "private",
                        "vcard_legacy",
                        "vcard4",
                        "version",
                        "uptime",
                        "time",
                        "ping",
                        "lastactivity",
                        "pep",
                        "blocklist",
                        "limits",
                        "carbons",
                        "mam",
                        "csi",
                        "server_contact_info",
                        "websocket",
                        "s2s_bidi",
                    ].into(),
                }.into(),
                LuaDefinition {
                    comment: Some(LuaComment::new("Path to SSL key and certificate for all server domains")),
                    key: "ssl".to_string(),
                    value: LuaValue::Map(vec![
                        ("key".into(), "/etc/prosody/certs/prose.org.local.key".into()),
                        ("certificate".into(), "/etc/prosody/certs/prose.org.local.crt".into()),
                    ].into_iter().collect()),
                }.into(),
                LuaDefinition {
                    comment: Some(LuaComment::new("Disable in-band registrations (done through the Prose Pod Dashboard/API)")),
                    key: "allow_registration".to_string(),
                    value: false.into()
                }.into(),
                Group::many(
                    "Mandate highest security levels",
                    vec![
                        def("c2s_require_encryption", true),
                        def("s2s_require_encryption", true),
                        def("s2s_secure_auth", false),
                    ],
                ),
                Group::many(
                    "Enforce safety C2S/S2S limits",
                    vec![
                        def("c2s_stanza_size_limit", mult(256, 1024)),
                        def("s2s_stanza_size_limit", mult(512, 1024)),
                    ],
                ),
                def("limits", LuaValue::Map(vec![
                    ("c2s".into(), LuaValue::Map(vec![
                        ("rate".into(), "50kb/s".into()),
                        ("burst".into(), "2s".into()),
                    ].into_iter().collect())),
                    ("s2sin".into(), LuaValue::Map(vec![
                        ("rate".into(), "250kb/s".into()),
                        ("burst".into(), "4s".into()),
                    ].into_iter().collect())),
                ].into_iter().collect())).into(),
                Group::many(
                    "Allow reverse-proxying to WebSocket service over insecure local HTTP",
                    vec![
                        def("consider_websocket_secure", true),
                        def("cross_domain_websocket", true),
                    ],
                ),
                LuaDefinition {
                    comment: Some(LuaComment::new("Specify server administrator")),
                    key: "contact_info".to_string(),
                    value: LuaValue::Map(vec![
                        ("admin".into(), vec!["mailto:hostmaster@prose.org.local"].into()),
                    ].into_iter().collect()),
                }.into(),
                Group::many(
                    "MAM settings",
                    vec![
                        def("archive_expires_after", "never"),
                        def("default_archive_policy", true),
                        def("max_archive_query_results", 100)
                    ],
                ),
                LuaDefinition {
                    comment: Some(LuaComment::new("Enable vCard legacy compatibility layer")),
                    key: "upgrade_legacy_vcards".to_string(),
                    value: true.into(),
                }.into(),
                LuaDefinition {
                    comment: Some(LuaComment::new("Define server members groups file")),
                    key: "groups_file".to_string(),
                    value: "/etc/prosody/roster_groups.txt".into(),
                }.into(),
            ],
            additional_sections: vec![
                ProsodyConfigSection::VirtualHost {
                    comments: vec![
                        "Server hosts and components".into(),
                    ],
                    hostname: "prose.org.local".to_string(),
                    settings: vec![],
                },
                ProsodyConfigSection::Component {
                    comments: vec![],
                    hostname: "groups.prose.org.local".to_string(),
                    plugin: "muc".to_string(),
                    name: "Chatrooms".to_string(),
                    settings: vec![
                        def("modules_enabled", vec!["muc_mam"]).into(),
                        def("restrict_room_creation", "local").into(),
                        vec![
                            def("log_all_rooms", true),
                            def("muc_log_expires_after", "never"),
                        ].into(),
                    ],
                },
                ProsodyConfigSection::Component {
                    comments: vec![],
                    hostname: "upload.prose.org.local".to_string(),
                    plugin: "http_file_share".to_string(),
                    name: "HTTP File Upload".to_string(),
                    settings: vec![
                        vec![
                            def("http_file_share_size_limit", mult(20, mult(1024, 1024))),
                            def("http_file_share_daily_quota", mult(250, mult(1024, 1024))),
                            def("http_file_share_expires_after", -1),
                            def("http_host", "localhost"),
                            def("http_external_url", "http://localhost:5280/"),
                        ].into(),
                    ],
                },
            ],
        };

        // Value from <https://github.com/prose-im/prose-pod-system/blob/f2e353758e628c01c0923fc0e46491f1644354c9/server/etc/prosody/prosody.cfg.lua#L114>
        // (with slight modifications for style consistency)
        let expected = r#"-- Prose Pod Server
-- XMPP Server Configuration
-- /!\ This file has been automatically generated by Prose Pod API.
-- /!\ Do NOT edit this file manually or your changes will be overriden during the next reload.

-- Base server configuration
pidfile = "/var/run/prosody/prosody.pid"

authentication = "internal_hashed"
storage = "internal"

log = {
  info = "*console";
  warn = "*console";
  error = "*console";
}

-- Network interfaces/ports
interfaces = { "*" }

c2s_ports = { 5222 }
s2s_ports = { 5269 }

http_ports = { 5280 }
http_interfaces = { "*" }

https_ports = {}
https_interfaces = {}

-- Enabled modules
modules_enabled = {
  "roster";
  "groups";
  "saslauth";
  "tls";
  "dialback";
  "disco";
  "posix";
  "smacks";
  "private";
  "vcard_legacy";
  "vcard4";
  "version";
  "uptime";
  "time";
  "ping";
  "lastactivity";
  "pep";
  "blocklist";
  "limits";
  "carbons";
  "mam";
  "csi";
  "server_contact_info";
  "websocket";
  "s2s_bidi";
}

-- Path to SSL key and certificate for all server domains
ssl = {
  key = "/etc/prosody/certs/prose.org.local.key";
  certificate = "/etc/prosody/certs/prose.org.local.crt";
}

-- Disable in-band registrations (done through the Prose Pod Dashboard/API)
allow_registration = false

-- Mandate highest security levels
c2s_require_encryption = true
s2s_require_encryption = true
s2s_secure_auth = false

-- Enforce safety C2S/S2S limits
c2s_stanza_size_limit = 256 * 1024
s2s_stanza_size_limit = 512 * 1024

limits = {
  c2s = {
    rate = "50kb/s";
    burst = "2s";
  };
  s2sin = {
    rate = "250kb/s";
    burst = "4s";
  };
}

-- Allow reverse-proxying to WebSocket service over insecure local HTTP
consider_websocket_secure = true
cross_domain_websocket = true

-- Specify server administrator
contact_info = {
  admin = { "mailto:hostmaster@prose.org.local" };
}

-- MAM settings
archive_expires_after = "never"
default_archive_policy = true
max_archive_query_results = 100

-- Enable vCard legacy compatibility layer
upgrade_legacy_vcards = true

-- Define server members groups file
groups_file = "/etc/prosody/roster_groups.txt"

-- Server hosts and components
VirtualHost "prose.org.local"

Component "groups.prose.org.local" "muc"
  name = "Chatrooms"

  modules_enabled = { "muc_mam" }

  restrict_room_creation = "local"

  log_all_rooms = true
  muc_log_expires_after = "never"

Component "upload.prose.org.local" "http_file_share"
  name = "HTTP File Upload"

  http_file_share_size_limit = 20 * 1024 * 1024
  http_file_share_daily_quota = 250 * 1024 * 1024
  http_file_share_expires_after = -1
  http_host = "localhost"
  http_external_url = "http://localhost:5280/"
"#;
        let result = default_config.to_string();
        assert_eq!(result, expected, "\n\"{result}\"\n!=\n\"{expected}\"");
    }
}
