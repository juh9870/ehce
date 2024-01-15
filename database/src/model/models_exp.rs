use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::ops::Index;
use std::path::{Path, PathBuf};

use bevy::asset::Handle;
use bevy::prelude::Image;
use duplicate::duplicate_item;
use itertools::Itertools;
use rustc_hash::FxHashMap;
use schemars::schema::RootSchema;
use strum_macros::{Display, EnumDiscriminants, EnumIs};

use slabmap::{SlabMap, SlabMapId, SlabMapKeyOrUntypedId, SlabMapUntypedId};

use crate::model::serialization;
use crate::model::serialization::RegistryEntry;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, EnumIs)]
#[serde(tag = "type")]
#[serde(rename_all = "PascalCase")]
#[serde(rename = "DatabaseItem")]
pub enum DatabaseItemSerialized {
    Ship(<RegistryEntry<crate::model::ship::Ship> as serialization::ModelDeserializableFallbackType>::Serialized),
    ShipBuild(<RegistryEntry<crate::model::ship_build::ShipBuild> as serialization::ModelDeserializableFallbackType>::Serialized),
    ComponentStats(<RegistryEntry<crate::model::component_stats::ComponentStats> as serialization::ModelDeserializableFallbackType>::Serialized),
    Variable(<RegistryEntry<crate::model::variable::Variable> as serialization::ModelDeserializableFallbackType>::Serialized),
    Component(<RegistryEntry<crate::model::component::Component> as serialization::ModelDeserializableFallbackType>::Serialized),
    Fleet(<RegistryEntry<crate::model::fleet::Fleet> as serialization::ModelDeserializableFallbackType>::Serialized),
    CombatSettings(<RegistryEntry<crate::model::combat_settings::CombatSettings> as serialization::ModelDeserializableFallbackType>::Serialized),
    Device(<RegistryEntry<crate::model::device::Device> as serialization::ModelDeserializableFallbackType>::Serialized),
}

impl DatabaseItemSerializedTrait for DatabaseItemSerialized {
    fn id(&self) -> &ItemId {
        match self {
            Self::Ship(s) => s.id(),
            Self::ShipBuild(s) => s.id(),
            Self::ComponentStats(s) => s.id(),
            Self::Variable(s) => s.id(),
            Self::Component(s) => s.id(),
            Self::Fleet(s) => s.id(),
            Self::CombatSettings(s) => s.id(),
            Self::Device(s) => s.id(),
        }
    }

    fn kind(&self) -> DatabaseItemKind {
        match self {
            Self::Ship(s) => s.kind(),
            Self::ShipBuild(s) => s.kind(),
            Self::ComponentStats(s) => s.kind(),
            Self::Variable(s) => s.kind(),
            Self::Component(s) => s.kind(),
            Self::Fleet(s) => s.kind(),
            Self::CombatSettings(s) => s.kind(),
            Self::Device(s) => s.kind(),
        }
    }
}

impl From<<RegistryEntry<crate::model::ship::Ship> as serialization::ModelDeserializableFallbackType>::Serialized> for DatabaseItemSerialized {
    fn from(value: <RegistryEntry<crate::model::ship::Ship> as serialization::ModelDeserializableFallbackType>::Serialized) -> Self {
        Self::Ship(value)
    }
}

impl From<<RegistryEntry<crate::model::ship_build::ShipBuild> as serialization::ModelDeserializableFallbackType>::Serialized> for DatabaseItemSerialized {
    fn from(value: <RegistryEntry<crate::model::ship_build::ShipBuild> as serialization::ModelDeserializableFallbackType>::Serialized) -> Self {
        Self::ShipBuild(value)
    }
}

impl From<<RegistryEntry<crate::model::component_stats::ComponentStats> as serialization::ModelDeserializableFallbackType>::Serialized> for DatabaseItemSerialized {
    fn from(value: <RegistryEntry<crate::model::component_stats::ComponentStats> as serialization::ModelDeserializableFallbackType>::Serialized) -> Self {
        Self::ComponentStats(value)
    }
}

impl From<<RegistryEntry<crate::model::variable::Variable> as serialization::ModelDeserializableFallbackType>::Serialized> for DatabaseItemSerialized {
    fn from(value: <RegistryEntry<crate::model::variable::Variable> as serialization::ModelDeserializableFallbackType>::Serialized) -> Self {
        Self::Variable(value)
    }
}

impl From<<RegistryEntry<crate::model::component::Component> as serialization::ModelDeserializableFallbackType>::Serialized> for DatabaseItemSerialized {
    fn from(value: <RegistryEntry<crate::model::component::Component> as serialization::ModelDeserializableFallbackType>::Serialized) -> Self {
        Self::Component(value)
    }
}

impl From<<RegistryEntry<crate::model::fleet::Fleet> as serialization::ModelDeserializableFallbackType>::Serialized> for DatabaseItemSerialized {
    fn from(value: <RegistryEntry<crate::model::fleet::Fleet> as serialization::ModelDeserializableFallbackType>::Serialized) -> Self {
        Self::Fleet(value)
    }
}

impl From<<RegistryEntry<crate::model::combat_settings::CombatSettings> as serialization::ModelDeserializableFallbackType>::Serialized> for DatabaseItemSerialized {
    fn from(value: <RegistryEntry<crate::model::combat_settings::CombatSettings> as serialization::ModelDeserializableFallbackType>::Serialized) -> Self {
        Self::CombatSettings(value)
    }
}

impl From<<RegistryEntry<crate::model::device::Device> as serialization::ModelDeserializableFallbackType>::Serialized> for DatabaseItemSerialized {
    fn from(value: <RegistryEntry<crate::model::device::Device> as serialization::ModelDeserializableFallbackType>::Serialized) -> Self {
        Self::Device(value)
    }
}

#[derive(Debug, Default)]
struct RawModRegistry {
    pub ship: FxHashMap<ItemId, <RegistryEntry<crate::model::ship::Ship> as serialization::ModelDeserializableFallbackType>::Serialized>,
    pub ship_build: FxHashMap<ItemId, <RegistryEntry<crate::model::ship_build::ShipBuild> as serialization::ModelDeserializableFallbackType>::Serialized>,
    pub component_stats: FxHashMap<ItemId, <RegistryEntry<crate::model::component_stats::ComponentStats> as serialization::ModelDeserializableFallbackType>::Serialized>,
    pub variable: FxHashMap<ItemId, <RegistryEntry<crate::model::variable::Variable> as serialization::ModelDeserializableFallbackType>::Serialized>,
    pub component: FxHashMap<ItemId, <RegistryEntry<crate::model::component::Component> as serialization::ModelDeserializableFallbackType>::Serialized>,
    pub fleet: FxHashMap<ItemId, <RegistryEntry<crate::model::fleet::Fleet> as serialization::ModelDeserializableFallbackType>::Serialized>,
    pub combat_settings: FxHashMap<ItemId, <RegistryEntry<crate::model::combat_settings::CombatSettings> as serialization::ModelDeserializableFallbackType>::Serialized>,
    pub device: FxHashMap<ItemId, <RegistryEntry<crate::model::device::Device> as serialization::ModelDeserializableFallbackType>::Serialized>,
}

