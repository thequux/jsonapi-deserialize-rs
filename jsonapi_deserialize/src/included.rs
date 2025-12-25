use crate::deserialize::{Error, JsonApiDeserialize, JsonApiDrop};
use crate::document::{Holder, RawResource};
use serde_json::Value;
use std::any::{type_name, TypeId};
use std::collections::HashMap;

pub struct IncludedMap<'a, 'gc: 'a> {
    holder: &'gc Holder,
    raw_map: HashMap<(&'a str, &'a str), &'a RawResource>,
    deserialized_map: HashMap<(&'a str, &'a str, TypeId), (TypeId, &'static str, *mut ())>,
}

impl<'doc, 'gc: 'doc> IncludedMap<'doc, 'gc> {
    pub fn get<T: 'gc>(&mut self, kind: &str, id: &str) -> Result<&'gc T, Error>
    where
        T: JsonApiDeserialize<'gc>,
    {

        if let Some((zid, type_name, ptr)) = self.deserialized_map.get(&(kind, id, T::TYPE_ID)).cloned() {
            if zid == T::TYPE_ID {
                // SAFETY: In theory, this could be used to expand the lifetime of T, but we'll be careful™
                return Ok(unsafe { &*(ptr as *const T) });
            } else {
                return Err(Error::ResourceTypeMismatch {
                    expected: std::any::type_name::<T>().to_owned(),
                    found: type_name.to_owned(),
                })
            }
        }

        let (kind, id, value) = {
            let raw_resource =
                self.raw_map
                    .get(&(kind, id))
                    .ok_or_else(|| Error::MissingResource {
                        kind: kind.to_string(),
                        id: id.to_string(),
                    })?;
            let kind = raw_resource.kind.as_str();
            let id = raw_resource.id.as_str();
            let value: Value = (*raw_resource).into();

            (kind, id, value)
        };
        // Put a stub in place, to handle cycles
        let new_item = self.holder.bump.alloc_with(T::stub);
        self.holder.to_free.borrow_mut().push(new_item as *mut T as *mut T::ErasedLifetime as *mut dyn JsonApiDrop);
        self.deserialized_map.insert((kind, id, T::TYPE_ID), (T::TYPE_ID, type_name::<T>(), new_item as *mut T as *mut ()));
        *new_item = T::from_value(&value, self)?;
        Ok(new_item as &'gc T)
    }
    pub fn empty(holder: &'gc Holder) -> Self{
        Self {
            holder,
            raw_map: HashMap::new(),
            deserialized_map: HashMap::new(),
        }
    }
}



impl<'a, 'gc> IncludedMap<'a, 'gc> {
    pub(crate) fn from_includes(resources: &'a Vec<RawResource>, holder: &'gc Holder) -> Self {
        let raw_map = resources
            .iter()
            .map(|raw| ((raw.kind.as_str(), raw.id.as_str()), raw))
            .collect();

        Self {
            holder,
            raw_map,
            deserialized_map: HashMap::new(),
        }
    }
}
