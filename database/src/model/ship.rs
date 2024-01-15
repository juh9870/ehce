use crate::model::{ComponentStatsOrId, DeviceOrId};
use bevy::asset::Handle;
use bevy::prelude::Image;
use database_model_macro::database_model;

#[database_model]
#[derive(Debug, Clone)]
pub struct Ship {
    pub sprite: Handle<Image>,
    #[model(min = 0.1, max = 100.0)]
    pub model_scale: f32,
    pub built_in_stats: Option<ComponentStatsOrId>,
    pub built_in_devices: Option<Vec<DeviceOrId>>,
}
