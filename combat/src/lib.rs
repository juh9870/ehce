use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;
use bytemuck::cast;
use euclid::{Length, Vector2D};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ehce_core::GameState::Combat), init_combat)
            .add_systems(OnExit(ehce_core::GameState::Combat), exit_combat)
            .add_systems(Update, update)
            .add_plugins(Shape2dPlugin::default());
    }
}

#[derive(Debug, Reflect, Resource)]
struct CombatData {
    camera: Entity,
}

fn init_combat(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera = commands.spawn(Camera2dBundle::default()).id();
    let data = CombatData { camera };

    let texture = asset_server.load("icon.png");

    let mut left = -480.0;
    for i in 1..=15 {
        let transform = Transform {
            translation: Vec3::new(left, 0.0, 0.0),
            scale: Vec3::splat(i as f32 * 10.0),
            ..Default::default()
        };
        commands.spawn(SpriteBundle {
            texture: texture.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(1.0)),
                ..Default::default()
            },
            transform,
            ..Default::default()
        });
        left += i as f32 * 10.0 + 5.0
    }
    commands.insert_resource(data);
}

fn exit_combat(mut commands: Commands, data: Res<CombatData>) {
    commands.entity(data.camera).despawn_recursive()
}

#[inline(always)]
fn as_glam(vec: ScreenVector) -> Vec2 {
    cast(vec)
}

fn update(query: Query<&mut Transform, With<Sprite>>, _painter: ShapePainter) {
    query.for_each(|_e| {
        // painter.transform = *e;
        // painter.hollow = true;
        // painter.color = Color::RED;
        // painter.thickness = 0.001;
        // painter.rect(Vec2::ONE);
    });
}

#[derive(Debug)]
struct WorldSpace;

type WorldLength = Length<f32, WorldSpace>;
type WorldVector = Vector2D<f32, WorldSpace>;

#[derive(Debug)]
pub struct ScreenSpace;
type ScreenLength = Length<f32, ScreenSpace>;
type ScreenVector = Vector2D<f32, ScreenSpace>;
