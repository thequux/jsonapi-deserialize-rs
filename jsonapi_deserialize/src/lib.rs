mod deserialize;
mod document;
mod error;
mod included;
mod link;

pub use deserialize::{deserialize_document, Error as DeserializeError, JsonApiDeserialize};
pub use document::{
    Document, DocumentError, DocumentLinks, ErrorLinks, ErrorSource, RawMultipleRelationship,
    RawOptionalRelationship, RawSingleRelationship, Reference,
};
pub use error::Error;
pub use included::IncludedMap;
pub use link::Link;

extern crate jsonapi_deserialize_derive;
pub use jsonapi_deserialize_derive::JsonApiDeserialize;

#[doc(hidden)]
pub extern crate zonbi;
#[doc(hidden)]
pub extern crate bumpalo;

pub use document::Holder;