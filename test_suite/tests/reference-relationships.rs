#![allow(unused)]

use jsonapi_deserialize::{deserialize_document, Document, JsonApiDeserialize, Reference};

#[derive(Debug, JsonApiDeserialize, Eq, PartialEq, Default)]
struct Article {
    id: String,
    title: String,
    #[json_api(relationship = "single")]
    author: Reference,
    #[json_api(relationship = "optional")]
    reviewer: Option<Reference>,
    #[json_api(relationship = "optional")]
    publisher: Option<Reference>,
    #[json_api(relationship = "multiple")]
    comments: Vec<Reference>,
}

#[test]
fn test_deserialize() {
    let holder = jsonapi_deserialize::Holder::default();
    let document: Document<Article> = deserialize_document(
        r#"{
            "data": {
                "id": "a-1",
                "type": "article",
                "attributes": {
                    "title": "Foo"
                },
                "relationships": {
                    "author": {
                        "data": { "type": "person", "id": "p-1" }
                    },
                    "reviewer": {
                        "data": { "type": "person", "id": "p-2" }
                    },
                    "publisher": {
                        "data": null
                    },
                    "comments": {
                        "data": [
                            { "type": "comment", "id": "c-1" },
                            { "type": "comment", "id": "c-2" }
                        ]
                    }
                }
            }
        }"#,
        &holder
    )
    .unwrap();

    assert_eq!(document.data.id, "a-1".to_string());
    assert_eq!(document.data.title, "Foo".to_string());
    assert_eq!(
        document.data.author,
        Reference {
            kind: "person".to_string(),
            id: "p-1".to_string()
        }
    );
    assert_eq!(
        document.data.reviewer,
        Some(Reference {
            kind: "person".to_string(),
            id: "p-2".to_string()
        })
    );
    assert!(document.data.publisher.is_none());
    assert_eq!(
        document.data.comments.first().cloned().unwrap(),
        Reference {
            kind: "comment".to_string(),
            id: "c-1".to_string()
        }
    );
    assert_eq!(
        document.data.comments.last().cloned().unwrap(),
        Reference {
            kind: "comment".to_string(),
            id: "c-2".to_string()
        }
    );
}
