use crate::registry::SerializationHub;
use crate::serialization::error::DeserializationError;
use crate::serialization::DeserializeModel;

pub trait DeserializeFrom<Registry: SerializationHub>: Sized {
    fn deserialize_from<U>(
        data: U,
        registry: &mut Registry,
    ) -> Result<Self, DeserializationError<Registry>>
    where
        U: DeserializeModel<Self, Registry>,
    {
        data.deserialize(registry)
    }
}

impl<Registry: SerializationHub, T> DeserializeFrom<Registry> for T {}
