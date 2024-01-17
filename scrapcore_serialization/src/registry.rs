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

pub type ItemCollection<T> = SlabMap<ItemId, RegistryEntry<T>>;
pub type PartialItemCollection<T> = SlabMap<ItemId, Option<RegistryEntry<T>>>;
pub type RawItemCollection<T> = FxHashMap<
    ItemId,
    (
        RegistryEntrySerialized<<T as SerializationFallback>::Fallback>,
        PathBuf,
    ),
>;

pub type Singleton<T> = T;
pub type PartialSingleton<T> = Option<(PathBuf, <T as SerializationFallback>::Fallback)>;

pub type AssetsCollection<T> = FxHashMap<AssetName, (T, PathBuf)>;

pub trait CollectionHolder<Value>: SerializationRegistry + ItemKindProvider<Value> {
    fn get_collection(&self) -> &ItemCollection<Value>;
    fn get_collection_mut(&mut self) -> &mut ItemCollection<Value>;
}

pub trait SingletonHolder<Value>: SerializationRegistry + ItemKindProvider<Value> {
    fn get_singleton(&self) -> &Singleton<Value>;
    fn get_singleton_mut(&mut self) -> &mut Singleton<Value>;
}

/// Trait for "partial" registries used during deserialization
pub trait PartialCollectionHolder<Value: SerializationFallback>:
    SerializationRegistry + ItemKindProvider<Value>
{
    fn get_collection(&mut self) -> &mut PartialItemCollection<Value>;
    fn get_raw_collection(&mut self) -> &mut RawItemCollection<Value>;
}

/// Trait for "partial" registries used during deserialization
pub trait PartialSingletonHolder<Value: SerializationFallback>:
    SerializationRegistry + ItemKindProvider<Value>
{
    // &mut Option usage is intentional so None can be replaced with Some
    fn get_singleton(&mut self) -> &mut PartialSingleton<Value>;
}

pub trait AssetsHolder<Value>: SerializationRegistry + AssetKindProvider<Value> {
    fn get_assets(&self) -> &AssetsCollection<Value>;
    fn get_assets_mut(&mut self) -> &mut AssetsCollection<Value>;
}

pub trait SerializationRegistry: Debug {
    /// Type indicating kind of registry or singleton items
    type ItemKind: Debug + Clone + Display;

    /// Type indicating kind of assets
    type AssetKind: Debug + Clone + Display;

    /// Custom error kind emitted during deserialization
    type Error: Error + Clone;
}
