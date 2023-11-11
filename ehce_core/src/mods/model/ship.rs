use crate::mods::model::{DatabaseItemTrait, ItemId, ModItemValidationError, ModRegistry};

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ship {
    id: ItemId,
    sprite: String,
    model_scale: f32,
}

impl DatabaseItemTrait for Ship {
    fn id(&self) -> &ItemId {
        &self.id
    }

    fn deserialize(self, registry: &mut ModRegistry) -> Result<(), ModItemValidationError> {
        registry.ships.insert(self.id().clone(), self);
        Ok(())
    }
}
