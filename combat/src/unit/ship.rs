use crate::resources::{ResourceEvaluationError, Resources};

use bevy::math::Vec3;
use bevy::prelude::{Assets, Bundle, Image, Sprite, SpriteBundle, Transform, Vec2};
use bevy_xpbd_2d::prelude::{Collider, RigidBody};
use ehce_core::database::model::ship::Ship;
use ehce_core::database::model::ship_build::ShipBuildData;
use ehce_core::mods::ModData;

pub fn calculate_resources(
    db: &ModData,
    ship: &Ship,
    build: impl AsRef<ShipBuildData>,
) -> Result<Resources, ResourceEvaluationError> {
    let build = build.as_ref();

    let stats = build
        .components
        .iter()
        .map(|e| &db.registry[e.component].stats)
        .chain(ship.built_in_stats.iter())
        .flat_map(|id| &db.registry[id].stats)
        .map(|(id, value)| (*id, *value));
    Resources::from_stats(db, stats)
}

pub fn make_ship(_db: &ModData, data: &Ship, image_assets: &Assets<Image>) -> ShipBundle {
    let collider = if let Some(image) = image_assets.get(&data.sprite) {
        collider_generator::compute_collider_for_texture(image, 0.01)
    } else {
        Collider::ball(data.model_scale / 2.0)
    };
    ShipBundle {
        sprite: SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(1.0)),
                ..Default::default()
            },
            transform: Transform {
                scale: Vec3::splat(data.model_scale),
                ..Default::default()
            },
            texture: data.sprite.clone_weak(),
            ..Default::default()
        },
        rb: RigidBody::Dynamic,
        collider,
    }
}

#[derive(Bundle)]
pub struct ShipBundle {
    pub sprite: SpriteBundle,
    pub rb: RigidBody,
    pub collider: Collider,
}