use crate::registry::entry::RegistryEntry;
use crate::registry::{RegistryHolder, SerializationRegistry};
use slabmap::SlabMapId;

pub trait RegistryIndex<Data> {
    fn get<'a, Registry: SerializationRegistry + RegistryHolder<Data>>(
        &'a self,
        registry: &'a Registry,
    ) -> &'a Data;
}

impl<Data> RegistryIndex<Data> for SlabMapId<RegistryEntry<Data>> {
    fn get<'a, Registry: SerializationRegistry + RegistryHolder<Data>>(
        &'a self,
        registry: &'a Registry,
    ) -> &'a Data {
        &registry.get_registry()[*self].data
    }
}

impl<Data> RegistryIndex<Data> for Data {
    fn get<'a, Registry: SerializationRegistry + RegistryHolder<Data>>(
        &'a self,
        _registry: &'a Registry,
    ) -> &'a Data {
        self
    }
}
