use crate::model::{ComponentStatsOrId, DeviceOrId};
use database_model_macro::database_model;

#[database_model]
#[derive(Debug, Clone)]
pub struct Component {
    pub stats: ComponentStatsOrId,
    pub devices: Vec<DeviceOrId>,
}
