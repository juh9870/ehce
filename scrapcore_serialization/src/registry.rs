use rustc_hash::FxHashMap;
use slabmap::SlabMap;
use std::error::Error;
use std::fmt::{Debug, Display};

use crate::registry::entry::{RegistryEntry, RegistryEntrySerialized};
use crate::serialization::SerializationFallback;
use crate::{AssetName, ItemId};

pub mod entry;
pub mod index;
pub mod inline;

pub trait RegistryHolder<Value>: SerializationRegistry {
    fn get_registry(&self) -> &SlabMap<ItemId, RegistryEntry<Value>>;
    fn get_registry_mut(&mut self) -> &mut SlabMap<ItemId, RegistryEntry<Value>>;
}

pub trait SingletonHolder<Value>: SerializationRegistry {
    fn get_singleton(&self) -> &Value;
    fn get_singleton_mut(&mut self) -> &mut Value;
}

/// Trait for "partial" registries used during deserialization
pub trait PartialRegistryHolder<Value: SerializationFallback>: SerializationRegistry {
    fn get_registry(&mut self) -> &mut SlabMap<ItemId, Option<RegistryEntry<Value>>>;
    fn get_raw_registry(
        &mut self,
    ) -> &mut FxHashMap<ItemId, RegistryEntrySerialized<Value::Fallback>>;
}

pub trait AssetsHolder<Value>: SerializationRegistry {
    fn get_assets(&self) -> &FxHashMap<AssetName, Value>;
    fn get_assets_mut(&mut self) -> &mut FxHashMap<AssetName, Value>;
}

pub trait ItemKindProvider<Item>: SerializationRegistry {
    fn kind() -> Self::ItemKind;
}

impl<Registry: ItemKindProvider<T>, T> ItemKindProvider<Option<T>> for Registry {
    fn kind() -> Self::ItemKind {
        <Registry as ItemKindProvider<T>>::kind()
    }
}

impl<Registry: ItemKindProvider<T>, T> ItemKindProvider<RegistryEntry<T>> for Registry {
    fn kind() -> Self::ItemKind {
        <Registry as ItemKindProvider<T>>::kind()
    }
}

pub trait AssetKindProvider<Asset>: SerializationRegistry {
    fn asset_kind() -> Self::AssetKind;
}

pub trait SerializationRegistry: Debug {
    type ItemKind: Debug + Clone + Display;
    type AssetKind: Debug + Clone + Display;
    type Error: Error + Clone;
}
