use crate::model::component::ComponentId;
use crate::model::ship::ShipId;
use database_model_macro::database_model;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[database_model]
#[derive(Debug, Clone)]
pub struct ShipBuild {
    #[model(id)]
    pub id: ShipBuildId,
    #[model_serde(flatten)]
    #[model(as_ref)]
    pub data: ShipBuildData,
}

#[database_model]
#[derive(Debug, Clone)]
pub struct ShipBuildData {
    pub ship: ShipId,
    pub components: Vec<InstalledComponent>,
}

#[database_model]
#[derive(Debug, Clone)]
pub struct InstalledComponent {
    pub component: ComponentId,
    #[model_serde(with = "UVec2Ref")]
    pub pos: glam::u32::UVec2,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(remote = "glam::u32::UVec2")]
struct UVec2Ref {
    pub x: u32,
    pub y: u32,
}
