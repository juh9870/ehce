use crate::spawning::ship_spawn;
use crate::unit::Team;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_vector_shapes::prelude::*;
use ehce_core::database::model::combat_settings::CombatSettingsData;

use ehce_core::glue::combat::CombatInit;
use ehce_core::mods::HotReloading;
use ehce_core::GameState;
use extension_trait::extension_trait;
use fleet::CombatFleet;
use miette::Report;

mod fleet;
mod resources;
mod spawning;
mod state;
mod unit;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum CombatSet {
    PreUpdate,
    Update,
    PostUpdate,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            (
                CombatSet::PreUpdate,
                CombatSet::Update,
                CombatSet::PostUpdate,
            )
                .chain()
                .after(HotReloading)
                .run_if(in_state(GameState::Combat)),
        );
        app.add_systems(OnEnter(GameState::Combat), init_combat)
            .add_systems(OnExit(GameState::Combat), exit_combat)
            .add_plugins(Shape2dPlugin::default());

        app.add_systems(FixedUpdate, ship_spawn.in_set(CombatSet::PreUpdate));
        app.add_systems(Update, (update).run_if(in_state(GameState::Combat)));

        app.add_plugins(WorldInspectorPlugin::new());
    }
}

#[derive(Debug, Resource)]
struct CombatData {
    combat_settings: CombatSettingsData,
    player_team: Team,
}

fn init_combat(world: &mut World) {
    let combat_init = world
        .remove_resource::<CombatInit>()
        .unwrap_or_else(|| todo!("Gracefully handle state switch error"));
    let player_team =
        Team::new_unchecked_do_not_use_directly_its_bad_really_will_be_very_hard_to_migrate_later(
            0,
        );
    let enemy_team =
        Team::new_unchecked_do_not_use_directly_its_bad_really_will_be_very_hard_to_migrate_later(
            1,
        );
    world.insert_resource(CombatData {
        combat_settings: combat_init.combat_settings,
        player_team,
    });
    world.spawn((player_team, CombatFleet::from(&combat_init.player_fleet)));
    world.spawn((enemy_team, CombatFleet::from(&combat_init.enemy_fleet)));
    world.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            near: -1e9,
            far: 1e9,
            scaling_mode: ScalingMode::WindowSize(64.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn exit_combat(_commands: Commands, _data: Res<CombatData>) {}

fn update(query: Query<&mut Transform, With<Sprite>>, mut painter: ShapePainter) {
    query.for_each(|e| {
        painter.transform = *e;
        painter.hollow = true;
        painter.color = Color::RED;
        painter.thickness = 0.001;
        painter.rect(Vec2::ONE);
    });
}

#[extension_trait]
pub impl<T, E: Into<Report>> ResultExt<T> for Result<T, E> {
    fn sys_fail(self) -> T {
        match self {
            Ok(data) => data,
            Err(err) => {
                error!("Something gone wrong.\n{:?}", err.into());
                std::process::exit(1);
            }
        }
    }
}
