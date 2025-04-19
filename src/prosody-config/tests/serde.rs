// prosody-config
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use prosody_config::*;

#[test]
fn test_serializing_enums() -> Result<(), serde_json::Error> {
    let storage = StorageConfig::Raw(StorageBackend::Sql);
    assert_eq!(serde_json::to_string(&storage)?, r#""sql""#);
    assert_eq!(StorageBackend::from_str("sql"), Ok(StorageBackend::Sql));
    assert_eq!(
        StorageBackend::from_str("appendmap"),
        Ok(StorageBackend::Other("appendmap".to_owned())),
    );

    let storage = StorageConfig::Map(
        [
            ("accounts".to_owned(), StorageBackend::Sql),
            (
                "roster".to_owned(),
                StorageBackend::Other("appendmap".to_owned()),
            ),
        ]
        .into_iter()
        .collect(),
    );
    let expected = r#"{"accounts":"sql","roster":"appendmap"}"#;
    assert_eq!(serde_json::to_string(&storage)?, expected);
    assert_eq!(serde_json::from_str::<StorageConfig>(expected)?, storage);

    let interface = Interface::Address("127.0.0.1".to_owned());
    assert_eq!(serde_json::to_string(&interface)?, r#""127.0.0.1""#);

    Ok(())
}

#[test]
fn test_deserializing_unsupported_keys() -> Result<(), serde_json::Error> {
    let json = r#"{"log":"warn"}"#;
    assert!(serde_json::from_str::<ProsodySettings>(json).is_err());

    let json = r#"{}"#;
    assert_eq!(
        serde_json::from_str::<ProsodySettings>(json)?,
        ProsodySettings::default(),
    );

    let json = r#"{"default_storage":"sql"}"#;
    assert_eq!(
        serde_json::from_str::<ProsodySettings>(json)?,
        ProsodySettings {
            default_storage: Some(StorageBackend::Sql),
            ..Default::default()
        },
    );

    Ok(())
}
