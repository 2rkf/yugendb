//! MongoDB document mapping helpers for yugendb.

/// Default MongoDB collection used by the driver.
pub const STORAGE_COLLECTION_NAME: &str = "yugendb_store";
/// Current document mapping version.
pub const DOCUMENT_SCHEMA_VERSION: u32 = 1;

/// Field used for the yugendb namespace.
pub const NAMESPACE_FIELD: &str = "namespace";
/// Field used for the yugendb collection.
pub const COLLECTION_FIELD: &str = "collection";
/// Field used for the yugendb key.
pub const KEY_FIELD: &str = "key";
/// Field used for serialised bytes.
pub const VALUE_FIELD: &str = "value";
/// Field used for codec name.
pub const CODEC_FIELD: &str = "codec";
/// Field used for creation time.
pub const CREATED_AT_FIELD: &str = "createdAt";
/// Field used for update time.
pub const UPDATED_AT_FIELD: &str = "updatedAt";
/// Field used for optional expiry time.
pub const EXPIRES_AT_FIELD: &str = "expiresAt";

/// Ordered field list exposed by the schema helper.
pub const STORED_DOCUMENT_FIELDS: [&str; 8] = [
    NAMESPACE_FIELD,
    COLLECTION_FIELD,
    KEY_FIELD,
    VALUE_FIELD,
    CODEC_FIELD,
    CREATED_AT_FIELD,
    UPDATED_AT_FIELD,
    EXPIRES_AT_FIELD,
];

/// Returns the compound identity index fields.
#[must_use]
pub const fn identity_index_fields() -> [&'static str; 3] {
    [NAMESPACE_FIELD, COLLECTION_FIELD, KEY_FIELD]
}
