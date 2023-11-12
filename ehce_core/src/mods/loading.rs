use bevy::asset::{AssetPath, LoadState, LoadedFolder, UntypedAssetId};
use bevy::core::FrameCount;

use bevy::prelude::*;
use camino::{Utf8Path, Utf8PathBuf};
use miette::{IntoDiagnostic, WrapErr};
use rustc_hash::FxHashSet;

use utils::slab_map::SlabMapUntypedId;
use utils::FxBiHashMap;

use crate::mods::model::{DatabaseAsset, DatabaseItemTrait, ModRegistry};
use crate::mods::{ModData, ModLoadErrorEvent, ModLoadedEvent, ModState, WantLoadModEvent};
use crate::{report_error, SimpleStateObjectPlugin};

pub fn load_last_mod(mut evt: EventWriter<WantLoadModEvent>) {
    evt.send(WantLoadModEvent("mod".to_string()));
}

#[derive(Debug)]
pub struct ModLoadingPlugin;

impl Plugin for ModLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SimpleStateObjectPlugin::<_, LoadingStateData>::new(
            ModState::Loading,
        ))
        .add_systems(Update, mod_load)
        .add_systems(Update, loader.run_if(in_state(ModState::Loading)))
        .add_systems(Update, hot_reload.run_if(in_state(ModState::Ready)))
        .add_systems(Update, asset_tracer.before(hot_reload));
    }
}

#[derive(Debug, Default, Resource)]
struct LoadingStateData {
    folder_handle: Handle<LoadedFolder>,
    not_ready_handles: Option<FxHashSet<UntypedAssetId>>,
}

// If multiple mod load events are passed in a frame, only the last one is handled
fn mod_load(
    mut evt: EventReader<WantLoadModEvent>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<ModState>>,
) {
    let Some(evt) = evt.read().last() else { return };
    let mod_folder = asset_server.load_folder(&evt.0);
    commands.insert_resource(LoadingStateData {
        folder_handle: mod_folder,
        not_ready_handles: None,
    });
    next_state.set(ModState::Loading)
}

fn loader(
    asset_server: Res<AssetServer>,
    folder_assets: Res<Assets<LoadedFolder>>,
    database_items: Res<Assets<DatabaseAsset>>,
    mut db_asset_events: ResMut<Events<AssetEvent<DatabaseAsset>>>,
    mut data: ResMut<LoadingStateData>,
    mut err_evt: EventWriter<ModLoadErrorEvent>,
    mut switch_evt: EventWriter<ModLoadedEvent>,
    frame: Res<FrameCount>,
    mut wait_until: Local<Option<u32>>,
) {
    match asset_server.load_state(&data.folder_handle) {
        LoadState::NotLoaded => {
            error!("Mod folder appears to be missing from asset server");
            err_evt.send(ModLoadErrorEvent);
            return;
        }
        LoadState::Failed => {
            err_evt.send(ModLoadErrorEvent);
            return;
        }
        _ => {}
    }
    let Some(folder) = folder_assets.get(&data.folder_handle) else {
        return;
    };

    let handles = data
        .not_ready_handles
        .get_or_insert_with(|| folder.handles.iter().map(|e| e.id()).collect());

    let mut errors = Vec::new();
    handles.retain(|e| match asset_server.load_state(*e) {
        LoadState::Loaded => false,
        LoadState::Failed => {
            asset_server.get_path(*e);
            errors.push(*e);
            true
        }
        _ => true,
    });

    if !errors.is_empty() {
        err_evt.send(ModLoadErrorEvent);
        return;
    }

    if !handles.is_empty() {
        return;
    }

    let wait_until = wait_until.get_or_insert(frame.0 + 1);

    if frame.0 < *wait_until {
        return;
    }

    // Clear all pending asset events to avoid hot reloading all currently loaded files
    db_asset_events.clear();

    let Some(path) = asset_server.get_path(&data.folder_handle) else {
        error!("Mod folder is missing asset path");
        err_evt.send(ModLoadErrorEvent);
        return;
    };

    info!("Mod assets are loaded");
    let mut files = Vec::new();
    for handle in &folder.handles {
        let Some(item) = database_items.get(handle) else {
            continue;
        };

        let Some(path) = asset_server.get_path(handle.id()) else {
            error!(?handle, id=?handle.id(), "Failed to fetch path for a database item");
            continue;
        };

        let path = path.path();
        let Ok(path) = Utf8PathBuf::try_from(path.to_path_buf()) else {
            error!(path=path.to_string_lossy().to_string(), raw_path=?path, ?handle, id=?handle.id(), "Asset path contains non-UTF8 symbols");
            continue;
        };

        files.push((path, item));
    }
    match construct_mod(path, data.folder_handle.clone(), files) {
        Ok(data) => {
            info!("Mod is constructed, sending events");
            switch_evt.send(ModLoadedEvent(data));
        }
        Err(err) => report_error(err.context("While loading a mod")),
    }
}

