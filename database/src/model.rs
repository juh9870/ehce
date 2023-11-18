use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::hash::Hash;

use duplicate::duplicate_item;
use itertools::Itertools;
use paste::paste;
use rustc_hash::FxHashMap;
use strum_macros::{Display, EnumDiscriminants, EnumIs};

use utils::slab_map::{SlabMap, SlabMapId, SlabMapKeyOrUntypedId, SlabMapUntypedId};

mod serialization;
pub mod ship;
pub mod ship_build;

#[derive(Debug, serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath)]
#[serde(transparent)]
pub struct DatabaseAsset(pub VersionedDatabaseItem);

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "version")]
pub enum VersionedDatabaseItem {
    #[serde(rename = "0")]
    V0(DatabaseItemSerialized),
}

impl VersionedDatabaseItem {
    pub fn into_serialized(self) -> DatabaseItemSerialized {
        match self {
            VersionedDatabaseItem::V0(item) => item,
        }
    }
}

pub trait DatabaseItemTrait {
    fn id(&self) -> SlabMapUntypedId;
    fn kind(&self) -> DatabaseItemKind;
}

pub trait DatabaseItemSerializedTrait {
    fn id(&self) -> &ItemId;
    fn kind(&self) -> DatabaseItemKind;
}

pub trait DatabaseModelSerializationHelper {
    type Serialized;
}

pub trait ModelKind {
    fn kind() -> DatabaseItemKind;
}

#[duplicate_item(
    ty;
    [ &T ]; [ Option<T> ]; [ Vec<T> ];
)]
impl<T: ModelKind> ModelKind for ty {
    fn kind() -> DatabaseItemKind {
        T::kind()
    }
}

pub type ItemId = String;
pub type ModelStore<T> = SlabMap<ItemId, T>;

fn insert_serialized<T: DatabaseItemSerializedTrait>(
    map: &mut FxHashMap<ItemId, T>,
    item: T,
) -> Result<(), T> {
    match map.entry(item.id().clone()) {
        Entry::Occupied(_) => Err(item),
        Entry::Vacant(entry) => {
            entry.insert(item);
            Ok(())
        }
    }
}

fn convert_raw<T>(raw: ModelStore<Option<T>>) -> ModelStore<T> {
    let mut out: ModelStore<T> = Default::default();
    for (key, id, value) in raw.into_iter().sorted_by_key(|(_, id, _)| *id) {
        let value = value.expect("All registered items should be filled before conversion");
        let (inserted_id, _) = out.insert(key, value);
        assert_eq!(inserted_id.raw(), id, "Should be inserted via the same ID");
    }

    out
}

