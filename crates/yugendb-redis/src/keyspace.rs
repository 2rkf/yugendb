//! Redis keyspace helpers for yugendb.

/// Default key prefix for Redis storage.
pub const DEFAULT_KEY_PREFIX: &str = "yugendb";
/// Separator used for readable Redis keys.
pub const KEY_SEPARATOR: &str = ":";

/// Description of the Redis keyspace used by a driver instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedisKeyspace {
    /// Prefix added to all Redis keys created by the driver.
    pub key_prefix: String,
}

impl RedisKeyspace {
    /// Creates keyspace metadata for a prefix.
    #[must_use]
    pub fn new(key_prefix: impl Into<String>) -> Self {
        Self {
            key_prefix: key_prefix.into(),
        }
    }
}

fn encode_component(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' => {
                encoded.push(*byte as char)
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}

fn decode_component(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return None;
            }
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
            let byte = u8::from_str_radix(hex, 16).ok()?;
            decoded.push(byte);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

/// Builds a Redis storage key from namespace, collection, and key parts.
#[must_use]
pub fn redis_key(prefix: &str, namespace: &str, collection: &str, key: &str) -> String {
    [
        prefix,
        &encode_component(namespace),
        &encode_component(collection),
        &encode_component(key),
    ]
    .join(KEY_SEPARATOR)
}

/// Builds a prefix used for backend prefix scans.
#[must_use]
pub fn redis_prefix(prefix: &str, namespace: &str, collection: &str, key_prefix: &str) -> String {
    [
        prefix,
        &encode_component(namespace),
        &encode_component(collection),
        &encode_component(key_prefix),
    ]
    .join(KEY_SEPARATOR)
}

/// Builds a collection-wide Redis key prefix.
#[must_use]
pub fn compose_collection_prefix(prefix: &str, namespace: &str, collection: &str) -> String {
    redis_prefix(prefix, namespace, collection, "")
}

/// Alias for composing a Redis storage key.
#[must_use]
pub fn compose_storage_key(prefix: &str, namespace: &str, collection: &str, key: &str) -> String {
    redis_key(prefix, namespace, collection, key)
}

/// Decodes the user key from a Redis key when it belongs to the target collection.
#[must_use]
pub fn decode_redis_key(
    prefix: &str,
    namespace: &str,
    collection: &str,
    redis_key: &str,
) -> Option<String> {
    let prefix = compose_collection_prefix(prefix, namespace, collection);
    let suffix = redis_key.strip_prefix(&prefix)?;
    decode_component(suffix)
}
