use crate::model::VariableId;
use database_model_macro::database_model;

#[database_model]
#[derive(Debug, Clone)]
pub struct EngineDevice {
    pub acceleration: VariableId,
    pub speed_cap: VariableId,
    pub angular_acceleration: VariableId,
    pub angular_speed_cap: VariableId,
}
