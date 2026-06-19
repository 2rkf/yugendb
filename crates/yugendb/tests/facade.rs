use yugendb::prelude::*;

#[test]
fn facade_re_exports_core_types() {
    let namespace = Namespace::try_from("app").expect("valid namespace");
    let collection = CollectionName::try_from("users").expect("valid collection");
    let key = Key::try_from("reader").expect("valid key");

    assert_eq!(namespace.as_ref(), "app");
    assert_eq!(collection.as_ref(), "users");
    assert_eq!(key.as_ref(), "reader");
}

#[test]
fn prelude_exposes_error_code_and_codec() {
    let codec = JsonCodec;
    assert_eq!(codec.name(), "json");
    assert_eq!(
        ErrorCode::SerialisationError.as_str(),
        "SERIALISATION_ERROR"
    );
}
