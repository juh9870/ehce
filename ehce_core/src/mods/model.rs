use miette::Diagnostic;
use thiserror::Error;
use utils::slab_map::SlabMap;

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

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "PascalCase")]
pub enum DatabaseItem {
    Ship(ship::Ship),
}

impl DatabaseItemTrait for DatabaseItem {
    fn id(&self) -> &ItemId {
        match self {
            DatabaseItem::Ship(s) => s.id(),
        }
    }

    fn deserialize(self, registry: &mut ModRegistry) -> Result<(), ModItemValidationError> {
        match self {
            DatabaseItem::Ship(s) => s.deserialize(registry),
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum ModItemValidationError {}

pub trait DatabaseItemTrait: Sized {
    fn id(&self) -> &ItemId;
    fn deserialize(self, registry: &mut ModRegistry) -> Result<(), ModItemValidationError>;
}
pub type ItemId = String;
pub type ModelStore<T> = SlabMap<String, T>;

#[derive(Debug, Default)]
pub struct ModRegistry {
    pub ships: ModelStore<ship::Ship>,
}