impl RawModRegistry {
    pub fn insert(&mut self, item: DatabaseItemSerialized) -> Result<(), DatabaseItemSerialized> {
        match item {
            DatabaseItemSerialized::Ship(item) => {
                insert_serialized(&mut self.ship, item).map_err(|e| e.into())
            }
            DatabaseItemSerialized::ShipBuild(item) => {
                insert_serialized(&mut self.ship_build, item).map_err(|e| e.into())
            }
            DatabaseItemSerialized::ComponentStats(item) => {
                insert_serialized(&mut self.component_stats, item).map_err(|e| e.into())
            }
            DatabaseItemSerialized::Variable(item) => {
                insert_serialized(&mut self.variable, item).map_err(|e| e.into())
            }
            DatabaseItemSerialized::Component(item) => {
                insert_serialized(&mut self.component, item).map_err(|e| e.into())
            }
            DatabaseItemSerialized::Fleet(item) => {
                insert_serialized(&mut self.fleet, item).map_err(|e| e.into())
            }
            DatabaseItemSerialized::CombatSettings(item) => {
                insert_serialized(&mut self.combat_settings, item).map_err(|e| e.into())
            }
            DatabaseItemSerialized::Device(item) => {
                insert_serialized(&mut self.device, item).map_err(|e| e.into())
            }
        }
    }
}

impl PartialModRegistry {
    pub fn deserialize(mut self) -> Result<ModRegistry, serialization::DeserializationError> {
        while let Some(value) = drain_one(&mut self.raw.ship) {
            serialization::ModelDeserializable::deserialize(value, &mut self)?;
        }
        while let Some(value) = drain_one(&mut self.raw.ship_build) {
            serialization::ModelDeserializable::deserialize(value, &mut self)?;
        }
        while let Some(value) = drain_one(&mut self.raw.component_stats) {
            serialization::ModelDeserializable::deserialize(value, &mut self)?;
        }
        while let Some(value) = drain_one(&mut self.raw.variable) {
            serialization::ModelDeserializable::deserialize(value, &mut self)?;
        }
        while let Some(value) = drain_one(&mut self.raw.component) {
            serialization::ModelDeserializable::deserialize(value, &mut self)?;
        }
        while let Some(value) = drain_one(&mut self.raw.fleet) {
            serialization::ModelDeserializable::deserialize(value, &mut self)?;
        }
        while let Some(value) = drain_one(&mut self.raw.combat_settings) {
            serialization::ModelDeserializable::deserialize(value, &mut self)?;
        }
        while let Some(value) = drain_one(&mut self.raw.device) {
            serialization::ModelDeserializable::deserialize(value, &mut self)?;
        }
        Ok(self.convert())
    }
}

#[derive(Debug, Default)]
pub(crate) struct PartialModRegistry {
    raw: RawModRegistry,
    assets: ModAssets,
    pub ship: ModelStore<Option<RegistryEntry<crate::model::ship::Ship>>>,
    pub ship_build: ModelStore<Option<RegistryEntry<crate::model::ship_build::ShipBuild>>>,
    pub component_stats:
        ModelStore<Option<RegistryEntry<crate::model::component_stats::ComponentStats>>>,
    pub variable: ModelStore<Option<RegistryEntry<crate::model::variable::Variable>>>,
    pub component: ModelStore<Option<RegistryEntry<crate::model::component::Component>>>,
    pub fleet: ModelStore<Option<RegistryEntry<crate::model::fleet::Fleet>>>,
    pub combat_settings:
        ModelStore<Option<RegistryEntry<crate::model::combat_settings::CombatSettings>>>,
    pub device: ModelStore<Option<RegistryEntry<crate::model::device::Device>>>,
}

impl PartialModRegistry {
    pub fn convert(self) -> ModRegistry {
        ModRegistry {
            ship: convert_raw(self.ship),
            ship_build: convert_raw(self.ship_build),
            component_stats: convert_raw(self.component_stats),
            variable: convert_raw(self.variable),
            component: convert_raw(self.component),
            fleet: convert_raw(self.fleet),
            combat_settings: convert_raw(self.combat_settings),
            device: convert_raw(self.device),
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants, EnumIs)]
#[strum_discriminants(derive(Display, Hash))]
#[strum_discriminants(name(DatabaseItemKind))]
pub enum DatabaseItem {
    Ship(RegistryEntry<crate::model::ship::Ship>),
    ShipBuild(RegistryEntry<crate::model::ship_build::ShipBuild>),
    ComponentStats(RegistryEntry<crate::model::component_stats::ComponentStats>),
    Variable(RegistryEntry<crate::model::variable::Variable>),
    Component(RegistryEntry<crate::model::component::Component>),
    Fleet(RegistryEntry<crate::model::fleet::Fleet>),
    CombatSettings(RegistryEntry<crate::model::combat_settings::CombatSettings>),
    Device(RegistryEntry<crate::model::device::Device>),
}

impl DatabaseItemTrait for DatabaseItem {
    fn id(&self) -> SlabMapUntypedId {
        match self {
            Self::Ship(s) => s.id(),
            Self::ShipBuild(s) => s.id(),
            Self::ComponentStats(s) => s.id(),
            Self::Variable(s) => s.id(),
            Self::Component(s) => s.id(),
            Self::Fleet(s) => s.id(),
            Self::CombatSettings(s) => s.id(),
            Self::Device(s) => s.id(),
        }
    }

