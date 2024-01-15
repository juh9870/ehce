use database_model_macro::database_model;
use engine_device::EngineDevice;

pub mod engine_device;

#[database_model]
#[model_serde(tag = "deviceType")]
#[derive(Debug, Clone)]
pub enum Device {
    Engine(EngineDevice),
}
