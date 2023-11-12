use std::borrow::Borrow;
use std::hash::Hash;

use miette::Diagnostic;
use paste::paste;
use strum_macros::{EnumDiscriminants, EnumIs};
use thiserror::Error;

use utils::slab_map::{SlabMap, SlabMapId, SlabMapKeyOrUntypedId, SlabMapUntypedId};

mod ship;

#[derive(Debug, serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath)]
#[serde(transparent)]
pub struct DatabaseAsset(VersionedDatabaseItem);

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "version")]
pub enum VersionedDatabaseItem {
    #[serde(rename = "0")]
    V0(DatabaseItem),
}

impl DatabaseAsset {
    pub fn database_item(&self) -> DatabaseItem {
        match &self.0 {
            VersionedDatabaseItem::V0(item) => item.clone(),
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum ModItemValidationError {}

pub trait DatabaseItemTrait: Sized {
    fn id(&self) -> &ItemId;
    fn deserialize(
        self,
        registry: &mut ModRegistry,
    ) -> Result<(SlabMapUntypedId, Option<DatabaseItem>), ModItemValidationError>;
    fn kind(&self) -> DatabaseItemKind;
}

pub type ItemId = String;
pub type ModelStore<T> = SlabMap<ItemId, T>;

#[derive(Debug, Clone)]
pub struct RegistryId<T: Borrow<ItemId>> {
    kind: DatabaseItemKind,
    id: SlabMapKeyOrUntypedId<T>,
}

impl RegistryId<&ItemId> {
    pub fn cloned(self) -> RegistryId<ItemId> {
        RegistryId::<ItemId> {
            kind: self.kind,
            id: match self.id {
                SlabMapKeyOrUntypedId::Key(key) => SlabMapKeyOrUntypedId::Key(key.to_string()),
                SlabMapKeyOrUntypedId::Id(id) => SlabMapKeyOrUntypedId::Id(id),
            },
        }
    }
}

impl DatabaseItem {
    pub fn registry_id(&self) -> RegistryId<&ItemId> {
        RegistryId {
            kind: self.kind(),
            id: SlabMapKeyOrUntypedId::Key(self.id()),
        }
    }
}

impl<'a> DatabaseItemRef<'a> {
    pub fn registry_id(&self) -> RegistryId<&ItemId> {
        RegistryId {
            kind: self.kind(),
            id: SlabMapKeyOrUntypedId::Key(self.id()),
        }
    }
}

macro_rules! registry {
    ($($name:ident: $ty:ty),*) => {
        paste! {
            #[derive(Debug, Clone, serde::Deserialize, EnumDiscriminants, EnumIs)]
            #[serde(tag = "type")]
            #[serde(rename_all = "PascalCase")]
            #[strum_discriminants(derive(Hash))]
            #[strum_discriminants(name(DatabaseItemKind))]
            pub enum DatabaseItem {
                $(
                    [< $name:camel >]($ty),
                )*
            }

            impl DatabaseItemTrait for DatabaseItem {
                fn id(&self) -> &ItemId {
                    match self {
                        $(
                            Self::[<$name:camel>](s) => s.id(),
                        )*
                    }
                }

                fn deserialize(self, registry: &mut ModRegistry) -> Result<(SlabMapUntypedId, Option<DatabaseItem>), ModItemValidationError> {
                    match self {
                        $(
                            Self::[<$name:camel>](s) => s.deserialize(registry),
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

            #[derive(Debug, Clone, EnumIs)]
            pub enum DatabaseItemRef<'a> {
                $(
                    [<$name:camel>](&'a $ty),
                )*
            }

            impl <'a> DatabaseItemRef<'a> {
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
                impl<'a> From<&'a $ty> for DatabaseItemRef<'a> {
                    fn from(value: &'a $ty) -> Self {
                        Self::[<$name:camel>](value)
                    }
                }
            )*

            $(
                impl From<SlabMapId<$ty>> for RegistryId<ItemId> {
                    fn from(item: SlabMapId<$ty>) -> RegistryId<ItemId> {
                        RegistryId {
                            kind: DatabaseItemKind::[<$name:camel>],
                            id: SlabMapKeyOrUntypedId::Id(item.as_untyped()),
                        }
                    }
                }
            )*

            #[derive(Debug, Default)]
            pub struct ModRegistry {
                $(
                    pub [<$name s>]: ModelStore<$ty>,
                )*
            }

            impl ModRegistry {
                pub fn get_item<'key, 'reg, T: Borrow<ItemId> + 'key>(&self, id: RegistryId<T>) -> Option<DatabaseItemRef> {
                    match id.kind {
                        $(
                            DatabaseItemKind::[<$name:camel>] => self.[<$name s>].get_by_untyped(id.id).map(DatabaseItemRef::from),
                        )*
                    }
                }
            }
        }
    };
}

registry!(ship: ship::Ship);
