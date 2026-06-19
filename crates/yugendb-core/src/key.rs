//! Namespace, collection name, and key validation.

use std::fmt::{Display, Formatter};

use crate::{Result, YugenDbError};

const DEFAULT_NAME: &str = "default";

fn validate_component(kind: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(match kind {
            "namespace" => YugenDbError::invalid_namespace("namespace must not be empty"),
            "collection" => YugenDbError::invalid_collection("collection name must not be empty"),
            "key" => YugenDbError::invalid_key("key must not be empty"),
            _ => YugenDbError::internal_error("unknown key component kind"),
        });
    }

    if value.chars().any(char::is_control) {
        return Err(match kind {
            "namespace" => {
                YugenDbError::invalid_namespace("namespace must not contain control characters")
            }
            "collection" => YugenDbError::invalid_collection(
                "collection name must not contain control characters",
            ),
            "key" => YugenDbError::invalid_key("key must not contain control characters"),
            _ => YugenDbError::internal_error("unknown key component kind"),
        });
    }

    Ok(())
}

/// Logical storage partition above collections.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Namespace(String);

impl Namespace {
    /// Creates a validated namespace.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_component("namespace", &value)?;
        Ok(Self(value))
    }

    /// Returns the default namespace.
    #[must_use]
    pub fn default_namespace() -> Self {
        Self(DEFAULT_NAME.to_owned())
    }

    /// Returns the namespace as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Namespace {
    fn default() -> Self {
        Self::default_namespace()
    }
}

impl TryFrom<String> for Namespace {
    type Error = YugenDbError;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Namespace {
    type Error = YugenDbError;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl AsRef<str> for Namespace {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for Namespace {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Logical collection name inside a namespace.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CollectionName(String);

impl CollectionName {
    /// Creates a validated collection name.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_component("collection", &value)?;
        Ok(Self(value))
    }

    /// Returns the default collection name.
    #[must_use]
    pub fn default_collection() -> Self {
        Self(DEFAULT_NAME.to_owned())
    }

    /// Returns the collection name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CollectionName {
    fn default() -> Self {
        Self::default_collection()
    }
}

impl TryFrom<String> for CollectionName {
    type Error = YugenDbError;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for CollectionName {
    type Error = YugenDbError;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl AsRef<str> for CollectionName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for CollectionName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Stable key inside a namespace and collection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(String);

impl Key {
    /// Creates a validated key.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_component("key", &value)?;
        Ok(Self(value))
    }

    /// Returns the key as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Key {
    type Error = YugenDbError;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Key {
    type Error = YugenDbError;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for Key {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