fn drain_one<T>(items: &mut FxHashMap<ItemId, T>) -> Option<T> {
    if let Some(key) = items.keys().next() {
        if let Some(value) = items.remove(&key.clone()) {
            return Some(value);
        }
    }
    None
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RegistryId {
    kind: DatabaseItemKind,
    id: SlabMapUntypedId,
}

impl RegistryId {
    pub fn kind(&self) -> DatabaseItemKind {
        self.kind
    }
    pub fn id(&self) -> SlabMapUntypedId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct RegistryKeyOrId<T: Borrow<ItemId>> {
    kind: DatabaseItemKind,
    id: SlabMapKeyOrUntypedId<T>,
}

impl<T: Borrow<ItemId>> RegistryKeyOrId<T> {
    pub fn kind(&self) -> DatabaseItemKind {
        self.kind
    }
    pub fn id(&self) -> &SlabMapKeyOrUntypedId<T> {
        &self.id
    }
}

impl RegistryKeyOrId<&ItemId> {
    pub fn cloned(self) -> RegistryKeyOrId<ItemId> {
        RegistryKeyOrId::<ItemId> {
            kind: self.kind,
            id: match self.id {
                SlabMapKeyOrUntypedId::Key(key) => SlabMapKeyOrUntypedId::Key(key.to_string()),
                SlabMapKeyOrUntypedId::Id(id) => SlabMapKeyOrUntypedId::Id(id),
            },
        }
    }
}
macro_rules! registry {
    ($($name:ident: $ty:ty),*$(,)?) => {
        paste! {
            #[derive(Debug, Clone, EnumDiscriminants, EnumIs)]
            #[strum_discriminants(derive(Display, Hash))]
            #[strum_discriminants(name(DatabaseItemKind))]
            pub enum DatabaseItem {
                $(
                    [< $name:camel >]($ty),
                )*
            }

            impl DatabaseItemTrait for DatabaseItem {
                fn id(&self) -> SlabMapUntypedId {
                    match self {
                        $(
                            Self::[<$name:camel>](s) => s.id(),
                        )*
                    }
                }

                fn kind(&self) -> DatabaseItemKind {
                    match self {
                        $(
                            Self::[<$name:camel>](s) => s.kind(),
                        )*
                    }
                }
            }

            impl DatabaseModelSerializationHelper for DatabaseItem {
                type Serialized = DatabaseItemSerialized;
            }

            #[derive(Debug, Clone, EnumIs)]
            pub enum DatabaseItemRef<'a> {
                $(
                    [<$name:camel>](&'a $ty),
                )*
            }

            impl <'a> DatabaseItemTrait for DatabaseItemRef<'a> {
                fn id(&self) -> SlabMapUntypedId {
                    match self {
                        $(
                            Self::[<$name:camel>](s) => s.id(),
                        )*
                    }
                }

                fn kind(&self) -> DatabaseItemKind {
                    match self {
                        $(
                            Self::[<$name:camel>](s) => s.kind(),
                        )*
                    }
                }
            }

            $(
                impl<'a> From<&'a $ty> for DatabaseItemRef<'a> {
                    fn from(value: &'a $ty) -> Self {
                        Self::[<$name:camel>](value)
                    }
                }
            )*

            $(
                impl From<SlabMapId<$ty>> for RegistryKeyOrId<ItemId> {
                    fn from(item: SlabMapId<$ty>) -> RegistryKeyOrId<ItemId> {
                        RegistryKeyOrId {
                            kind: DatabaseItemKind::[<$name:camel>],
                            id: SlabMapKeyOrUntypedId::Id(item.as_untyped()),
                        }
                    }
                }
                impl From<SlabMapId<$ty>> for RegistryId {
                    fn from(item: SlabMapId<$ty>) -> RegistryId {
                        RegistryId {
                            kind: DatabaseItemKind::[<$name:camel>],
                            id: item.as_untyped(),
                        }
                    }
                }
            )*

            #[derive(Debug, Default)]
            pub struct ModRegistry {
                $(
                    pub $name: ModelStore<$ty>,
                )*
            }

            impl ModRegistry {
                pub fn get<T: Borrow<ItemId>>(&self, id: RegistryKeyOrId<T>) -> Option<DatabaseItemRef> {
                    match id.kind {
                        $(
                            DatabaseItemKind::[<$name:camel>] => self.$name.get_by_untyped(id.id).map(DatabaseItemRef::from),
                        )*
                    }
                }
                pub fn get_by_id(&self, id: RegistryId) -> Option<DatabaseItemRef> {
                    match id.kind {
                        $(
                            DatabaseItemKind::[<$name:camel>] => self.$name.get_by_untyped_id(id.id).map(DatabaseItemRef::from),
                        )*
                    }
                }
            }
        }
    };
}

impl DatabaseItem {
    pub fn registry_id(&self) -> RegistryId {
        RegistryId {
            kind: self.kind(),
            id: self.id(),
        }
    }
}

impl<'a> DatabaseItemRef<'a> {
    pub fn registry_id(&self) -> RegistryId {
        RegistryId {
            kind: self.kind(),
            id: self.id(),
        }
    }
}

impl ModRegistry {
    pub fn build(
        items: impl IntoIterator<Item = DatabaseItemSerialized>,
    ) -> Result<Self, serialization::DeserializationError> {
        let mut raws = RawModRegistry::default();
        for item in items.into_iter() {
            if let Err(item) = raws.insert(item) {
                return Err(serialization::DeserializationErrorKind::DuplicateItem(
                    item.id().clone(),
                    item.kind(),
                )
                .into());
            }
        }

        let partial = PartialModRegistry {
            raw: raws,
            ..Default::default()
        };

        partial.deserialize()
    }
}

macro_rules! registry_partial {
    ($($name:ident: $ty:ty),*$(,)?) => {
        paste! {
            #[derive(Debug, Default)]
            pub(crate) struct PartialModRegistry {
                raw: RawModRegistry,
                $(
                    pub $name: ModelStore<Option<$ty>>,
                )*
            }

            impl PartialModRegistry {
                pub fn convert(self) -> ModRegistry {
                    ModRegistry {
                        $(
                            $name: convert_raw(self.$name),
                        )*
                    }
                }
            }
        }
    };
}

macro_rules! registry_raw {
    ($($name:ident: $ty:ty),*$(,)?) => {
        paste! {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, EnumIs)]
            #[serde(tag = "type")]
            #[serde(rename_all = "PascalCase")]
            pub enum DatabaseItemSerialized {
                $(
                    [< $name:camel >](<$ty as DatabaseModelSerializationHelper>::Serialized),
                )*
            }

            impl DatabaseItemSerializedTrait for DatabaseItemSerialized {
                fn id(&self) -> &ItemId {
                    match self {
                        $(
                            Self::[<$name:camel>](s) => s.id(),
                        )*
                    }
                }

                fn kind(&self) -> DatabaseItemKind {
                    match self {
                        $(
                            Self::[<$name:camel>](s) => s.kind(),
                        )*
                    }
                }
            }

            $(
                impl From<<$ty as DatabaseModelSerializationHelper>::Serialized> for DatabaseItemSerialized {
                    fn from(value: <$ty as DatabaseModelSerializationHelper>::Serialized) -> Self {
                        Self::[< $name:camel >](value)
                    }
                }
            )*

            #[derive(Debug, Default)]
            struct RawModRegistry {
                $(
                    pub $name: FxHashMap<ItemId, <$ty as DatabaseModelSerializationHelper>::Serialized>,
                )*
            }

            impl RawModRegistry {
                pub fn insert(&mut self, item: DatabaseItemSerialized) -> Result<(), DatabaseItemSerialized> {
                    match item {
                        $(
                            DatabaseItemSerialized::[<$name:camel>](item) => {
                                insert_serialized(&mut self.$name, item).map_err(|e|e.into())
                            },
                        )*
                    }
                }
            }

            impl PartialModRegistry {
                pub fn deserialize(mut self) -> Result<ModRegistry, serialization::DeserializationError> {
                    $(
                        while let Some(value) = drain_one(&mut self.raw.$name) {
                            serialization::ModelDeserializable::deserialize(value, &mut self)?;
                        }
                    )*
                    Ok(self.convert())
                }
            }
        }
    };
}

#[macro_export]
macro_rules! call_with_all_models {
    ($macro_name:ident) => {
        $macro_name!(ship: $crate::model::ship::Ship, ship_build: $crate::model::ship_build::ShipBuild);
    };
}

pub(crate) use call_with_all_models;

// registry!(ship: ship::Ship, ship_build: ship_build::ShipBuild);
call_with_all_models!(registry_raw);
call_with_all_models!(registry_partial);
call_with_all_models!(registry);
