use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use yugendb_core::{
    Capabilities, CodecExt, CollectionName, ErrorCode, JsonCodec, Key, Namespace, StoredValue,
    ValueBytes, ValueMetadata,
};

#[test]
fn valid_namespace_is_accepted() {
    let namespace = Namespace::try_from("app").expect("namespace should be valid");
    assert_eq!(namespace.as_str(), "app");
}

#[test]
fn invalid_namespace_is_rejected() {
    let error = Namespace::try_from("").expect_err("empty namespace should fail");
    assert_eq!(error.code(), ErrorCode::InvalidNamespace);
}

#[test]
fn valid_key_is_accepted() {
    let key = Key::try_from("users/reader").expect("key should be valid");
    assert_eq!(key.as_str(), "users/reader");
}

#[test]
fn invalid_key_is_rejected() {
    let error = Key::try_from(
        "bad
key",
    )
    .expect_err("control characters should fail");
    assert_eq!(error.code(), ErrorCode::InvalidKey);
}

#[test]
fn collection_default_is_default() {
    let collection = CollectionName::default();
    assert_eq!(collection.as_str(), "default");
}

#[test]
fn error_codes_are_stable_strings() {
    assert_eq!(ErrorCode::NotFound.as_str(), "NOT_FOUND");
    assert_eq!(
        ErrorCode::SerialisationError.as_str(),
        "SERIALISATION_ERROR"
    );
    assert_eq!(
        ErrorCode::DeserialisationError.as_str(),
        "DESERIALISATION_ERROR"
    );
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct User {
    display_name: String,
}

#[test]
fn json_codec_serialises_and_deserialises() {
    let codec = JsonCodec;
    let user = User {
        display_name: "Reader".to_owned(),
    };

    let bytes = codec.serialise(&user).expect("serialise user");
    let loaded: User = codec.deserialise(&bytes).expect("deserialise user");

    assert_eq!(loaded, user);
}

#[test]
fn json_codec_reports_deserialisation_failure() {
    let codec = JsonCodec;
    let bytes = ValueBytes::from(vec![b'{']);
    let error = codec
        .deserialise::<User>(&bytes)
        .expect_err("invalid JSON should fail");

    assert_eq!(error.code(), ErrorCode::DeserialisationError);
}

#[test]
fn stored_value_expiry_is_detected() {
    let now = Utc::now();
    let metadata = ValueMetadata::new("json", now, Some(now - Duration::seconds(1)));
    let stored = StoredValue::new(ValueBytes::from(vec![1, 2, 3]), metadata);

    assert!(stored.is_expired());
}

#[test]
fn capabilities_helpers_are_coherent() {
    let minimal = Capabilities::minimal();
    assert!(!minimal.prefix_scan);
    assert!(!minimal.batch_write);

    let memory = Capabilities::memory();
    assert!(memory.ttl);
    assert!(memory.prefix_scan);
    assert!(memory.batch_write);

    let sqlite = Capabilities::sqlite();
    assert!(sqlite.transactions);
    assert!(!sqlite.raw_sql);
}
