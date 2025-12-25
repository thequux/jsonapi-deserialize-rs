use std::any::TypeId;
use crate::document::{Document, Holder, RawDocument};
use crate::included::IncludedMap;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid type")]
    InvalidType(&'static str),

    #[error("Document contains neither data nor error")]
    IncompleteDocument,

    #[error("Missing ID")]
    MissingId,

    #[error("Missing resource type")]
    MissingResourceType,

    #[error("Missing attributes")]
    MissingAttributes,

    #[error("Missing relationships")]
    MissingRelationships,

    #[error("Missing field")]
    MissingField(&'static str),

    #[error("Missing resource")]
    MissingResource { kind: String, id: String },

    #[error("Resource type mismatch")]
    ResourceTypeMismatch { expected: String, found: String },

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

pub trait JsonApiDrop {}

pub trait JsonApiDeserialize<'gc>: Sized + JsonApiDrop {
    const TYPE_ID: TypeId = TypeId::of::<Self::ErasedLifetime>();
    type ErasedLifetime: 'static;

    fn from_value(value: &Value, included_map: &mut IncludedMap<'_, 'gc>) -> Result<Self, Error>;

    fn stub() -> Self;
}

impl<T> JsonApiDrop for T {}
impl<'gc, T> JsonApiDeserialize<'gc> for Option<T>
where
    T: JsonApiDeserialize<'gc>,
{
    type ErasedLifetime = Option<T::ErasedLifetime>;
    fn from_value(value: &Value, included_map: &mut IncludedMap<'_, 'gc>) -> Result<Self, Error> {
        if value.is_null() {
            return Ok(None);
        }

        T::from_value(value, included_map).map(Some)
    }

    fn stub() -> Self {
        Some(T::stub())
    }
}

impl<'gc, T> JsonApiDeserialize<'gc> for Vec<T>
where
    T: JsonApiDeserialize<'gc>,
{
    type ErasedLifetime = Vec<T::ErasedLifetime>;
    fn from_value(value: &Value, included_map: &mut IncludedMap<'_, 'gc>) -> Result<Self, Error> {
        value
            .as_array()
            .ok_or(Error::InvalidType("Expected an array"))?
            .iter()
            .map(|value| T::from_value(value, included_map))
            .collect()
    }

    fn stub() -> Self {
        Vec::new()
    }
}

pub fn deserialize_document<'a, 'gc: 'a, T: JsonApiDeserialize<'gc>>(
    json: &'a str,
    bump: &'gc Holder,
) -> Result<Document<'gc, T>, crate::error::Error> {
    let raw_document: RawDocument = serde_json::from_str(json).map_err(Error::SerdeError)?;
    let default_included = Vec::new();
    let included = raw_document.included.as_ref().unwrap_or(&default_included);

    let mut included_map = IncludedMap::from_includes(included, &bump);
    //     match raw_document.included {
    //     Some(ref resources) => IncludedMap::from_includes(resources, &mutation),
    //     None => IncludedMap::empty(&mutation),
    // };

    if let Some(errors) = raw_document.errors {
        return Err(crate::error::Error::DocumentError(errors));
    }

    let data = bump.bump.alloc(T::from_value(
        &raw_document.data.ok_or(Error::IncompleteDocument)?,
        &mut included_map,
    )?);
    bump.to_free.borrow_mut().push(data as *mut T as *mut T::ErasedLifetime as *mut dyn JsonApiDrop);

    drop(included_map);

    Ok(Document {
        data,
        meta: raw_document.meta,
        links: raw_document.links,
    })
}
