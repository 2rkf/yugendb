//! Structured error model for yugendb.

use thiserror::Error;

/// Result alias used throughout yugendb.
pub type Result<T> = std::result::Result<T, YugenDbError>;

/// Stable cross-language error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// A requested record was not found when the operation required it.
    NotFound,
    /// A backend connection could not be opened or used.
    ConnectionError,
    /// A value could not be serialised.
    SerialisationError,
    /// Stored bytes could not be deserialised into the requested type.
    DeserialisationError,
    /// An operation exceeded its allowed time.
    Timeout,
    /// An operation conflicted with existing data or concurrent work.
    Conflict,
    /// A transaction was aborted.
    TransactionAborted,
    /// The selected driver does not support the requested feature.
    UnsupportedFeature,
    /// A storage constraint was violated.
    ConstraintViolation,
    /// A key was invalid.
    InvalidKey,
    /// A namespace was invalid.
    InvalidNamespace,
    /// A collection name was invalid.
    InvalidCollection,
    /// A value was invalid.
    InvalidValue,
    /// A driver reported a backend-specific failure.
    DriverError,
    /// A yugendb invariant failed.
    InternalError,
}

impl ErrorCode {
    /// Returns the stable string used in public APIs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "NOT_FOUND",
            Self::ConnectionError => "CONNECTION_ERROR",
            Self::SerialisationError => "SERIALISATION_ERROR",
            Self::DeserialisationError => "DESERIALISATION_ERROR",
            Self::Timeout => "TIMEOUT",
            Self::Conflict => "CONFLICT",
            Self::TransactionAborted => "TRANSACTION_ABORTED",
            Self::UnsupportedFeature => "UNSUPPORTED_FEATURE",
            Self::ConstraintViolation => "CONSTRAINT_VIOLATION",
            Self::InvalidKey => "INVALID_KEY",
            Self::InvalidNamespace => "INVALID_NAMESPACE",
            Self::InvalidCollection => "INVALID_COLLECTION",
            Self::InvalidValue => "INVALID_VALUE",
            Self::DriverError => "DRIVER_ERROR",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Main structured error type for yugendb.
#[derive(Debug, Error)]
pub enum YugenDbError {
    /// A requested record was not found when the operation required it.
    #[error("record was not found: {message}")]
    NotFound { message: String },

    /// A backend connection could not be opened or used.
    #[error("connection error: {message}")]
    ConnectionError { message: String },

    /// A value could not be serialised.
    #[error("serialisation error: {message}")]
    SerialisationError { message: String },

    /// Stored bytes could not be deserialised into the requested type.
    #[error("deserialisation error: {message}")]
    DeserialisationError { message: String },

    /// An operation exceeded its allowed time.
    #[error("operation timed out: {message}")]
    Timeout { message: String },

    /// An operation conflicted with existing data or concurrent work.
    #[error("conflict: {message}")]
    Conflict { message: String },

    /// A transaction was aborted.
    #[error("transaction aborted: {message}")]
    TransactionAborted { message: String },

    /// The selected driver does not support the requested feature.
    #[error("unsupported feature `{feature}`: {message}")]
    UnsupportedFeature { feature: String, message: String },

    /// A storage constraint was violated.
    #[error("constraint violation: {message}")]
    ConstraintViolation { message: String },

    /// A key was invalid.
    #[error("invalid key: {message}")]
    InvalidKey { message: String },

    /// A namespace was invalid.
    #[error("invalid namespace: {message}")]
    InvalidNamespace { message: String },

    /// A collection name was invalid.
    #[error("invalid collection: {message}")]
    InvalidCollection { message: String },

    /// A value was invalid.
    #[error("invalid value: {message}")]
    InvalidValue { message: String },

    /// A driver reported a backend-specific failure.
    #[error("driver error from `{driver}`: {message}")]
    DriverError { driver: String, message: String },

