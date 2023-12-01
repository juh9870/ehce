use bevy::app::{App, Plugin};
use bevy::prelude::*;

use crate::glue::combat::CombatInit;
use crate::mods::loading::load_last_mod;
use crate::mods::{ModLoadErrorEvent, ModLoadedEvent, ModState};
use crate::GameState;

#[derive(Debug)]
pub struct InitPlugin;

impl Plugin for InitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Init), load_last_mod)
            .add_systems(PostUpdate, init_tick.run_if(in_state(GameState::Init)));
    }
}

fn init_tick(
    mut errors: EventReader<ModLoadErrorEvent>,
    mut loaded: ResMut<Events<ModLoadedEvent>>,
    mut state: ResMut<NextState<GameState>>,
    mut mod_state: ResMut<NextState<ModState>>,
    mut commands: Commands,
) {
    if errors.read().next().is_some() {
        state.set(GameState::Error);
        mod_state.set(ModState::None);
        warn!("Got a mod loading error during initialization, switching to error state");
        return;
    }

    if let Some(data) = loaded.drain().last() {
        info!("Mod is loaded, switching to combat state");
        let mod_data = data.0;

        let fleet = mod_data
            .registry
            .fleet
            .values()
            .next()
            .expect("Should have at least one fleet")
            .data
            .clone();
        commands.insert_resource(CombatInit {
            player_fleet: fleet.clone(),
            enemy_fleet: fleet,
            combat_settings: mod_data
                .registry
                .combat_settings
                .values()
                .next()
                .expect("Should have at least one Combat settings")
                .data
                .clone(),
        });
        commands.insert_resource(mod_data);
        mod_state.set(ModState::Ready);
        state.set(GameState::Combat);
    }
}
