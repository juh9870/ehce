use bevy::app::{App, Plugin};
use bevy::asset::{Handle, LoadedFolder};
use bevy::prelude::{Event, Resource, States};
use camino::Utf8PathBuf;
use database::model::{ModRegistry, RegistryId};

use utils::slab_map::SlabMapId;
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

/// Event that triggers loading of a new mod
///
/// Should generally be only raised by app code, but not listened to
#[derive(Debug, Event)]
pub struct WantLoadModEvent(String);

/// Event that is triggered when mod loading fails for any reason
///
/// This event should not be raised outside of mod loading code
///
/// Errors are logged via error!, so use custom tracing frontend to report
/// errors to the user
#[derive(Debug, Event)]
pub struct ModLoadErrorEvent;

/// Event that is triggered when mod is loaded successfully
///
/// At any point in an app lifecycle, there should only be one system listening
/// for this event, and it should drain this event as soon as possible
///
/// Payload is a full mod data
#[derive(Debug, Event)]
pub struct ModLoadedEvent(pub ModData);

/// Event that is triggered when hot reload happens
///
/// For most use cases, [ModHotReloadEvent] is more ergonomic
#[derive(Debug, Event)]
pub enum ModUntypedHotReloadEvent {
    Full,
    /// Payload is the registry ID of a changed asset
    Single(RegistryId),
}

/// Event that is triggered when hot reload happens
#[derive(Debug, Event)]
pub enum ModHotReloadEvent<T> {
    Full,
    /// Payload is the registry ID of a changed asset
    Single(SlabMapId<T>),
}
