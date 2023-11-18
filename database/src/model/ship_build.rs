use database_model_macro::database_model;
use utils::slab_map::SlabMapId;

use super::component::Component;
use super::ship::Ship;

#[database_model]
#[derive(Debug, Clone)]
pub struct ShipBuild {
    #[model(id)]
    pub id: SlabMapId<ShipBuild>,
    pub ship: SlabMapId<Ship>,
    #[model(ty=Vec<InstalledComponentSerialized>)]
    pub components: Vec<InstalledComponent>,
}

#[database_model]
#[derive(Debug, Clone)]
pub struct InstalledComponent {
    pub component: SlabMapId<Component>,
    pub pos: Option<glam::u32::UVec2>,
}
