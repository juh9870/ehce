use crate::registry::entry::{RegistryEntry, RegistryEntrySerialized};
use crate::registry::{PartialCollectionHolder, SerializationRegistry};
use crate::reservation::{get_reserved_key, insert_reserved, reserve};
use crate::serialization::error::{
    DeserializationError, DeserializationErrorKind, DeserializationErrorStackItem,
};
use crate::{ItemId, ItemIdRef};
use rustc_hash::FxHashMap;
use slabmap::SlabMapId;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::{BuildHasher, Hash};

pub mod box_wrapper;
pub mod error;

pub mod min_max;

pub mod helpers;

#[cfg(feature = "bevy")]
pub mod bevy;

pub trait SerializationFallback {
    type Fallback;
}

pub trait DeserializeModel<T, Registry: SerializationRegistry> {
    fn deserialize(self, registry: &mut Registry) -> Result<T, DeserializationError<Registry>>;
}

impl<Registry: SerializationRegistry, T: DeserializeModel<R, Registry>, R>
    DeserializeModel<Option<R>, Registry> for Option<T>
{
    fn deserialize(
        self,
        registry: &mut Registry,
    ) -> Result<Option<R>, DeserializationError<Registry>> {
        self.map(|e| e.deserialize(registry)).transpose()
    }
}

impl<T: SerializationFallback> SerializationFallback for Option<T> {
    type Fallback = Option<T::Fallback>;
}

// region Vec

impl<Registry: SerializationRegistry, T: DeserializeModel<R, Registry>, R>
    DeserializeModel<Vec<R>, Registry> for Vec<T>
{
    #[inline]
    fn deserialize(
        self,
        registry: &mut Registry,
    ) -> Result<Vec<R>, DeserializationError<Registry>> {
        self.into_iter()
            .enumerate()
            .map(|(i, e)| {
                e.deserialize(registry)
                    .map_err(|e| e.context(DeserializationErrorStackItem::Index(i)))
            })
            .collect()
    }
}

impl<T: SerializationFallback> SerializationFallback for Vec<T> {
    type Fallback = Vec<T::Fallback>;
}

// endregion

// region HashMap

impl<
        Registry: SerializationRegistry,
        RawKey: DeserializeModel<Key, Registry> + Eq + Hash + Display,
        Key: Eq + Hash,
        RawValue: DeserializeModel<Value, Registry>,
        Value,
        RawHasher: BuildHasher,
        Hasher: BuildHasher + Default,
    > DeserializeModel<HashMap<Key, Value, Hasher>, Registry>
    for HashMap<RawKey, RawValue, RawHasher>
{
    fn deserialize(
        self,
        registry: &mut Registry,
    ) -> Result<HashMap<Key, Value, Hasher>, DeserializationError<Registry>> {
        self.into_iter()
            .map(|(k, v)| {
                let key_str = k.to_string();
                let v = v.deserialize(registry).map_err(|e| {
                    e.context(DeserializationErrorStackItem::MapEntry(key_str.clone()))
                })?;
                let k = k
                    .deserialize(registry)
                    .map_err(|e| e.context(DeserializationErrorStackItem::MapKey(key_str)))?;
                Ok((k, v))
            })
            .collect()
    }
}

impl<Key: SerializationFallback, Value: SerializationFallback, Hasher: BuildHasher>
    SerializationFallback for HashMap<Key, Value, Hasher>
{
    type Fallback = FxHashMap<Key::Fallback, Value::Fallback>;
}

// endregion

impl<Item> SerializationFallback for SlabMapId<Item> {
    type Fallback = ItemId;
}

impl<Registry: SerializationRegistry, T> DeserializeModel<T, Registry> for String
where
    for<'a> &'a str: DeserializeModel<T, Registry>,
{
    fn deserialize(self, registry: &mut Registry) -> Result<T, DeserializationError<Registry>> {
        self.as_str().deserialize(registry)
    }
}

impl<
        'a,
        Registry: SerializationRegistry + PartialCollectionHolder<Data>,
        Data: SerializationFallback,
    > DeserializeModel<SlabMapId<RegistryEntry<Data>>, Registry> for ItemIdRef<'a>
where
    Data::Fallback: DeserializeModel<Data, Registry>,
{
    fn deserialize(
        self,
        registry: &mut Registry,
    ) -> Result<SlabMapId<RegistryEntry<Data>>, DeserializationError<Registry>> {
        let items = registry.get_collection();
        if let Some(id) = get_reserved_key(items, self) {
            return Ok(id);
        }
        let Some(other) = registry.get_raw_collection().remove(self) else {
            return Err(DeserializationErrorKind::<Registry>::MissingItem(
                self.to_string(),
                Registry::kind(),
            )
            .into());
        };
        other.0.deserialize(registry)
    }
}

impl<
        Registry: SerializationRegistry + PartialCollectionHolder<Data>,
        Data: SerializationFallback,
    > DeserializeModel<SlabMapId<RegistryEntry<Data>>, Registry>
    for RegistryEntrySerialized<Data::Fallback>
where
    Data::Fallback: DeserializeModel<Data, Registry>,
{
    fn deserialize(
        self,
        registry: &mut Registry,
    ) -> Result<SlabMapId<RegistryEntry<Data>>, DeserializationError<Registry>> {
        let items = registry.get_collection();
        let reserved = reserve(items, self.id.clone())?;
        let data =
            DeserializeModel::<Data, Registry>::deserialize(self.data, registry).map_err(|e| {
                e.context(DeserializationErrorStackItem::ItemById(
                    self.id,
                    Registry::kind(),
                ))
            })?;
        let id = reserved.raw();
        let model = RegistryEntry { id, data };

        let items = registry.get_collection();
        let id = insert_reserved(items, reserved, model);

        Ok(id)
    }
}
