use crate::model::ship_build::ShipBuildId;
use database_model_macro::database_model;

#[database_model]
#[derive(Debug, Clone)]
pub struct Fleet {
    #[model(id)]
    pub id: FleetId,
    #[model_serde(flatten)]
    #[model(as_ref)]
    pub data: FleetData,
}

#[database_model]
#[derive(Debug, Clone)]
pub struct FleetData {
    pub builds: Vec<ShipBuildId>,
}