pub fn available_mods(
    folders: impl IntoIterator<Item = impl AsRef<Utf8Path>>,
) -> impl Iterator<Item = String> {
    folders
        .into_iter()
        .filter_map(|e| std::fs::read_dir(e.as_ref()).ok())
        .flat_map(|e| {
            e.filter_map(|e| {
                e.ok().and_then(|e| {
                    e.path()
                        .file_name()
                        .and_then(|e| e.to_str().map(|e| e.to_string()))
                })
            })
        })
}

fn asset_tracer(
    mut folder_evt: EventReader<AssetEvent<LoadedFolder>>,
    mut asset_evt: EventReader<AssetEvent<DatabaseAsset>>,
    frame: Res<FrameCount>,
) {
    for evt in folder_evt.read() {
        info!(frame = frame.0, ?evt, "Folder event")
    }
    for evt in asset_evt.read() {
        info!(frame = frame.0, ?evt, "Asset event")
    }
}

fn hot_reload(
    mut evt: EventReader<AssetEvent<DatabaseAsset>>,
    asset: Res<Assets<DatabaseAsset>>,
    asset_server: Res<AssetServer>,
    mut loaded_mod: ResMut<ModData>,
) {
    enum Action {
        Add,
        Update,
    }
    for evt in evt.read() {
        let (id, _action) = match evt {
            AssetEvent::Added { id } => (id, Action::Add),
            AssetEvent::Modified { id } => (id, Action::Update),
            AssetEvent::Removed { .. } => continue,
            AssetEvent::LoadedWithDependencies { .. } => continue,
        };
        let Some(path) = asset_server.get_path(*id) else {
            continue;
        };
        if !path.path().starts_with(&loaded_mod.mod_path) {
            continue;
        }
        let Some(asset) = asset.get(*id) else {
            error!(?path, "Failed to fetch updated asset");
            continue;
        };
        let item = asset.database_item();
        let id = item.id().clone();

        item.deserialize(&mut loaded_mod.registry)
            .map(|_| {
                info!("Hot reloaded item {}", id);
            })
            .with_context(|| format!("While hot reloading item {}", id))
            .unwrap_or_else(report_error);
    }
}

fn construct_mod<'a>(
    mod_path: AssetPath,
    folder_handle: Handle<LoadedFolder>,
    files: impl IntoIterator<Item = (Utf8PathBuf, &'a DatabaseAsset)>,
) -> miette::Result<ModData> {
    let mut registry = ModRegistry::default();
    let mut asset_paths: FxBiHashMap<Utf8PathBuf, SlabMapUntypedId> = Default::default();
    for (path, asset) in files {
        let item = asset.database_item();
        let display_id = item.id().to_string();
        let (id, old) = item.deserialize(&mut registry)?;
        if old.is_some() {
            let Some(old_path) = asset_paths.get_by_right(&id) else {
                error!(path=path.to_string(),
                    id=display_id,
                    raw_id=?id,
                    "Conflicting mod items detected, \
                    but conflicting asset path was not found. What's going on?");
                continue;
            };
            error!(
                first_item = old_path.to_string(),
                second_item = path.to_string(),
                id=display_id,
                raw_id=?id,
                "Conflicting mod items detected"
            )
        }
        asset_paths.insert(path, id);
    }
    Ok(ModData {
        registry,
        mod_path: Utf8PathBuf::try_from(mod_path.path().to_path_buf()).into_diagnostic()?,
        folder_handle,
        assets: asset_paths,
    })
}
