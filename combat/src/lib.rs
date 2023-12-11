use crate::spawning::ship_spawn;
use crate::unit::Team;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::SystemParam;
use bevy::log::Level;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::utils::tracing::Callsite;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_sysfail::Failure;
use bevy_vector_shapes::prelude::*;
use bevy_xpbd_2d::plugins::{PhysicsDebugPlugin, PhysicsPlugins};
use bevy_xpbd_2d::prelude::PhysicsDebugConfig;
use bevy_xpbd_2d::resources::Gravity;
use ehce_core::database::model::combat_settings::CombatSettingsData;

use ehce_core::glue::combat::CombatInit;
use ehce_core::mods::HotReloading;
use ehce_core::GameState;

use fleet::CombatFleet;
use miette::{Diagnostic, Report};

mod fleet;
mod resources;
mod spawning;
mod state;
mod unit;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum CombatSet {
    PreUpdate,
    PhysicsUpdate,
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
                CombatSet::PhysicsUpdate,
                CombatSet::Update,
                CombatSet::PostUpdate,
            )
                .chain()
                .after(HotReloading)
                .run_if(in_state(GameState::Combat)),
        );

        app.init_resource::<Events<CombatErrorEvent>>();

        app.add_systems(OnEnter(GameState::Combat), init_combat)
            .add_systems(OnExit(GameState::Combat), exit_combat)
            .add_plugins(Shape2dPlugin::default());

        app.add_systems(FixedUpdate, ship_spawn.in_set(CombatSet::PreUpdate));

        app.add_plugins((
            PhysicsPlugins::new(PhysicsUpdate),
            PhysicsDebugPlugin::default(),
        ));
        app.add_systems(FixedUpdate, run_physics.in_set(CombatSet::PhysicsUpdate));

        app.add_systems(Update, (update).run_if(in_state(GameState::Combat)));
        app.add_systems(Last, (error_handler).run_if(in_state(GameState::Combat)));

        app.add_plugins(WorldInspectorPlugin::new());
    }
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PhysicsUpdate;

fn run_physics(world: &mut World) {
    world.run_schedule(PhysicsUpdate)
}

#[derive(Debug, Resource)]
struct CombatData {
    combat_settings: CombatSettingsData,
    player_team: Team,
}

fn init_combat(world: &mut World) {
    world.init_resource::<PhysicsDebugConfig>();
    let mut physics_debug = world.get_resource_or_insert_with(PhysicsDebugConfig::default);
    physics_debug.enabled = true;
    physics_debug.axis_lengths = None;

    let mut gravity = world.get_resource_or_insert_with(Gravity::default);
    gravity.0 = Vec2::ZERO;

    let combat_init = world
        .remove_resource::<CombatInit>()
        .unwrap_or_else(|| todo!("Gracefully handle state switch error"));
    let player_team =
        Team::new_unchecked_do_not_use_directly_its_bad_really_will_be_very_hard_to_migrate_later(
            0,
        );
    let _enemy_team =
        Team::new_unchecked_do_not_use_directly_its_bad_really_will_be_very_hard_to_migrate_later(
            1,
        );
    world.insert_resource(CombatData {
        combat_settings: combat_init.combat_settings,
        player_team,
    });
    world.spawn((player_team, CombatFleet::from(&combat_init.player_fleet)));
    // world.spawn((enemy_team, CombatFleet::from(&combat_init.enemy_fleet)));
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

fn update(_query: Query<&mut Transform, With<Sprite>>, mut _painter: ShapePainter) {
    // query.for_each(|e| {
    //     painter.transform = *e;
    //     painter.hollow = true;
    //     painter.color = Color::RED;
    //     painter.thickness = 0.001;
    //     painter.rect(Vec2::ONE);
    // });
}

pub struct EmitCombatError(pub Report);

impl<T: Diagnostic + Send + Sync + 'static> From<T> for EmitCombatError {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl Failure for EmitCombatError {
    type Param = EventWriter<'static, CombatErrorEvent>;
    const LEVEL: Level = Level::ERROR;

    fn handle_error(
        self,
        mut param: <Self::Param as SystemParam>::Item<'_, '_>,
        _: Option<&'static impl Callsite>,
    ) {
        param.send(CombatErrorEvent(self.0))
    }
}

#[derive(Debug, Event)]
pub struct CombatErrorEvent(Report);

fn error_handler(mut errors: ResMut<Events<CombatErrorEvent>>) {
    if errors.is_empty() {
        return;
    }
    for event in errors.drain() {
        error!("{:?}", event.0);
    }

    std::process::exit(1);
}
