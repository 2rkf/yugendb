/** MongoDB document mapping helpers for yugendb. */
export const storageCollectionName = "yugendb_store";
export const documentSchemaVersion = 1;
export const namespaceField = "namespace";
export const collectionField = "collection";
export const keyField = "key";
export const valueField = "value";
export const codecField = "codec";
export const createdAtField = "createdAt";
export const updatedAtField = "updatedAt";
export const expiresAtField = "expiresAt";

export const storedDocumentFields = [
  namespaceField,
  collectionField,
  keyField,
  valueField,
  codecField,
  createdAtField,
  updatedAtField,
  expiresAtField,
] as const;

/** Returns the fields used to uniquely identify a yugendb value. */
export function identityIndexFields(): readonly string[] {
  return [namespaceField, collectionField, keyField];
}

/** Describes the MongoDB document shape used by the driver. */
export interface MongoDbDocumentSchema {
  readonly collectionName: string;
  readonly version: number;
  readonly fields: readonly string[];
  readonly identityIndexFields: readonly string[];
}

/** Returns the current MongoDB document schema description. */
export function mongodbDocumentSchema(): MongoDbDocumentSchema {
  return {
    collectionName: storageCollectionName,
    version: documentSchemaVersion,
    fields: storedDocumentFields,
    identityIndexFields: identityIndexFields(),
  };
}
