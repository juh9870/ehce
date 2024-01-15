use crate::registry::entry::RegistryEntry;
use crate::registry::index::RegistryIndex;
use crate::registry::{RegistryHolder, SerializationHub};
use crate::serialization::error::DeserializationError;
use crate::serialization::{DeserializeModel, SerializationFallback};
use crate::ItemId;
use serde::{Deserialize, Serialize};
use slabmap::SlabMapId;

#[derive(Debug, Clone)]
pub enum InlineOrId<Data> {
    Id(SlabMapId<RegistryEntry<Data>>),
    Inline(Data),
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum InlineOrIdSerialized<DataSerialized> {
    Id(ItemId),
    Inline(DataSerialized),
}

impl<Data: SerializationFallback> SerializationFallback for InlineOrId<Data> {
    type Fallback = InlineOrIdSerialized<Data::Fallback>;
}

impl<Registry: SerializationHub, Data, DataSerialized: DeserializeModel<Data, Registry>>
    DeserializeModel<InlineOrId<Data>, Registry> for InlineOrIdSerialized<DataSerialized>
where
    for<'a> &'a str: DeserializeModel<SlabMapId<RegistryEntry<Data>>, Registry>,
{
    fn deserialize(
        self,
        registry: &mut Registry,
    ) -> Result<InlineOrId<Data>, DeserializationError<Registry>> {
        Ok(match self {
            InlineOrIdSerialized::Id(id) => InlineOrId::Id(id.deserialize(registry)?),
            InlineOrIdSerialized::Inline(data) => InlineOrId::Inline(data.deserialize(registry)?),
        })
    }
}

impl<Data> RegistryIndex<Data> for InlineOrId<Data> {
    fn get<'a, Registry: SerializationHub + RegistryHolder<Data>>(
        &'a self,
        registry: &'a Registry,
    ) -> &'a Data {
        match self {
            InlineOrId::Id(id) => &registry.get_registry()[*id].data,
            InlineOrId::Inline(data) => data,
        }
    }
}
