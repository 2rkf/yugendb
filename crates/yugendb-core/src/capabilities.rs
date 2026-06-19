//! Driver capability reporting.

/// Feature report exposed by each driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    /// Driver supports transactional operation groups.
    pub transactions: bool,
    /// Driver supports time-to-live expiry.
    pub ttl: bool,
    /// Driver supports prefix scans.
    pub prefix_scan: bool,
    /// Driver supports atomic increment operations.
    pub atomic_increment: bool,
    /// Driver supports batch writes.
    pub batch_write: bool,
    /// Driver exposes a raw SQL escape hatch.
    pub raw_sql: bool,
    /// Driver exposes a document query escape hatch.
    pub document_query: bool,
    /// Driver exposes JSON query behaviour.
    pub json_query: bool,
    /// Driver supports migration helpers.
    pub migrations: bool,
    /// Driver manages connection pooling.
    pub connection_pooling: bool,
    /// Driver supports watch or subscription behaviour.
    pub watch: bool,
    /// Driver supports backup helpers.
    pub backup: bool,
}

impl Capabilities {
    /// Minimal capability set required of every driver.
    #[must_use]
    pub const fn minimal() -> Self {
        Self {
            transactions: false,
            ttl: false,
            prefix_scan: false,
            atomic_increment: false,
            batch_write: false,
            raw_sql: false,
            document_query: false,
            json_query: false,
            migrations: false,
            connection_pooling: false,
            watch: false,
            backup: false,
        }
    }

    /// Capability shape for the memory driver.
    #[must_use]
    pub const fn memory() -> Self {
        Self {
            transactions: false,
            ttl: true,
            prefix_scan: true,
            atomic_increment: false,
            batch_write: true,
            raw_sql: false,
            document_query: false,
            json_query: false,
            migrations: false,
            connection_pooling: false,
            watch: false,
            backup: false,
        }
    }

    /// Capability shape for the SQLite driver.
    #[must_use]
    pub const fn sqlite() -> Self {
        Self {
            transactions: true,
            ttl: true,
            prefix_scan: true,
            atomic_increment: false,
            batch_write: true,
            raw_sql: false,
            document_query: false,
            json_query: false,
            migrations: true,
            connection_pooling: false,
            watch: false,
            backup: false,
        }
    }
}

impl Default for Capabilities {
    fn default() -> Self {
        Self::minimal()
    }
}
