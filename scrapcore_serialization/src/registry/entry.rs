use crate::serialization::SerializationFallback;
use crate::ItemId;
use serde::{Deserialize, Serialize};
use slabmap::SlabMapId;

#[derive(Debug, Clone)]
pub struct RegistryEntry<Data> {
    pub id: SlabMapId<Self>,
    pub data: Data,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegistryEntrySerialized<DataSerialized> {
    pub id: ItemId,
    #[serde(flatten)]
    pub data: DataSerialized,
}

impl<Data: SerializationFallback> SerializationFallback for RegistryEntry<Data> {
    type Fallback = RegistryEntrySerialized<Data::Fallback>;
}

impl<Data> AsRef<Data> for RegistryEntry<Data> {
    fn as_ref(&self) -> &Data {
        &self.data
    }
}
