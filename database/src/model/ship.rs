use bevy::{asset::Handle, render::texture::Image};
use database_model_macro::database_model;
use utils::slab_map::SlabMapId;

#[database_model]
#[derive(Debug, Clone)]
pub struct Ship {
    #[model(id)]
    pub id: SlabMapId<Ship>,
    pub sprite: Handle<Image>,
    #[model(min = 0.1, max = 100.0)]
    pub model_scale: f32,
}
