//! Codec model for serialising and deserialising typed values.

use serde::{de::DeserializeOwned, Serialize};

use crate::{Result, ValueBytes, YugenDbError};

/// Object-safe codec contract used by stores.
///
/// The byte-level methods keep the trait usable behind `dyn Codec`. The
/// [`CodecExt`] extension trait provides ergonomic typed `serialise` and
/// `deserialise` helpers for all codec implementations.
pub trait Codec: Send + Sync {
    /// Stable codec name stored in value metadata.
    fn name(&self) -> &'static str;

    /// Serialises a JSON value into bytes for storage.
    fn serialise_json_value(&self, value: &serde_json::Value) -> Result<ValueBytes>;

    /// Deserialises bytes into a JSON value.
    fn deserialise_json_value(&self, bytes: &ValueBytes) -> Result<serde_json::Value>;
}

/// Typed codec helpers.
pub trait CodecExt: Codec {
    /// Serialises a typed value into bytes.
    fn serialise<T>(&self, value: &T) -> Result<ValueBytes>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)
            .map_err(|error| YugenDbError::serialisation_error(error.to_string()))?;
        self.serialise_json_value(&json_value)
    }

    /// Deserialises bytes into a typed value.
    fn deserialise<T>(&self, bytes: &ValueBytes) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let json_value = self.deserialise_json_value(bytes)?;
        serde_json::from_value(json_value)
            .map_err(|error| YugenDbError::deserialisation_error(error.to_string()))
    }
}

impl<T> CodecExt for T where T: Codec + ?Sized {}

/// JSON codec using `serde_json`.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonCodec;

impl Codec for JsonCodec {
    fn name(&self) -> &'static str {
        "json"
    }

    fn serialise_json_value(&self, value: &serde_json::Value) -> Result<ValueBytes> {
        let bytes = serde_json::to_vec(value)
            .map_err(|error| YugenDbError::serialisation_error(error.to_string()))?;
        Ok(ValueBytes::from(bytes))
    }

    fn deserialise_json_value(&self, bytes: &ValueBytes) -> Result<serde_json::Value> {
        serde_json::from_slice(bytes.as_slice())
            .map_err(|error| YugenDbError::deserialisation_error(error.to_string()))
    }
}
