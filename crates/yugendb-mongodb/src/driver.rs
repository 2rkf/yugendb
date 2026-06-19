//! MongoDB driver implementation.
//!
//! MongoDB stores yugendb records as documents. MongoDB remains an
//! implementation detail. Applications use the same namespace, collection, key,
//! and serialised value API as the other yugendb drivers.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mongodb::{
    bson::{doc, spec::BinarySubtype, Binary, Bson, Document},
    options::{FindOptions, IndexOptions, UpdateOptions},
    Client, Collection, IndexModel,
};
use parking_lot::Mutex;
use yugendb_core::{
    Batch, BatchOperation, BatchResult, Capabilities, CollectionName, DeleteOptions, Driver, Key,
    Namespace, ReadOptions, Result, ScanOptions, StoredValue, ValueBytes, ValueMetadata,
    WriteOptions, YugenDbError,
};

use crate::document::{
    CODEC_FIELD, COLLECTION_FIELD, CREATED_AT_FIELD, EXPIRES_AT_FIELD, KEY_FIELD, NAMESPACE_FIELD,
    STORAGE_COLLECTION_NAME, UPDATED_AT_FIELD, VALUE_FIELD,
};

/// Options used to configure the MongoDB driver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MongoDbDriverOptions {
    /// MongoDB connection string.
    pub connection_string: String,
    /// Database name to store yugendb documents in.
    pub database: String,
    /// Backing MongoDB collection name.
    pub storage_collection: String,
    /// Whether indexes should be created during initialisation.
    pub create_indexes_on_initialise: bool,
}

impl MongoDbDriverOptions {
    /// Creates default MongoDB options for a connection string.
    #[must_use]
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
            database: "yugendb".to_string(),
            storage_collection: STORAGE_COLLECTION_NAME.to_string(),
            create_indexes_on_initialise: true,
        }
    }
}

/// MongoDB document-backed driver.
#[derive(Debug)]
pub struct MongoDbDriver {
    options: MongoDbDriverOptions,
    client: Mutex<Option<Client>>,
}

impl MongoDbDriver {
    /// Creates a new MongoDB driver using default options.
    #[must_use]
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self::with_options(MongoDbDriverOptions::new(connection_string))
    }

    /// Creates a new MongoDB driver with explicit options.
    #[must_use]
    pub fn with_options(options: MongoDbDriverOptions) -> Self {
        Self {
            options,
            client: Mutex::new(None),
        }
    }

    /// Returns configured options.
    #[must_use]
    pub const fn options(&self) -> &MongoDbDriverOptions {
        &self.options
    }

    fn map_error(error: mongodb::error::Error) -> YugenDbError {
        YugenDbError::driver_error("mongodb", error.to_string())
    }

    fn client(&self) -> Result<Client> {
        self.client.lock().as_ref().cloned().ok_or_else(|| {
            YugenDbError::connection_error("MongoDB driver has not been initialised")
        })
    }

    fn store_collection(&self) -> Result<Collection<Document>> {
        let client = self.client()?;
        Ok(client
            .database(&self.options.database)
            .collection(&self.options.storage_collection))
    }

    fn identity_filter(namespace: &Namespace, collection: &CollectionName, key: &Key) -> Document {
        let mut filter = Document::new();
        filter.insert(NAMESPACE_FIELD, namespace.as_str());
        filter.insert(COLLECTION_FIELD, collection.as_str());
        filter.insert(KEY_FIELD, key.as_str());
        filter
    }

    fn document_from_value(
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        value: &StoredValue,
    ) -> Document {
        let mut document = Document::new();
        document.insert(NAMESPACE_FIELD, namespace.as_str());
        document.insert(COLLECTION_FIELD, collection.as_str());
        document.insert(KEY_FIELD, key.as_str());
        document.insert(
            VALUE_FIELD,
            Bson::Binary(Binary {
                subtype: BinarySubtype::Generic,
                bytes: value.bytes.as_slice().to_vec(),
            }),
        );
        document.insert(CODEC_FIELD, value.metadata.codec.clone());
        document.insert(CREATED_AT_FIELD, value.metadata.created_at.to_rfc3339());
        document.insert(UPDATED_AT_FIELD, value.metadata.updated_at.to_rfc3339());
        match value.metadata.expires_at {
            Some(expires_at) => document.insert(EXPIRES_AT_FIELD, expires_at.to_rfc3339()),
            None => document.insert(EXPIRES_AT_FIELD, Bson::Null),
        };
        document
    }

    fn value_from_document(document: Document) -> Result<StoredValue> {
        let bytes = match document.get(VALUE_FIELD) {
            Some(Bson::Binary(binary)) => binary.bytes.clone(),
            _ => {
                return Err(YugenDbError::driver_error(
                    "mongodb",
                    "stored document has invalid value field",
                ))
            }
        };
        let codec = document
            .get_str(CODEC_FIELD)
            .map_err(|error| YugenDbError::driver_error("mongodb", error.to_string()))?
            .to_owned();
        let created_at = DateTime::parse_from_rfc3339(
            document
                .get_str(CREATED_AT_FIELD)
                .map_err(|error| YugenDbError::driver_error("mongodb", error.to_string()))?,
        )
        .map_err(|error| YugenDbError::driver_error("mongodb", error.to_string()))?
        .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(
            document
                .get_str(UPDATED_AT_FIELD)
                .map_err(|error| YugenDbError::driver_error("mongodb", error.to_string()))?,
        )
        .map_err(|error| YugenDbError::driver_error("mongodb", error.to_string()))?
        .with_timezone(&Utc);
        let expires_at = match document.get(EXPIRES_AT_FIELD) {
            Some(Bson::String(value)) => Some(
                DateTime::parse_from_rfc3339(value)
                    .map_err(|error| YugenDbError::driver_error("mongodb", error.to_string()))?
                    .with_timezone(&Utc),
            ),
            Some(Bson::Null) | None => None,
            _ => {
                return Err(YugenDbError::driver_error(
                    "mongodb",
                    "stored document has invalid expiresAt field",
                ))
            }
        };
        Ok(StoredValue::new(
            ValueBytes::from(bytes),
            ValueMetadata {
                codec,
                created_at,
                updated_at,
                expires_at,
            },
        ))
    }
}

