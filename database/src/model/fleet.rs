use crate::model::ShipBuildId;
use database_model_macro::database_model;

#[database_model]
#[derive(Debug, Clone)]
pub struct Fleet {
    pub builds: Vec<ShipBuildId>,
}
