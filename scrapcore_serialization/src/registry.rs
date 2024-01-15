use rustc_hash::FxHashMap;
use slabmap::SlabMap;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::path::PathBuf;

use crate::registry::entry::{RegistryEntry, RegistryEntrySerialized};
use crate::registry::kind::{AssetKindProvider, ItemKindProvider};
use crate::serialization::SerializationFallback;
use crate::{AssetName, ItemId};

pub mod entry;
pub mod index;
pub mod inline;
pub mod insert;
pub mod kind;

pub trait RegistryHolder<Value>: SerializationRegistry + ItemKindProvider<Value> {
    fn get_registry(&self) -> &SlabMap<ItemId, RegistryEntry<Value>>;
    fn get_registry_mut(&mut self) -> &mut SlabMap<ItemId, RegistryEntry<Value>>;
}

pub trait SingletonHolder<Value>: SerializationRegistry + ItemKindProvider<Value> {
    fn get_singleton(&self) -> &Value;
    fn get_singleton_mut(&mut self) -> &mut Value;
}

/// Trait for "partial" registries used during deserialization
pub trait PartialRegistryHolder<Value: SerializationFallback>:
    SerializationRegistry + ItemKindProvider<Value>
{
    fn get_registry(&mut self) -> &mut SlabMap<ItemId, Option<RegistryEntry<Value>>>;
    fn get_raw_registry(
        &mut self,
    ) -> &mut FxHashMap<ItemId, (RegistryEntrySerialized<Value::Fallback>, PathBuf)>;
}

/// Trait for "partial" registries used during deserialization
pub trait PartialSingletonHolder<Value: SerializationFallback>:
    SerializationRegistry + ItemKindProvider<Value>
{
    // &mut Option usage is intentional so None can be replaced with Some
    fn get_singleton(&mut self) -> &mut Option<(PathBuf, Value::Fallback)>;
}

pub trait AssetsHolder<Value>: SerializationRegistry + AssetKindProvider<Value> {
    fn get_assets(&self) -> &FxHashMap<AssetName, (Value, PathBuf)>;
    fn get_assets_mut(&mut self) -> &mut FxHashMap<AssetName, (Value, PathBuf)>;
}

pub trait SerializationRegistry: Debug {
    /// Type indicating kind of registry or singleton items
    type ItemKind: Debug + Clone + Display;

    /// Type indicating kind of assets
    type AssetKind: Debug + Clone + Display;

    /// Custom error kind emitted during deserialization
    type Error: Error + Clone;
}
