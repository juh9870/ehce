use database_model_macro::database_model;
use utils::slab_map::SlabMapId;

use super::component::Component;
use super::ship::Ship;

#[database_model]
#[derive(Debug, Clone)]
pub struct ShipBuild {
    #[model(id)]
    pub id: SlabMapId<ShipBuild>,
    #[model_attr(serde(flatten))]
    pub data: ShipBuildData,
}

#[database_model]
#[derive(Debug, Clone)]
pub struct ShipBuildData {
    pub ship: SlabMapId<Ship>,
    pub components: Vec<InstalledComponent>,
}

#[database_model]
#[derive(Debug, Clone)]
pub struct InstalledComponent {
    pub component: SlabMapId<Component>,
    pub pos: glam::u32::UVec2,
}

impl AsRef<ShipBuildData> for ShipBuild {
    fn as_ref(&self) -> &ShipBuildData {
        &self.data
    }
}
