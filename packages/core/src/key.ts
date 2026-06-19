import { YugenDbError } from "./errors";

declare const namespaceBrand: unique symbol;
declare const collectionNameBrand: unique symbol;
declare const keyBrand: unique symbol;

/** Validated namespace string used to group yugendb data. */
export type Namespace = string & { readonly [namespaceBrand]: "Namespace" };
/** Validated collection name used inside a namespace. */
export type CollectionName = string & { readonly [collectionNameBrand]: "CollectionName" };
/** Validated storage key used inside a collection. */
export type Key = string & { readonly [keyBrand]: "Key" };

/** Default namespace used when a store does not specify one. */
export const DEFAULT_NAMESPACE = "default" as Namespace;
/** Default collection used when a store does not specify one. */
export const DEFAULT_COLLECTION = "default" as CollectionName;

const CONTROL_CHARACTER_PATTERN = /[\u0000-\u001F\u007F]/u;

function normaliseText(
  value: string,
  label: "namespace" | "collection" | "key",
): string {
  const text = value.trim();

  if (text.length === 0) {
    throw new YugenDbError({
      code:
        label === "namespace"
          ? "INVALID_NAMESPACE"
          : label === "collection"
            ? "INVALID_COLLECTION"
            : "INVALID_KEY",
      message: `Invalid ${label}: value must not be empty.`,
    });
  }

  if (CONTROL_CHARACTER_PATTERN.test(text)) {
    throw new YugenDbError({
      code:
        label === "namespace"
          ? "INVALID_NAMESPACE"
          : label === "collection"
            ? "INVALID_COLLECTION"
            : "INVALID_KEY",
      message: `Invalid ${label}: value must not contain control characters.`,
    });
  }

  return text;
}

/** Validates and returns a namespace string. */
export function normaliseNamespace(value: string | Namespace): Namespace {
  return normaliseText(value, "namespace") as Namespace;
}

/** Validates and returns a collection name. */
export function normaliseCollectionName(value: string | CollectionName): CollectionName {
  return normaliseText(value, "collection") as CollectionName;
}

/** Validates and returns a storage key. */
export function normaliseKey(value: string | Key): Key {
  return normaliseText(value, "key") as Key;
}