    /// A yugendb invariant failed.
    #[error("internal error: {message}")]
    InternalError { message: String },
}

impl YugenDbError {
    /// Returns the stable cross-language error code.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::NotFound { .. } => ErrorCode::NotFound,
            Self::ConnectionError { .. } => ErrorCode::ConnectionError,
            Self::SerialisationError { .. } => ErrorCode::SerialisationError,
            Self::DeserialisationError { .. } => ErrorCode::DeserialisationError,
            Self::Timeout { .. } => ErrorCode::Timeout,
            Self::Conflict { .. } => ErrorCode::Conflict,
            Self::TransactionAborted { .. } => ErrorCode::TransactionAborted,
            Self::UnsupportedFeature { .. } => ErrorCode::UnsupportedFeature,
            Self::ConstraintViolation { .. } => ErrorCode::ConstraintViolation,
            Self::InvalidKey { .. } => ErrorCode::InvalidKey,
            Self::InvalidNamespace { .. } => ErrorCode::InvalidNamespace,
            Self::InvalidCollection { .. } => ErrorCode::InvalidCollection,
            Self::InvalidValue { .. } => ErrorCode::InvalidValue,
            Self::DriverError { .. } => ErrorCode::DriverError,
            Self::InternalError { .. } => ErrorCode::InternalError,
        }
    }

    /// Creates a not found error.
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
        }
    }

    /// Creates a connection error.
    #[must_use]
    pub fn connection_error(message: impl Into<String>) -> Self {
        Self::ConnectionError {
            message: message.into(),
        }
    }

    /// Creates a serialisation error.
    #[must_use]
    pub fn serialisation_error(message: impl Into<String>) -> Self {
        Self::SerialisationError {
            message: message.into(),
        }
    }

    /// Creates a deserialisation error.
    #[must_use]
    pub fn deserialisation_error(message: impl Into<String>) -> Self {
        Self::DeserialisationError {
            message: message.into(),
        }
    }

    /// Creates a timeout error.
    #[must_use]
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            message: message.into(),
        }
    }

    /// Creates a conflict error.
    #[must_use]
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    /// Creates a transaction aborted error.
    #[must_use]
    pub fn transaction_aborted(message: impl Into<String>) -> Self {
        Self::TransactionAborted {
            message: message.into(),
        }
    }

    /// Creates an unsupported feature error.
    #[must_use]
    pub fn unsupported_feature(feature: impl Into<String>, message: impl Into<String>) -> Self {
        Self::UnsupportedFeature {
            feature: feature.into(),
            message: message.into(),
        }
    }

    /// Creates a constraint violation error.
    #[must_use]
    pub fn constraint_violation(message: impl Into<String>) -> Self {
        Self::ConstraintViolation {
            message: message.into(),
        }
    }

    /// Creates an invalid key error.
    #[must_use]
    pub fn invalid_key(message: impl Into<String>) -> Self {
        Self::InvalidKey {
            message: message.into(),
        }
    }

    /// Creates an invalid namespace error.
    #[must_use]
    pub fn invalid_namespace(message: impl Into<String>) -> Self {
        Self::InvalidNamespace {
            message: message.into(),
        }
    }

    /// Creates an invalid collection error.
    #[must_use]
    pub fn invalid_collection(message: impl Into<String>) -> Self {
        Self::InvalidCollection {
            message: message.into(),
        }
    }

    /// Creates an invalid value error.
    #[must_use]
    pub fn invalid_value(message: impl Into<String>) -> Self {
        Self::InvalidValue {
            message: message.into(),
        }
    }

    /// Creates a driver error.
    #[must_use]
    pub fn driver_error(driver: impl Into<String>, message: impl Into<String>) -> Self {
        Self::DriverError {
            driver: driver.into(),
            message: message.into(),
        }
    }

    /// Creates an invariant error.
    #[must_use]
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }
}

impl From<std::convert::Infallible> for YugenDbError {
    fn from(error: std::convert::Infallible) -> Self {
        match error {}
    }
}
