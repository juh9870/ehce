use crate::resources::Resources;
use bevy::math::Vec3;
use bevy::prelude::{Bundle, Sprite, SpriteBundle, Transform, Vec2};
use bevy_xpbd_2d::prelude::{Collider, RigidBody};
use ehce_core::database::model::ship::Ship;
use ehce_core::database::model::ship_build::ShipBuildData;
use ehce_core::mods::ModData;

pub fn make_build(data: impl AsRef<ShipBuildData>, db: ModData) {
    let data = data.as_ref();

    let _ship = &db.registry.ship[data.ship];
}

pub fn make_ship(data: &Ship, _db: ModData) -> ShipBundle {
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
        collider: Collider::ball(1.0),
        resources: Default::default(),
    }
}

#[derive(Bundle)]
pub struct ShipBundle {
    sprite: SpriteBundle,
    rb: RigidBody,
    collider: Collider,
    resources: Resources,
}
