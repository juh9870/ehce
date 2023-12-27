use crate::variables::{VariableEvaluationError, Variables};

use bevy::math::Vec3;
use bevy::prelude::{Assets, Bundle, Image, Sprite, SpriteBundle, Transform, Vec2};
use bevy_xpbd_2d::prelude::{Collider, RigidBody};
use ehce_core::database::model::ship::Ship;
use ehce_core::database::model::ship_build::ShipBuild;
use ehce_core::mods::ModData;

pub fn calculate_variables(
    db: &ModData,
    ship: impl AsRef<Ship>,
    build: impl AsRef<ShipBuild>,
) -> Result<Variables, VariableEvaluationError> {
    let build = build.as_ref();

    let ship = ship.as_ref();

    let stats = build
        .components
        .iter()
        .map(|e| &db.registry[e.component].data.stats)
        .chain(ship.built_in_stats.iter())
        .flat_map(|id| &id.get(&db.registry).stats)
        .map(|(id, value)| (*id, *value));
    Variables::from_stats(db, stats)
}

pub fn make_ship(
    _db: &ModData,
    data: impl AsRef<Ship>,
    image_assets: &Assets<Image>,
) -> ShipBundle {
    let data = data.as_ref();
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