    fn kind(&self) -> DatabaseItemKind {
        match self {
            Self::Ship(s) => s.kind(),
            Self::ShipBuild(s) => s.kind(),
            Self::ComponentStats(s) => s.kind(),
            Self::Variable(s) => s.kind(),
            Self::Component(s) => s.kind(),
            Self::Fleet(s) => s.kind(),
            Self::CombatSettings(s) => s.kind(),
            Self::Device(s) => s.kind(),
        }
    }
}

impl serialization::ModelDeserializableFallbackType for DatabaseItem {
    type Serialized = DatabaseItemSerialized;
}

#[derive(Debug, Clone, EnumIs)]
pub enum DatabaseItemRef<'a> {
    Ship(&'a RegistryEntry<crate::model::ship::Ship>),
    ShipBuild(&'a RegistryEntry<crate::model::ship_build::ShipBuild>),
    ComponentStats(&'a RegistryEntry<crate::model::component_stats::ComponentStats>),
    Variable(&'a RegistryEntry<crate::model::variable::Variable>),
    Component(&'a RegistryEntry<crate::model::component::Component>),
    Fleet(&'a RegistryEntry<crate::model::fleet::Fleet>),
    CombatSettings(&'a RegistryEntry<crate::model::combat_settings::CombatSettings>),
    Device(&'a RegistryEntry<crate::model::device::Device>),
}

impl<'a> DatabaseItemTrait for DatabaseItemRef<'a> {
    fn id(&self) -> SlabMapUntypedId {
        match self {
            Self::Ship(s) => s.id(),
            Self::ShipBuild(s) => s.id(),
            Self::ComponentStats(s) => s.id(),
            Self::Variable(s) => s.id(),
            Self::Component(s) => s.id(),
            Self::Fleet(s) => s.id(),
            Self::CombatSettings(s) => s.id(),
            Self::Device(s) => s.id(),
        }
    }

    fn kind(&self) -> DatabaseItemKind {
        match self {
            Self::Ship(s) => s.kind(),
            Self::ShipBuild(s) => s.kind(),
            Self::ComponentStats(s) => s.kind(),
            Self::Variable(s) => s.kind(),
            Self::Component(s) => s.kind(),
            Self::Fleet(s) => s.kind(),
            Self::CombatSettings(s) => s.kind(),
            Self::Device(s) => s.kind(),
        }
    }
}

impl<'a> From<&'a RegistryEntry<crate::model::ship::Ship>> for DatabaseItemRef<'a> {
    fn from(value: &'a RegistryEntry<crate::model::ship::Ship>) -> Self {
        Self::Ship(value)
    }
}

impl<'a> From<&'a RegistryEntry<crate::model::ship_build::ShipBuild>> for DatabaseItemRef<'a> {
    fn from(value: &'a RegistryEntry<crate::model::ship_build::ShipBuild>) -> Self {
        Self::ShipBuild(value)
    }
}

impl<'a> From<&'a RegistryEntry<crate::model::component_stats::ComponentStats>>
    for DatabaseItemRef<'a>
{
    fn from(value: &'a RegistryEntry<crate::model::component_stats::ComponentStats>) -> Self {
        Self::ComponentStats(value)
    }
}

impl<'a> From<&'a RegistryEntry<crate::model::variable::Variable>> for DatabaseItemRef<'a> {
    fn from(value: &'a RegistryEntry<crate::model::variable::Variable>) -> Self {
        Self::Variable(value)
    }
}

impl<'a> From<&'a RegistryEntry<crate::model::component::Component>> for DatabaseItemRef<'a> {
    fn from(value: &'a RegistryEntry<crate::model::component::Component>) -> Self {
        Self::Component(value)
    }
}

impl<'a> From<&'a RegistryEntry<crate::model::fleet::Fleet>> for DatabaseItemRef<'a> {
    fn from(value: &'a RegistryEntry<crate::model::fleet::Fleet>) -> Self {
        Self::Fleet(value)
    }
}

impl<'a> From<&'a RegistryEntry<crate::model::combat_settings::CombatSettings>>
    for DatabaseItemRef<'a>
{
    fn from(value: &'a RegistryEntry<crate::model::combat_settings::CombatSettings>) -> Self {
        Self::CombatSettings(value)
    }
}

impl<'a> From<&'a RegistryEntry<crate::model::device::Device>> for DatabaseItemRef<'a> {
    fn from(value: &'a RegistryEntry<crate::model::device::Device>) -> Self {
        Self::Device(value)
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::ship::Ship>>> for RegistryKeyOrId<ItemId> {
    fn from(item: SlabMapId<RegistryEntry<crate::model::ship::Ship>>) -> RegistryKeyOrId<ItemId> {
        RegistryKeyOrId {
            kind: DatabaseItemKind::Ship,
            id: SlabMapKeyOrUntypedId::Id(item.as_untyped()),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::ship::Ship>>> for RegistryId {
    fn from(item: SlabMapId<RegistryEntry<crate::model::ship::Ship>>) -> RegistryId {
        RegistryId {
            kind: DatabaseItemKind::Ship,
            id: item.as_untyped(),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::ship_build::ShipBuild>>>
    for RegistryKeyOrId<ItemId>
{
    fn from(
        item: SlabMapId<RegistryEntry<crate::model::ship_build::ShipBuild>>,
    ) -> RegistryKeyOrId<ItemId> {
        RegistryKeyOrId {
            kind: DatabaseItemKind::ShipBuild,
            id: SlabMapKeyOrUntypedId::Id(item.as_untyped()),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::ship_build::ShipBuild>>> for RegistryId {
    fn from(item: SlabMapId<RegistryEntry<crate::model::ship_build::ShipBuild>>) -> RegistryId {
        RegistryId {
            kind: DatabaseItemKind::ShipBuild,
            id: item.as_untyped(),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::component_stats::ComponentStats>>>
    for RegistryKeyOrId<ItemId>
{
    fn from(
        item: SlabMapId<RegistryEntry<crate::model::component_stats::ComponentStats>>,
    ) -> RegistryKeyOrId<ItemId> {
        RegistryKeyOrId {
            kind: DatabaseItemKind::ComponentStats,
            id: SlabMapKeyOrUntypedId::Id(item.as_untyped()),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::component_stats::ComponentStats>>> for RegistryId {
    fn from(
        item: SlabMapId<RegistryEntry<crate::model::component_stats::ComponentStats>>,
    ) -> RegistryId {
        RegistryId {
            kind: DatabaseItemKind::ComponentStats,
            id: item.as_untyped(),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::variable::Variable>>> for RegistryKeyOrId<ItemId> {
    fn from(
        item: SlabMapId<RegistryEntry<crate::model::variable::Variable>>,
    ) -> RegistryKeyOrId<ItemId> {
        RegistryKeyOrId {
            kind: DatabaseItemKind::Variable,
            id: SlabMapKeyOrUntypedId::Id(item.as_untyped()),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::variable::Variable>>> for RegistryId {
    fn from(item: SlabMapId<RegistryEntry<crate::model::variable::Variable>>) -> RegistryId {
        RegistryId {
            kind: DatabaseItemKind::Variable,
            id: item.as_untyped(),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::component::Component>>>
    for RegistryKeyOrId<ItemId>
{
    fn from(
        item: SlabMapId<RegistryEntry<crate::model::component::Component>>,
    ) -> RegistryKeyOrId<ItemId> {
        RegistryKeyOrId {
            kind: DatabaseItemKind::Component,
            id: SlabMapKeyOrUntypedId::Id(item.as_untyped()),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::component::Component>>> for RegistryId {
    fn from(item: SlabMapId<RegistryEntry<crate::model::component::Component>>) -> RegistryId {
        RegistryId {
            kind: DatabaseItemKind::Component,
            id: item.as_untyped(),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::fleet::Fleet>>> for RegistryKeyOrId<ItemId> {
    fn from(item: SlabMapId<RegistryEntry<crate::model::fleet::Fleet>>) -> RegistryKeyOrId<ItemId> {
        RegistryKeyOrId {
            kind: DatabaseItemKind::Fleet,
            id: SlabMapKeyOrUntypedId::Id(item.as_untyped()),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::fleet::Fleet>>> for RegistryId {
    fn from(item: SlabMapId<RegistryEntry<crate::model::fleet::Fleet>>) -> RegistryId {
        RegistryId {
            kind: DatabaseItemKind::Fleet,
            id: item.as_untyped(),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::combat_settings::CombatSettings>>>
    for RegistryKeyOrId<ItemId>
{
    fn from(
        item: SlabMapId<RegistryEntry<crate::model::combat_settings::CombatSettings>>,
    ) -> RegistryKeyOrId<ItemId> {
        RegistryKeyOrId {
            kind: DatabaseItemKind::CombatSettings,
            id: SlabMapKeyOrUntypedId::Id(item.as_untyped()),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::combat_settings::CombatSettings>>> for RegistryId {
    fn from(
        item: SlabMapId<RegistryEntry<crate::model::combat_settings::CombatSettings>>,
    ) -> RegistryId {
        RegistryId {
            kind: DatabaseItemKind::CombatSettings,
            id: item.as_untyped(),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::device::Device>>> for RegistryKeyOrId<ItemId> {
    fn from(
        item: SlabMapId<RegistryEntry<crate::model::device::Device>>,
    ) -> RegistryKeyOrId<ItemId> {
        RegistryKeyOrId {
            kind: DatabaseItemKind::Device,
            id: SlabMapKeyOrUntypedId::Id(item.as_untyped()),
        }
    }
}

impl From<SlabMapId<RegistryEntry<crate::model::device::Device>>> for RegistryId {
    fn from(item: SlabMapId<RegistryEntry<crate::model::device::Device>>) -> RegistryId {
        RegistryId {
            kind: DatabaseItemKind::Device,
            id: item.as_untyped(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ModRegistry {
    pub ship: ModelStore<RegistryEntry<crate::model::ship::Ship>>,
    pub ship_build: ModelStore<RegistryEntry<crate::model::ship_build::ShipBuild>>,
    pub component_stats: ModelStore<RegistryEntry<crate::model::component_stats::ComponentStats>>,
    pub variable: ModelStore<RegistryEntry<crate::model::variable::Variable>>,
    pub component: ModelStore<RegistryEntry<crate::model::component::Component>>,
    pub fleet: ModelStore<RegistryEntry<crate::model::fleet::Fleet>>,
    pub combat_settings: ModelStore<RegistryEntry<crate::model::combat_settings::CombatSettings>>,
    pub device: ModelStore<RegistryEntry<crate::model::device::Device>>,
}

impl ModRegistry {
    pub fn get(&self, id: RegistryKeyOrId<ItemId>) -> Option<DatabaseItemRef> {
        match id.kind {
            DatabaseItemKind::Ship => self.ship.get_by_untyped(id.id).map(DatabaseItemRef::from),
            DatabaseItemKind::ShipBuild => self
                .ship_build
                .get_by_untyped(id.id)
                .map(DatabaseItemRef::from),
            DatabaseItemKind::ComponentStats => self
                .component_stats
                .get_by_untyped(id.id)
                .map(DatabaseItemRef::from),
            DatabaseItemKind::Variable => self
                .variable
                .get_by_untyped(id.id)
                .map(DatabaseItemRef::from),
            DatabaseItemKind::Component => self
                .component
                .get_by_untyped(id.id)
                .map(DatabaseItemRef::from),
            DatabaseItemKind::Fleet => self.fleet.get_by_untyped(id.id).map(DatabaseItemRef::from),
            DatabaseItemKind::CombatSettings => self
                .combat_settings
                .get_by_untyped(id.id)
                .map(DatabaseItemRef::from),
            DatabaseItemKind::Device => {
                self.device.get_by_untyped(id.id).map(DatabaseItemRef::from)
            }
        }
    }
    pub fn get_by_id(&self, id: RegistryId) -> Option<DatabaseItemRef> {
        match id.kind {
            DatabaseItemKind::Ship => self
                .ship
                .get_by_untyped_id(id.id)
                .map(DatabaseItemRef::from),
            DatabaseItemKind::ShipBuild => self
                .ship_build
                .get_by_untyped_id(id.id)
                .map(DatabaseItemRef::from),
            DatabaseItemKind::ComponentStats => self
                .component_stats
                .get_by_untyped_id(id.id)
                .map(DatabaseItemRef::from),
            DatabaseItemKind::Variable => self
                .variable
                .get_by_untyped_id(id.id)
                .map(DatabaseItemRef::from),
            DatabaseItemKind::Component => self
                .component
                .get_by_untyped_id(id.id)
                .map(DatabaseItemRef::from),
            DatabaseItemKind::Fleet => self
                .fleet
                .get_by_untyped_id(id.id)
                .map(DatabaseItemRef::from),
            DatabaseItemKind::CombatSettings => self
                .combat_settings
                .get_by_untyped_id(id.id)
                .map(DatabaseItemRef::from),
            DatabaseItemKind::Device => self
                .device
                .get_by_untyped_id(id.id)
                .map(DatabaseItemRef::from),
        }
    }
}

impl Index<SlabMapId<RegistryEntry<crate::model::ship::Ship>>> for ModRegistry {
    type Output = RegistryEntry<crate::model::ship::Ship>;

    fn index(&self, index: SlabMapId<RegistryEntry<crate::model::ship::Ship>>) -> &Self::Output {
        &self.ship[index]
    }
}

impl Index<&SlabMapId<RegistryEntry<crate::model::ship::Ship>>> for ModRegistry {
    type Output = RegistryEntry<crate::model::ship::Ship>;

    fn index(&self, index: &SlabMapId<RegistryEntry<crate::model::ship::Ship>>) -> &Self::Output {
        &self.ship[*index]
    }
}

impl Index<SlabMapId<RegistryEntry<crate::model::ship_build::ShipBuild>>> for ModRegistry {
    type Output = RegistryEntry<crate::model::ship_build::ShipBuild>;

    fn index(
        &self,
        index: SlabMapId<RegistryEntry<crate::model::ship_build::ShipBuild>>,
    ) -> &Self::Output {
        &self.ship_build[index]
    }
}

impl Index<&SlabMapId<RegistryEntry<crate::model::ship_build::ShipBuild>>> for ModRegistry {
    type Output = RegistryEntry<crate::model::ship_build::ShipBuild>;

    fn index(
        &self,
        index: &SlabMapId<RegistryEntry<crate::model::ship_build::ShipBuild>>,
    ) -> &Self::Output {
        &self.ship_build[*index]
    }
}

impl Index<SlabMapId<RegistryEntry<crate::model::component_stats::ComponentStats>>>
    for ModRegistry
{
    type Output = RegistryEntry<crate::model::component_stats::ComponentStats>;

    fn index(
        &self,
        index: SlabMapId<RegistryEntry<crate::model::component_stats::ComponentStats>>,
    ) -> &Self::Output {
        &self.component_stats[index]
    }
}

impl Index<&SlabMapId<RegistryEntry<crate::model::component_stats::ComponentStats>>>
    for ModRegistry
{
    type Output = RegistryEntry<crate::model::component_stats::ComponentStats>;

    fn index(
        &self,
        index: &SlabMapId<RegistryEntry<crate::model::component_stats::ComponentStats>>,
    ) -> &Self::Output {
        &self.component_stats[*index]
    }
}

impl Index<SlabMapId<RegistryEntry<crate::model::variable::Variable>>> for ModRegistry {
    type Output = RegistryEntry<crate::model::variable::Variable>;

    fn index(
        &self,
        index: SlabMapId<RegistryEntry<crate::model::variable::Variable>>,
    ) -> &Self::Output {
        &self.variable[index]
    }
}

impl Index<&SlabMapId<RegistryEntry<crate::model::variable::Variable>>> for ModRegistry {
    type Output = RegistryEntry<crate::model::variable::Variable>;

    fn index(
        &self,
        index: &SlabMapId<RegistryEntry<crate::model::variable::Variable>>,
    ) -> &Self::Output {
        &self.variable[*index]
    }
}

impl Index<SlabMapId<RegistryEntry<crate::model::component::Component>>> for ModRegistry {
    type Output = RegistryEntry<crate::model::component::Component>;

    fn index(
        &self,
        index: SlabMapId<RegistryEntry<crate::model::component::Component>>,
    ) -> &Self::Output {
        &self.component[index]
    }
}

impl Index<&SlabMapId<RegistryEntry<crate::model::component::Component>>> for ModRegistry {
    type Output = RegistryEntry<crate::model::component::Component>;

    fn index(
        &self,
        index: &SlabMapId<RegistryEntry<crate::model::component::Component>>,
    ) -> &Self::Output {
        &self.component[*index]
    }
}

impl Index<SlabMapId<RegistryEntry<crate::model::fleet::Fleet>>> for ModRegistry {
    type Output = RegistryEntry<crate::model::fleet::Fleet>;

    fn index(&self, index: SlabMapId<RegistryEntry<crate::model::fleet::Fleet>>) -> &Self::Output {
        &self.fleet[index]
    }
}

impl Index<&SlabMapId<RegistryEntry<crate::model::fleet::Fleet>>> for ModRegistry {
    type Output = RegistryEntry<crate::model::fleet::Fleet>;

    fn index(&self, index: &SlabMapId<RegistryEntry<crate::model::fleet::Fleet>>) -> &Self::Output {
        &self.fleet[*index]
    }
}

impl Index<SlabMapId<RegistryEntry<crate::model::combat_settings::CombatSettings>>>
    for ModRegistry
{
    type Output = RegistryEntry<crate::model::combat_settings::CombatSettings>;

    fn index(
        &self,
        index: SlabMapId<RegistryEntry<crate::model::combat_settings::CombatSettings>>,
    ) -> &Self::Output {
        &self.combat_settings[index]
    }
}

impl Index<&SlabMapId<RegistryEntry<crate::model::combat_settings::CombatSettings>>>
    for ModRegistry
{
    type Output = RegistryEntry<crate::model::combat_settings::CombatSettings>;

    fn index(
        &self,
        index: &SlabMapId<RegistryEntry<crate::model::combat_settings::CombatSettings>>,
    ) -> &Self::Output {
        &self.combat_settings[*index]
    }
}

impl Index<SlabMapId<RegistryEntry<crate::model::device::Device>>> for ModRegistry {
    type Output = RegistryEntry<crate::model::device::Device>;

    fn index(
        &self,
        index: SlabMapId<RegistryEntry<crate::model::device::Device>>,
    ) -> &Self::Output {
        &self.device[index]
    }
}

impl Index<&SlabMapId<RegistryEntry<crate::model::device::Device>>> for ModRegistry {
    type Output = RegistryEntry<crate::model::device::Device>;

    fn index(
        &self,
        index: &SlabMapId<RegistryEntry<crate::model::device::Device>>,
    ) -> &Self::Output {
        &self.device[*index]
    }
}

pub type ShipId = SlabMapId<RegistryEntry<crate::model::ship::Ship>>;
pub type ShipOrId = serialization::InlineOrId<crate::model::ship::Ship>;

impl DatabaseItemTrait for RegistryEntry<crate::model::ship::Ship> {
    fn id(&self) -> SlabMapUntypedId {
        self.id.as_untyped()
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::Ship
    }
}

impl ModelKind for RegistryEntry<crate::model::ship::Ship> {
    fn kind() -> DatabaseItemKind {
        DatabaseItemKind::Ship
    }
}

impl DatabaseItemSerializedTrait for <RegistryEntry<crate::model::ship::Ship> as serialization::ModelDeserializableFallbackType>::Serialized {
    fn id(&self) -> &ItemId {
        &self.id
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::Ship
    }
}

impl serialization::ModelDeserializable<SlabMapId<RegistryEntry<crate::model::ship::Ship>>>
for <RegistryEntry<crate::model::ship::Ship> as serialization::ModelDeserializableFallbackType>::Serialized
{
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<SlabMapId<RegistryEntry<crate::model::ship::Ship>>, serialization::DeserializationError> {
        let reserved = serialization::reserve(&mut registry.ship, self.id.clone())?;
        let data = serialization::ModelDeserializable::<crate::model::ship::Ship>::deserialize(
            self.data, registry,
        )
            .map_err(|e| {
                e.context(
                    serialization::DeserializationErrorStackItem::Item(
                        self.id,
                        <RegistryEntry::<crate::model::ship::Ship> as ModelKind>::kind(),
                    ),
                )
            })?;
        let id = reserved.raw();
        let model = RegistryEntry { id, data };
        let id = serialization::insert_reserved(&mut registry.ship, reserved, model);
        Ok(id)
    }
}

#[automatically_derived]
impl serialization::ModelDeserializable<ShipId> for &str {
    fn deserialize(
        self,
        registry: &mut crate::model::PartialModRegistry,
    ) -> Result<ShipId, serialization::DeserializationError> {
        if let Some(id) = serialization::get_reserved_key(&mut registry.ship, self) {
            return Ok(id);
        }
        let Some(other) = registry.raw.ship.remove(self) else {
            return Err(serialization::DeserializationErrorKind::MissingItem(
                self.to_string(),
                <RegistryEntry<crate::model::ship::Ship> as ModelKind>::kind(),
            )
            .into());
        };
        other.deserialize(registry)
    }
}

pub type ShipBuildId = SlabMapId<RegistryEntry<crate::model::ship_build::ShipBuild>>;
pub type ShipBuildOrId = serialization::InlineOrId<crate::model::ship_build::ShipBuild>;

impl DatabaseItemTrait for RegistryEntry<crate::model::ship_build::ShipBuild> {
    fn id(&self) -> SlabMapUntypedId {
        self.id.as_untyped()
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::ShipBuild
    }
}

impl ModelKind for RegistryEntry<crate::model::ship_build::ShipBuild> {
    fn kind() -> DatabaseItemKind {
        DatabaseItemKind::ShipBuild
    }
}

impl DatabaseItemSerializedTrait for <RegistryEntry<crate::model::ship_build::ShipBuild> as serialization::ModelDeserializableFallbackType>::Serialized {
    fn id(&self) -> &ItemId {
        &self.id
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::ShipBuild
    }
}

impl serialization::ModelDeserializable<SlabMapId<RegistryEntry<crate::model::ship_build::ShipBuild>>>
for <RegistryEntry<crate::model::ship_build::ShipBuild> as serialization::ModelDeserializableFallbackType>::Serialized
{
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<SlabMapId<RegistryEntry<crate::model::ship_build::ShipBuild>>, serialization::DeserializationError> {
        let reserved = serialization::reserve(&mut registry.ship_build, self.id.clone())?;
        let data = serialization::ModelDeserializable::<crate::model::ship_build::ShipBuild>::deserialize(
            self.data, registry,
        )
            .map_err(|e| {
                e.context(
                    serialization::DeserializationErrorStackItem::Item(
                        self.id,
                        <RegistryEntry::<crate::model::ship_build::ShipBuild> as ModelKind>::kind(),
                    ),
                )
            })?;
        let id = reserved.raw();
        let model = RegistryEntry { id, data };
        let id = serialization::insert_reserved(&mut registry.ship_build, reserved, model);
        Ok(id)
    }
}

#[automatically_derived]
impl serialization::ModelDeserializable<ShipBuildId> for &str {
    fn deserialize(
        self,
        registry: &mut crate::model::PartialModRegistry,
    ) -> Result<ShipBuildId, serialization::DeserializationError> {
        if let Some(id) = serialization::get_reserved_key(&mut registry.ship_build, self) {
            return Ok(id);
        }
        let Some(other) = registry.raw.ship_build.remove(self) else {
            return Err(serialization::DeserializationErrorKind::MissingItem(
                self.to_string(),
                <RegistryEntry<crate::model::ship_build::ShipBuild> as ModelKind>::kind(),
            )
            .into());
        };
        other.deserialize(registry)
    }
}

pub type ComponentStatsId = SlabMapId<RegistryEntry<crate::model::component_stats::ComponentStats>>;
pub type ComponentStatsOrId =
    serialization::InlineOrId<crate::model::component_stats::ComponentStats>;

impl DatabaseItemTrait for RegistryEntry<crate::model::component_stats::ComponentStats> {
    fn id(&self) -> SlabMapUntypedId {
        self.id.as_untyped()
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::ComponentStats
    }
}

impl ModelKind for RegistryEntry<crate::model::component_stats::ComponentStats> {
    fn kind() -> DatabaseItemKind {
        DatabaseItemKind::ComponentStats
    }
}

impl DatabaseItemSerializedTrait for <RegistryEntry<crate::model::component_stats::ComponentStats> as serialization::ModelDeserializableFallbackType>::Serialized {
    fn id(&self) -> &ItemId {
        &self.id
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::ComponentStats
    }
}

impl serialization::ModelDeserializable<SlabMapId<RegistryEntry<crate::model::component_stats::ComponentStats>>>
for <RegistryEntry<crate::model::component_stats::ComponentStats> as serialization::ModelDeserializableFallbackType>::Serialized
{
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<SlabMapId<RegistryEntry<crate::model::component_stats::ComponentStats>>, serialization::DeserializationError> {
        let reserved = serialization::reserve(&mut registry.component_stats, self.id.clone())?;
        let data = serialization::ModelDeserializable::<crate::model::component_stats::ComponentStats>::deserialize(
            self.data, registry,
        )
            .map_err(|e| {
                e.context(
                    serialization::DeserializationErrorStackItem::Item(
                        self.id,
                        <RegistryEntry::<crate::model::component_stats::ComponentStats> as ModelKind>::kind(),
                    ),
                )
            })?;
        let id = reserved.raw();
        let model = RegistryEntry { id, data };
        let id = serialization::insert_reserved(&mut registry.component_stats, reserved, model);
        Ok(id)
    }
}

#[automatically_derived]
impl serialization::ModelDeserializable<ComponentStatsId> for &str {
    fn deserialize(
        self,
        registry: &mut crate::model::PartialModRegistry,
    ) -> Result<ComponentStatsId, serialization::DeserializationError> {
        if let Some(id) = serialization::get_reserved_key(&mut registry.component_stats, self) {
            return Ok(id);
        }
        let Some(other) = registry.raw.component_stats.remove(self) else {
            return Err(serialization::DeserializationErrorKind::MissingItem(
                self.to_string(),
                <RegistryEntry<crate::model::component_stats::ComponentStats> as ModelKind>::kind(),
            )
            .into());
        };
        other.deserialize(registry)
    }
}

pub type VariableId = SlabMapId<RegistryEntry<crate::model::variable::Variable>>;
pub type VariableOrId = serialization::InlineOrId<crate::model::variable::Variable>;

impl DatabaseItemTrait for RegistryEntry<crate::model::variable::Variable> {
    fn id(&self) -> SlabMapUntypedId {
        self.id.as_untyped()
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::Variable
    }
}

impl ModelKind for RegistryEntry<crate::model::variable::Variable> {
    fn kind() -> DatabaseItemKind {
        DatabaseItemKind::Variable
    }
}

impl DatabaseItemSerializedTrait for <RegistryEntry<crate::model::variable::Variable> as serialization::ModelDeserializableFallbackType>::Serialized {
    fn id(&self) -> &ItemId {
        &self.id
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::Variable
    }
}

impl serialization::ModelDeserializable<SlabMapId<RegistryEntry<crate::model::variable::Variable>>>
for <RegistryEntry<crate::model::variable::Variable> as serialization::ModelDeserializableFallbackType>::Serialized
{
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<SlabMapId<RegistryEntry<crate::model::variable::Variable>>, serialization::DeserializationError> {
        let reserved = serialization::reserve(&mut registry.variable, self.id.clone())?;
        let data = serialization::ModelDeserializable::<crate::model::variable::Variable>::deserialize(
            self.data, registry,
        )
            .map_err(|e| {
                e.context(
                    serialization::DeserializationErrorStackItem::Item(
                        self.id,
                        <RegistryEntry::<crate::model::variable::Variable> as ModelKind>::kind(),
                    ),
                )
            })?;
        let id = reserved.raw();
        let model = RegistryEntry { id, data };
        let id = serialization::insert_reserved(&mut registry.variable, reserved, model);
        Ok(id)
    }
}

#[automatically_derived]
impl serialization::ModelDeserializable<VariableId> for &str {
    fn deserialize(
        self,
        registry: &mut crate::model::PartialModRegistry,
    ) -> Result<VariableId, serialization::DeserializationError> {
        if let Some(id) = serialization::get_reserved_key(&mut registry.variable, self) {
            return Ok(id);
        }
        let Some(other) = registry.raw.variable.remove(self) else {
            return Err(serialization::DeserializationErrorKind::MissingItem(
                self.to_string(),
                <RegistryEntry<crate::model::variable::Variable> as ModelKind>::kind(),
            )
            .into());
        };
        other.deserialize(registry)
    }
}

pub type ComponentId = SlabMapId<RegistryEntry<crate::model::component::Component>>;
pub type ComponentOrId = serialization::InlineOrId<crate::model::component::Component>;

impl DatabaseItemTrait for RegistryEntry<crate::model::component::Component> {
    fn id(&self) -> SlabMapUntypedId {
        self.id.as_untyped()
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::Component
    }
}

impl ModelKind for RegistryEntry<crate::model::component::Component> {
    fn kind() -> DatabaseItemKind {
        DatabaseItemKind::Component
    }
}

impl DatabaseItemSerializedTrait for <RegistryEntry<crate::model::component::Component> as serialization::ModelDeserializableFallbackType>::Serialized {
    fn id(&self) -> &ItemId {
        &self.id
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::Component
    }
}

impl serialization::ModelDeserializable<SlabMapId<RegistryEntry<crate::model::component::Component>>>
for <RegistryEntry<crate::model::component::Component> as serialization::ModelDeserializableFallbackType>::Serialized
{
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<SlabMapId<RegistryEntry<crate::model::component::Component>>, serialization::DeserializationError> {
        let reserved = serialization::reserve(&mut registry.component, self.id.clone())?;
        let data = serialization::ModelDeserializable::<crate::model::component::Component>::deserialize(
            self.data, registry,
        )
            .map_err(|e| {
                e.context(
                    serialization::DeserializationErrorStackItem::Item(
                        self.id,
                        <RegistryEntry::<crate::model::component::Component> as ModelKind>::kind(),
                    ),
                )
            })?;
        let id = reserved.raw();
        let model = RegistryEntry { id, data };
        let id = serialization::insert_reserved(&mut registry.component, reserved, model);
        Ok(id)
    }
}

#[automatically_derived]
impl serialization::ModelDeserializable<ComponentId> for &str {
    fn deserialize(
        self,
        registry: &mut crate::model::PartialModRegistry,
    ) -> Result<ComponentId, serialization::DeserializationError> {
        if let Some(id) = serialization::get_reserved_key(&mut registry.component, self) {
            return Ok(id);
        }
        let Some(other) = registry.raw.component.remove(self) else {
            return Err(serialization::DeserializationErrorKind::MissingItem(
                self.to_string(),
                <RegistryEntry<crate::model::component::Component> as ModelKind>::kind(),
            )
            .into());
        };
        other.deserialize(registry)
    }
}

pub type FleetId = SlabMapId<RegistryEntry<crate::model::fleet::Fleet>>;
pub type FleetOrId = serialization::InlineOrId<crate::model::fleet::Fleet>;

impl DatabaseItemTrait for RegistryEntry<crate::model::fleet::Fleet> {
    fn id(&self) -> SlabMapUntypedId {
        self.id.as_untyped()
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::Fleet
    }
}

impl ModelKind for RegistryEntry<crate::model::fleet::Fleet> {
    fn kind() -> DatabaseItemKind {
        DatabaseItemKind::Fleet
    }
}

impl DatabaseItemSerializedTrait for <RegistryEntry<crate::model::fleet::Fleet> as serialization::ModelDeserializableFallbackType>::Serialized {
    fn id(&self) -> &ItemId {
        &self.id
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::Fleet
    }
}

impl serialization::ModelDeserializable<SlabMapId<RegistryEntry<crate::model::fleet::Fleet>>>
for <RegistryEntry<crate::model::fleet::Fleet> as serialization::ModelDeserializableFallbackType>::Serialized
{
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<SlabMapId<RegistryEntry<crate::model::fleet::Fleet>>, serialization::DeserializationError> {
        let reserved = serialization::reserve(&mut registry.fleet, self.id.clone())?;
        let data = serialization::ModelDeserializable::<crate::model::fleet::Fleet>::deserialize(
            self.data, registry,
        )
            .map_err(|e| {
                e.context(
                    serialization::DeserializationErrorStackItem::Item(
                        self.id,
                        <RegistryEntry::<crate::model::fleet::Fleet> as ModelKind>::kind(),
                    ),
                )
            })?;
        let id = reserved.raw();
        let model = RegistryEntry { id, data };
        let id = serialization::insert_reserved(&mut registry.fleet, reserved, model);
        Ok(id)
    }
}

#[automatically_derived]
impl serialization::ModelDeserializable<FleetId> for &str {
    fn deserialize(
        self,
        registry: &mut crate::model::PartialModRegistry,
    ) -> Result<FleetId, serialization::DeserializationError> {
        if let Some(id) = serialization::get_reserved_key(&mut registry.fleet, self) {
            return Ok(id);
        }
        let Some(other) = registry.raw.fleet.remove(self) else {
            return Err(serialization::DeserializationErrorKind::MissingItem(
                self.to_string(),
                <RegistryEntry<crate::model::fleet::Fleet> as ModelKind>::kind(),
            )
            .into());
        };
        other.deserialize(registry)
    }
}

pub type CombatSettingsId = SlabMapId<RegistryEntry<crate::model::combat_settings::CombatSettings>>;
pub type CombatSettingsOrId =
    serialization::InlineOrId<crate::model::combat_settings::CombatSettings>;

impl DatabaseItemTrait for RegistryEntry<crate::model::combat_settings::CombatSettings> {
    fn id(&self) -> SlabMapUntypedId {
        self.id.as_untyped()
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::CombatSettings
    }
}

impl ModelKind for RegistryEntry<crate::model::combat_settings::CombatSettings> {
    fn kind() -> DatabaseItemKind {
        DatabaseItemKind::CombatSettings
    }
}

impl DatabaseItemSerializedTrait for <RegistryEntry<crate::model::combat_settings::CombatSettings> as serialization::ModelDeserializableFallbackType>::Serialized {
    fn id(&self) -> &ItemId {
        &self.id
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::CombatSettings
    }
}

impl serialization::ModelDeserializable<SlabMapId<RegistryEntry<crate::model::combat_settings::CombatSettings>>>
for <RegistryEntry<crate::model::combat_settings::CombatSettings> as serialization::ModelDeserializableFallbackType>::Serialized
{
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<SlabMapId<RegistryEntry<crate::model::combat_settings::CombatSettings>>, serialization::DeserializationError> {
        let reserved = serialization::reserve(&mut registry.combat_settings, self.id.clone())?;
        let data = serialization::ModelDeserializable::<crate::model::combat_settings::CombatSettings>::deserialize(
            self.data, registry,
        )
            .map_err(|e| {
                e.context(
                    serialization::DeserializationErrorStackItem::Item(
                        self.id,
                        <RegistryEntry::<crate::model::combat_settings::CombatSettings> as ModelKind>::kind(),
                    ),
                )
            })?;
        let id = reserved.raw();
        let model = RegistryEntry { id, data };
        let id = serialization::insert_reserved(&mut registry.combat_settings, reserved, model);
        Ok(id)
    }
}

#[automatically_derived]
impl serialization::ModelDeserializable<CombatSettingsId> for &str {
    fn deserialize(
        self,
        registry: &mut crate::model::PartialModRegistry,
    ) -> Result<CombatSettingsId, serialization::DeserializationError> {
        if let Some(id) = serialization::get_reserved_key(&mut registry.combat_settings, self) {
            return Ok(id);
        }
        let Some(other) = registry.raw.combat_settings.remove(self) else {
            return Err(serialization::DeserializationErrorKind::MissingItem(
                self.to_string(),
                <RegistryEntry<crate::model::combat_settings::CombatSettings> as ModelKind>::kind(),
            )
            .into());
        };
        other.deserialize(registry)
    }
}

pub type DeviceId = SlabMapId<RegistryEntry<crate::model::device::Device>>;
pub type DeviceOrId = serialization::InlineOrId<crate::model::device::Device>;

impl DatabaseItemTrait for RegistryEntry<crate::model::device::Device> {
    fn id(&self) -> SlabMapUntypedId {
        self.id.as_untyped()
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::Device
    }
}

impl ModelKind for RegistryEntry<crate::model::device::Device> {
    fn kind() -> DatabaseItemKind {
        DatabaseItemKind::Device
    }
}

impl DatabaseItemSerializedTrait for <RegistryEntry<crate::model::device::Device> as serialization::ModelDeserializableFallbackType>::Serialized {
    fn id(&self) -> &ItemId {
        &self.id
    }
    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::Device
    }
}

impl serialization::ModelDeserializable<SlabMapId<RegistryEntry<crate::model::device::Device>>>
for <RegistryEntry<crate::model::device::Device> as serialization::ModelDeserializableFallbackType>::Serialized
{
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<SlabMapId<RegistryEntry<crate::model::device::Device>>, serialization::DeserializationError> {
        let reserved = serialization::reserve(&mut registry.device, self.id.clone())?;
        let data = serialization::ModelDeserializable::<crate::model::device::Device>::deserialize(
            self.data, registry,
        )
            .map_err(|e| {
                e.context(
                    serialization::DeserializationErrorStackItem::Item(
                        self.id,
                        <RegistryEntry::<crate::model::device::Device> as ModelKind>::kind(),
                    ),
                )
            })?;
        let id = reserved.raw();
        let model = RegistryEntry { id, data };
        let id = serialization::insert_reserved(&mut registry.device, reserved, model);
        Ok(id)
    }
}

#[automatically_derived]
impl serialization::ModelDeserializable<DeviceId> for &str {
    fn deserialize(
        self,
        registry: &mut crate::model::PartialModRegistry,
    ) -> Result<DeviceId, serialization::DeserializationError> {
        if let Some(id) = serialization::get_reserved_key(&mut registry.device, self) {
            return Ok(id);
        }
        let Some(other) = registry.raw.device.remove(self) else {
            return Err(serialization::DeserializationErrorKind::MissingItem(
                self.to_string(),
                <RegistryEntry<crate::model::device::Device> as ModelKind>::kind(),
            )
            .into());
        };
        other.deserialize(registry)
    }
}

impl DatabaseItemSerialized {
    pub fn schema() -> RootSchema {
        schemars::schema_for!(Self)
    }
}

#[derive(Debug, Default)]
struct ModAssets {
    pub images: FxHashMap<String, (PathBuf, Handle<Image>)>,
}

impl ModRegistry {
    pub fn build<'a>(
        items: impl IntoIterator<Item = (impl AsRef<Path>, &'a DatabaseAsset)>,
        images: impl IntoIterator<Item = (impl AsRef<Path>, Handle<Image>)>,
    ) -> Result<Self, serialization::DeserializationError> {
        let mut raws = RawModRegistry::default();
        for (_path, item) in items.into_iter() {
            if let Err(item) = raws.insert(item.0.clone()) {
                return Err(serialization::DeserializationErrorKind::DuplicateItem(
                    item.id().clone(),
                    item.kind(),
                )
                .into());
            }
        }

        let mut assets = ModAssets::default();

        images
            .into_iter()
            .try_for_each(|(path, image): (_, Handle<Image>)| {
                let path: &Path = path.as_ref();
                let Some(name) = path.file_name() else {
                    return Err(serialization::DeserializationErrorKind::MissingName(
                        path.to_path_buf(),
                    ));
                };

                let Some(name) = name.to_str() else {
                    return Err(serialization::DeserializationErrorKind::NonUtf8Path(
                        path.to_path_buf(),
                    ));
                };

                match assets.images.entry(name.to_ascii_lowercase()) {
                    Entry::Occupied(e) => {
                        return Err(serialization::DeserializationErrorKind::DuplicateImage {
                            name: name.to_string(),
                            path_a: e.get().0.clone(),
                            path_b: path.to_path_buf(),
                        });
                    }
                    Entry::Vacant(e) => {
                        e.insert((path.to_path_buf(), image));
                    }
                }

                Ok(())
            })?;

        let partial = PartialModRegistry {
            raw: raws,
            assets,
            ..Default::default()
        };

        partial.deserialize()
    }
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

#[derive(
    Debug, serde::Deserialize, serde::Serialize, bevy::asset::Asset, bevy::reflect::TypePath,
)]
#[serde(transparent)]
pub struct DatabaseAsset(pub DatabaseItemSerialized);

pub trait DatabaseItemTrait {
    fn id(&self) -> SlabMapUntypedId;
    fn kind(&self) -> DatabaseItemKind;
}

pub trait DatabaseItemSerializedTrait {
    fn id(&self) -> &ItemId;
    fn kind(&self) -> DatabaseItemKind;
}

pub trait ModelKind {
    fn kind() -> DatabaseItemKind;
}

#[duplicate_item(
ty;
[ & T ]; [ Option < T > ]; [ Vec < T > ];
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
pub struct RegistryKeyOrId<T> {
    kind: DatabaseItemKind,
    id: SlabMapKeyOrUntypedId<T>,
}

impl<T: Hash + Eq> RegistryKeyOrId<T>
where
    ItemId: Borrow<T>,
{
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
