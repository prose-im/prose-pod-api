// prosody-config
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prosody_config::*;

#[test]
fn test_serializing_enums() -> Result<(), serde_json::Error> {
    let storage = StorageConfig::Raw(StorageBackend::SQL);
    assert_eq!(serde_json::to_string(&storage)?, r#""sql""#);

    let storage = StorageConfig::Map(
        [("roster".to_owned(), StorageBackend::SQL)]
            .into_iter()
            .collect(),
    );
    assert_eq!(serde_json::to_string(&storage)?, r#"{"roster":"sql"}"#);

    let interface = Interface::Address("127.0.0.1".to_owned());
    assert_eq!(serde_json::to_string(&interface)?, r#""127.0.0.1""#);

    Ok(())
}
