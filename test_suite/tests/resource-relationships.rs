#![allow(unused)]

use jsonapi_deserialize::{deserialize_document, Document, Holder, JsonApiDeserialize};
use std::sync::Arc;

#[derive(Debug, JsonApiDeserialize, Default)]
struct Article<'a> {
    id: String,
    title: String,
    #[json_api(relationship = "single", resource = "Person")]
    author: &'a Person,
    #[json_api(relationship = "optional", resource = "Person")]
    reviewer: Option<&'a Person>,
    #[json_api(relationship = "optional", resource = "Person")]
    publisher: Option<&'a Person>,
    #[json_api(relationship = "multiple", resource = "Comment")]
    comments: Vec<&'a Comment<'a>>,
}

#[derive(Debug, JsonApiDeserialize, Default)]
struct Person {
    name: String,
}

#[derive(Debug, JsonApiDeserialize, Default)]
struct Comment<'a> {
    #[json_api(relationship = "optional", resource = "Person")]
    author: Option<&'a Person>,
    content: String,
}

#[test]
fn test_deserialize() {
    let holder = Holder::default();
    let document: Document<Article> = deserialize_document(
        r#"{
            "data": {
                "id": "123",
                "type": "article",
                "attributes": {
                    "title": "Foo"
                },
                "relationships": {
                    "author": {
                        "data": { "type": "person", "id": "p-1" }
                    },
                    "reviewer": {
                        "data": { "type": "person", "id": "p-1" }
                    },
                    "publisher": {
                        "data": null
                    },
                    "comments": {
                        "data": [
                            { "type": "comment", "id": "c-1" }
                        ]
                    }
                }
            },
            "included": [
                {
                    "type": "person",
                    "id": "p-1",
                    "attributes": {
                        "name": "John Smith"
                    }
                },
                {
                    "type": "comment",
                    "id": "c-1",
                    "attributes": {
                        "content": "Lorem Ipsum"
                    },
                    "relationships": {
                        "author": {
                            "data": { "type": "person", "id": "p-1" }
                        }
                    }
                }
            ]
        }"#,
        &holder
    )
    .unwrap();

    assert_eq!(document.data.title, "Foo".to_string());
    // assert_eq!(document.data.author.name, "John Smith");
    assert_eq!(document.data.reviewer.as_ref().unwrap().name, "John Smith");
    assert!(document.data.publisher.is_none());

    let comment = document.data.comments.first().unwrap();
    assert_eq!(comment.content, "Lorem Ipsum".to_string());
    println!("{:#?}", comment);
    assert_eq!(comment.author.as_ref().unwrap().name, "John Smith");
}
