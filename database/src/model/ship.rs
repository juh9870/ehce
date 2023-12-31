use crate::model::ComponentStatsOrId;
use bevy::{asset::Handle, render::texture::Image};
use database_model_macro::database_model;

#[database_model]
#[derive(Debug, Clone)]
pub struct Ship {
    pub sprite: Handle<Image>,
    #[model(min = 0.1, max = 100.0)]
    pub model_scale: f32,
    pub built_in_stats: Option<ComponentStatsOrId>,
}
