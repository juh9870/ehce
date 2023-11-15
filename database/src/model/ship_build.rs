use database_model_macro::database_model;
use utils::slab_map::SlabMapId;

use crate::model::ship::Ship;

#[database_model]
#[derive(Debug, Clone)]
pub struct ShipBuild {
    #[model(id)]
    pub id: SlabMapId<ShipBuild>,
    pub ship: SlabMapId<Ship>,
}