impl Clone for MongoDbDriver {
    fn clone(&self) -> Self {
        Self::with_options(self.options.clone())
    }
}

/// Capabilities for the implemented MongoDB driver.
#[must_use]
pub const fn mongodb_capabilities() -> Capabilities {
    Capabilities {
        transactions: false,
        ttl: true,
        prefix_scan: true,
        atomic_increment: false,
        batch_write: true,
        raw_sql: false,
        document_query: false,
        json_query: false,
        migrations: false,
        connection_pooling: true,
        watch: false,
        backup: false,
    }
}

/// Convenience constructor for the MongoDB driver.
#[must_use]
pub fn mongodb(connection_string: impl Into<String>) -> MongoDbDriver {
    MongoDbDriver::new(connection_string)
}

#[async_trait]
impl Driver for MongoDbDriver {
    fn name(&self) -> &'static str {
        "mongodb"
    }
    fn capabilities(&self) -> Capabilities {
        mongodb_capabilities()
    }

    async fn initialise(&self) -> Result<()> {
        let client = Client::with_uri_str(&self.options.connection_string)
            .await
            .map_err(Self::map_error)?;
        if self.options.create_indexes_on_initialise {
            let collection = client
                .database(&self.options.database)
                .collection::<Document>(&self.options.storage_collection);
            let identity = IndexModel::builder()
                .keys({
                    let mut keys = Document::new();
                    keys.insert(NAMESPACE_FIELD, 1);
                    keys.insert(COLLECTION_FIELD, 1);
                    keys.insert(KEY_FIELD, 1);
                    keys
                })
                .options(
                    IndexOptions::builder()
                        .unique(true)
                        .name("idx_yugendb_identity".to_owned())
                        .build(),
                )
                .build();
            collection
                .create_index(identity, None)
                .await
                .map_err(Self::map_error)?;
            collection
                .create_index(
                    IndexModel::builder()
                        .keys({
                            let mut keys = Document::new();
                            keys.insert(EXPIRES_AT_FIELD, 1);
                            keys
                        })
                        .build(),
                    None,
                )
                .await
                .map_err(Self::map_error)?;
        }
        *self.client.lock() = Some(client);
        Ok(())
    }

    async fn finalise(&self) -> Result<()> {
        *self.client.lock() = None;
        Ok(())
    }

    async fn get(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &ReadOptions,
    ) -> Result<Option<StoredValue>> {
        let store = self.store_collection()?;
        let document = store
            .find_one(Self::identity_filter(namespace, collection, key), None)
            .await
            .map_err(Self::map_error)?;
        let Some(document) = document else {
            return Ok(None);
        };
        let value = Self::value_from_document(document)?;
        Ok((options.allow_expired || !value.is_expired()).then_some(value))
    }

    async fn set(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        value: StoredValue,
        options: &WriteOptions,
    ) -> Result<()> {
        if !options.overwrite && self.exists(namespace, collection, key).await? {
            return Err(YugenDbError::conflict(
                "MongoDB driver refused to overwrite an existing value",
            ));
        }
        let store = self.store_collection()?;
        let document = Self::document_from_value(namespace, collection, key, &value);
        store
            .update_one(
                Self::identity_filter(namespace, collection, key),
                doc! { "$set": document },
                Some(UpdateOptions::builder().upsert(true).build()),
            )
            .await
            .map_err(Self::map_error)?;
        Ok(())
    }

    async fn delete(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &DeleteOptions,
    ) -> Result<bool> {
        let store = self.store_collection()?;
        let result = store
            .delete_one(Self::identity_filter(namespace, collection, key), None)
            .await
            .map_err(Self::map_error)?;
        let removed = result.deleted_count > 0;
        if options.must_exist && !removed {
            return Err(YugenDbError::not_found(
                "MongoDB driver could not delete a missing value",
            ));
        }
        Ok(removed)
    }

    async fn exists(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
    ) -> Result<bool> {
        Ok(self
            .get(namespace, collection, key, &ReadOptions::default())
            .await?
            .is_some())
    }

    async fn scan_prefix(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        options: &ScanOptions,
    ) -> Result<Vec<(Key, StoredValue)>> {
        let store = self.store_collection()?;
        let prefix = regex_escape(options.prefix.as_deref().unwrap_or(""));
        let mut key_filter = Document::new();
        key_filter.insert("$regex", format!("^{prefix}"));
        let mut filter = Document::new();
        filter.insert(NAMESPACE_FIELD, namespace.as_str());
        filter.insert(COLLECTION_FIELD, collection.as_str());
        filter.insert(KEY_FIELD, key_filter);
        let find_options = FindOptions::builder()
            .sort({
                let mut sort = Document::new();
                sort.insert(KEY_FIELD, 1);
                sort
            })
            .build();
        let mut cursor = store
            .find(filter, Some(find_options))
            .await
            .map_err(Self::map_error)?;
        let mut values = Vec::new();
        while cursor.advance().await.map_err(Self::map_error)? {
            let document = cursor.deserialize_current().map_err(Self::map_error)?;
            let key_text = document
                .get_str(KEY_FIELD)
                .map_err(|error| YugenDbError::driver_error("mongodb", error.to_string()))?
                .to_owned();
            let value = Self::value_from_document(document)?;
            if value.is_expired() && !options.include_expired {
                continue;
            }
            values.push((Key::try_from(key_text)?, value));
            if let Some(limit) = options.limit {
                if values.len() >= limit {
                    break;
                }
            }
        }
        Ok(values)
    }

    async fn batch(&self, batch: Batch) -> Result<BatchResult> {
        let mut set_count = 0;
        let mut delete_count = 0;
        for operation in batch.into_operations() {
            match operation {
                BatchOperation::Set {
                    namespace,
                    collection,
                    key,
                    value,
                } => {
                    self.set(
                        &namespace,
                        &collection,
                        &key,
                        value,
                        &WriteOptions::default(),
                    )
                    .await?;
                    set_count += 1;
                }
                BatchOperation::Delete {
                    namespace,
                    collection,
                    key,
                } => {
                    if self
                        .delete(&namespace, &collection, &key, &DeleteOptions::default())
                        .await?
                    {
                        delete_count += 1;
                    }
                }
            }
        }
        Ok(BatchResult::new(set_count, delete_count))
    }
}

fn regex_escape(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\' => {
                escaped.push('\\');
                escaped.push(character);
            }
            other => escaped.push(other),
        }
    }
    escaped
}
