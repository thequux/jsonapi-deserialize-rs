use std::cell::RefCell;
use crate::deserialize::{JsonApiDeserialize, JsonApiDrop};
use crate::link::Link;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Default)]
pub struct Holder{
    pub(crate) bump: bumpalo::Bump,
    pub(crate) to_free: RefCell<Vec<*mut dyn JsonApiDrop>>
}

impl Drop for Holder {
    fn drop(&mut self) {
        for ptr in self.to_free.borrow_mut().drain(..) {
            unsafe { std::ptr::drop_in_place(ptr) };
        }
    }
}

#[derive(Debug)]
pub struct Document<'a, T: 'a>
where
    T: JsonApiDeserialize<'a>,
{
    pub data: &'a T,
    pub meta: Option<HashMap<String, Value>>,
    pub links: Option<DocumentLinks>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentError {
    pub id: Option<String>,
    pub links: Option<ErrorLinks>,
    pub status: Option<String>,
    pub code: Option<String>,
    pub title: Option<String>,
    pub detail: Option<String>,
    pub source: Option<ErrorSource>,
    pub meta: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorLinks {
    pub about: Option<Link>,
    #[serde(rename = "type")]
    pub kind: Option<Link>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorSource {
    pub pointer: Option<String>,
    pub parameter: Option<String>,
    pub header: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DocumentLinks {
    #[serde(rename = "self")]
    pub this: Option<Link>,
    pub related: Option<Link>,
    #[serde(rename = "describedby")]
    pub described_by: Option<Link>,
    pub first: Option<Link>,
    pub last: Option<Link>,
    pub prev: Option<Link>,
    pub next: Option<Link>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone, Default)]
pub struct Reference {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawDocument {
    pub data: Option<Value>,
    pub errors: Option<Vec<DocumentError>>,
    pub meta: Option<HashMap<String, Value>>,
    pub links: Option<DocumentLinks>,
    pub included: Option<Vec<RawResource>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawResource {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub attributes: Option<Value>,
    pub relationships: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct RawSingleRelationship {
    pub data: Reference,
}

#[derive(Debug, Deserialize)]
pub struct RawOptionalRelationship {
    pub data: Option<Reference>,
}

#[derive(Debug, Deserialize)]
pub struct RawMultipleRelationship {
    pub data: Vec<Reference>,
}

impl<'a> From<&'a RawResource> for Value {
    fn from(resource: &'a RawResource) -> Self {
        let mut value = serde_json::json!({
            "type": resource.kind,
            "id": resource.id,
        });

        if let Some(attributes) = &resource.attributes {
            let attrs_value = serde_json::json!(attributes);
            value["attributes"] = attrs_value;
        }

        if let Some(relationships) = &resource.relationships {
            let rels_value = serde_json::json!(relationships);
            value["relationships"] = rels_value;
        }

        value
    }
}
