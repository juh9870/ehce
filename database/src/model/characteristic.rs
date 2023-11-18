use database_model_macro::database_model;
use utils::slab_map::SlabMapId;

#[database_model]
#[derive(Debug, Clone)]
pub struct Characteristic {
    #[model(id)]
    pub id: SlabMapId<Characteristic>,
    pub name: String,
}
