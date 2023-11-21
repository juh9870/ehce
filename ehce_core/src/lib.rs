use crate::init::InitPlugin;
use crate::json5_asset_plugin::Json5AssetPlugin;
use crate::mods::ModPlugin;
use bevy::app::App;
use bevy::ecs::prelude::States;
use bevy::prelude::*;

use database::model::DatabaseAsset;
use std::marker::PhantomData;

// Re-export database
pub use database;

mod init;
mod json5_asset_plugin;
pub mod mods;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameState {
    /// Critical unrecoverable error state
    Error,
    /// Application initialization state
    #[default]
    Init,
    /// Combat state
    Combat,
}

impl States for GameState {}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>().add_plugins((
            Json5AssetPlugin::<DatabaseAsset>::new(&["json", "json5"]),
            InitPlugin,
            ModPlugin,
        ));
    }
}

struct SimpleStateObjectPlugin<S: States + Clone, T: Resource>(S, PhantomData<T>);

impl<S: States + Clone, T: Resource> SimpleStateObjectPlugin<S, T> {
    pub fn new(state: S) -> Self {
        Self(state, Default::default())
    }
}

impl<S: States + Clone, T: Resource> Plugin for SimpleStateObjectPlugin<S, T> {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(self.0.clone()), assert_state_object::<T>)
            .add_systems(OnExit(self.0.clone()), cleanup_state_object::<T>);
    }
}

pub fn assert_state_object<T: Resource>(res: Option<Res<T>>, state: Res<State<GameState>>) {
    if res.is_none() {
        error!(
            ?state,
            "State object is missing after transitioning to a state"
        )
    }
}

pub fn cleanup_state_object<T: Resource>(mut commands: Commands) {
    commands.remove_resource::<T>();
}

pub fn report_error(err: impl Into<miette::Report>) {
    error!("Something gone wrong.\n{:?}", err.into())
}
