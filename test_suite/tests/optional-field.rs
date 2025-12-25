#![allow(unused)]

use jsonapi_deserialize::{deserialize_document, Document, JsonApiDeserialize, Reference};
use zonbi::Zonbi;

#[derive(Debug, JsonApiDeserialize, Zonbi, Default)]
struct Resource<'gc>  {
    id: String,
    #[json_api(default)]
    default_string: String,
    #[json_api(default)]
    default_option: Option<String>,
    #[json_api(optional)]
    optional: Option<String>,
    #[json_api(optional)]
    optional_nullable: Option<Option<String>>,
    #[json_api(default, relationship = "single")]
    default_ref: Option<Reference>,
    #[json_api(optional, relationship = "optional")]
    optional_ref: Option<Option<Reference>>,

    #[json_api(optional, relationship = "optional", resource = "Resource")]
    optional_self_ref: Option<Option<&'gc Resource<'gc>>>,

}

#[test]
fn test_optional_missing_attribute() {
    let holder = jsonapi_deserialize::Holder::default();
    let document: Document<Resource> = deserialize_document(
        r#"{
                "data": {
                    "id": "1",
                    "type": "resource"
                }
            }"#,
        &holder
    )
        .unwrap();

    assert_eq!(document.data.default_string.as_str(), "");
    assert_eq!(document.data.default_option, None);
    assert_eq!(document.data.optional, None);
    assert_eq!(document.data.optional_nullable, None);
}

#[test]
fn test_optional_set_fields() {
    let holder = jsonapi_deserialize::Holder::default();

    let document: Document<Resource> = deserialize_document(
        r#"{
            "data": {
                "id": "1",
                "type": "resource",
                "attributes": {
                    "defaultString": "foo",
                    "defaultOption": "foo",
                    "optional": "bar",
                    "optionalNullable": "baz"
                }
            }
        }"#,
        &holder
    )
    .unwrap();

    assert_eq!(document.data.default_string, "foo");
    assert_eq!(document.data.default_option, Some("foo".to_string()));
    assert_eq!(document.data.optional, Some("bar".to_string()));
    assert_eq!(
        document.data.optional_nullable,
        Some(Some("baz".to_string()))
    );
}

#[test]
fn test_optional_null_field() {
    let holder = jsonapi_deserialize::Holder::default();

    let document: Document<Resource> = deserialize_document(
        r#"{
            "data": {
                "id": "1",
                "type": "resource",
                "attributes": {
                    "optionalNullable": null
                }
            }
        }"#, &holder
    )
    .unwrap();

    assert_eq!(document.data.optional_nullable, Some(None));
}

#[test]
fn test_optional_missing_relationships() {
    let holder = jsonapi_deserialize::Holder::default();
    let document: Document<Resource> = deserialize_document(
        r#"{
            "data": {
                "id": "1",
                "type": "resource"
            }
        }"#, &holder
    )
    .unwrap();

    assert_eq!(document.data.default_ref, None);
    assert_eq!(document.data.optional_ref, None);
}
