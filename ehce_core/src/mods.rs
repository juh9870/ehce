use bevy::app::{App, Plugin};
use bevy::asset::{Handle, LoadedFolder};
use bevy::prelude::{Event, Resource, States};
use camino::Utf8PathBuf;
use database::model::{ModRegistry, RegistryId};

use utils::FxBiHashMap;

use crate::mods::loading::ModLoadingPlugin;

pub mod loading;

#[derive(Debug)]
pub struct ModPlugin;

impl Plugin for ModPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<ModState>()
            .add_event::<WantLoadModEvent>()
            .add_event::<ModLoadErrorEvent>()
            .add_event::<ModLoadedEvent>()
            .add_plugins(ModLoadingPlugin);
    }
}

#[derive(Debug, Resource)]
pub struct ModData {
    pub registry: ModRegistry,
    pub mod_path: Utf8PathBuf,
    pub folder_handle: Handle<LoadedFolder>,
    pub assets: FxBiHashMap<Utf8PathBuf, RegistryId>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub enum ModState {
    /// Default state, signifying that no mod is loaded
    #[default]
    None,
    /// State signifying that a mod is loading
    Loading,
    /// State signifying a ready to use mod data
    Ready,
}

impl States for ModState {}

#[derive(Debug, Event)]
pub struct WantLoadModEvent(String);

#[derive(Debug, Event)]
pub struct ModLoadErrorEvent;

#[derive(Debug, Event)]
pub struct ModLoadedEvent(pub ModData);
