use crate::registry::SerializationRegistry;
use crate::serialization::error::DeserializationError;
use crate::serialization::{DeserializeModel, SerializationFallback};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct SerializationBoxingWrapper<T>(T);

impl<T: SerializationFallback> SerializationFallback for Arc<T> {
    type Fallback = SerializationBoxingWrapper<T::Fallback>;
}

impl<Registry: SerializationRegistry, T: DeserializeModel<R, Registry>, R>
    DeserializeModel<Arc<R>, Registry> for SerializationBoxingWrapper<T>
{
    #[inline(always)]
    fn deserialize(
        self,
        registry: &mut Registry,
    ) -> Result<Arc<R>, DeserializationError<Registry>> {
        self.0.deserialize(registry).map(Arc::new)
    }
}

impl<T: SerializationFallback> SerializationFallback for Box<T> {
    type Fallback = SerializationBoxingWrapper<T::Fallback>;
}

impl<Registry: SerializationRegistry, T: DeserializeModel<R, Registry>, R>
    DeserializeModel<Box<R>, Registry> for SerializationBoxingWrapper<T>
{
    #[inline(always)]
    fn deserialize(
        self,
        registry: &mut Registry,
    ) -> Result<Box<R>, DeserializationError<Registry>> {
        self.0.deserialize(registry).map(Box::new)
    }
}
