use crate::model::{
    DatabaseItem, DatabaseItemKind, DatabaseItemTrait, ItemId, ModItemValidationError, ModRegistry,
    RegistryId,
};

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ship {
    pub id: ItemId,
    pub sprite: String,
    pub model_scale: f32,
}

impl DatabaseItemTrait for Ship {
    fn id(&self) -> &ItemId {
        &self.id
    }

    fn deserialize(
        self,
        registry: &mut ModRegistry,
    ) -> Result<(RegistryId, Option<DatabaseItem>), ModItemValidationError> {
        let (id, old) = registry.ships.insert(self.id().clone(), self);
        Ok((id.into(), old.map(DatabaseItem::Ship)))
    }

    fn kind(&self) -> DatabaseItemKind {
        DatabaseItemKind::Ship
    }
}
