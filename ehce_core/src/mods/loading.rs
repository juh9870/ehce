use bevy::asset::{LoadState, LoadedFolder, UntypedAssetId};
use bevy::core::FrameCount;
use bevy::prelude::*;
use database::call_with_all_models;
use miette::Diagnostic;
use rustc_hash::FxHashSet;
use std::path::{Path, PathBuf};
use utils::miette_ext::DiagnosticWrapper;

use database::model::{DatabaseAsset, DatabaseItemKind, ModRegistry, RegistryId};

use crate::mods::{
    ModData, ModHotReloadEvent, ModLoadErrorEvent, ModLoadedEvent, ModState,
    ModUntypedHotReloadEvent, WantLoadModEvent,
};
use crate::{report_error, SimpleStateObjectPlugin};

pub fn load_last_mod(mut evt: EventWriter<WantLoadModEvent>) {
    evt.send(WantLoadModEvent("mod".to_string()));
}

#[derive(Debug)]
pub struct ModLoadingPlugin;

impl Plugin for ModLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SimpleStateObjectPlugin::<_, LoadingStateData>::new(ModState::Loading),
            TypedHotReloadEventsPlugin,
        ))
        .add_systems(Update, mod_load)
        .add_systems(Update, loader.run_if(in_state(ModState::Loading)))
        .add_systems(OnExit(ModState::Loading), clear_hot_reload_events)
        .add_systems(Update, hot_reload.run_if(in_state(ModState::Ready)))
        .add_systems(Update, asset_tracer.before(hot_reload))
        .add_systems(Update, hot_reload_events.after(hot_reload));
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
    images: Res<Assets<Image>>,
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
    let mut db_files = Vec::new();
    let mut db_images = Vec::new();
    for handle in &folder.handles {
        if let Some(item) = database_items.get(handle) {
            let Some(path) = asset_path(&asset_server, handle) else {
                continue;
            };

            db_files.push((path, item));
        } else if images.contains(handle) {
            let Some(path) = asset_path(&asset_server, handle) else {
                continue;
            };
            db_images.push((path, handle.clone_weak().typed::<Image>()));
        }
    }

    match construct_mod(
        path.path().to_path_buf(),
        data.folder_handle.clone(),
        db_files,
        db_images,
    ) {
        Ok(data) => {
            info!("Mod is constructed, sending events");
            switch_evt.send(ModLoadedEvent(data));
        }
        Err(err) => {
            report_error(err.wrap("Failed to load a mod"));
            err_evt.send(ModLoadErrorEvent);
        }
    }
}

pub fn available_mods<'a>(
    folders: impl IntoIterator<Item = impl AsRef<&'a Path>>,
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

fn asset_path(asset_server: &AssetServer, handle: &UntypedHandle) -> Option<PathBuf> {
    let Some(path) = asset_server.get_path(handle.id()) else {
        error!(?handle, id=?handle.id(), "Failed to fetch path for a database item");
        return None;
    };

    Some(path.path().to_path_buf())
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
    _hot_reload_event: EventWriter<InternalHotReloadEvent>,
    _asset: Res<Assets<DatabaseAsset>>,
    asset_server: Res<AssetServer>,
    loaded_mod: ResMut<ModData>,
) {
    enum Action {
        Add,
        Update,
    }
    for evt in evt.read() {
        let (asset_id, _action) = match evt {
            AssetEvent::Added { id } => (id, Action::Add),
            AssetEvent::Modified { id } => (id, Action::Update),
            AssetEvent::Removed { .. } => continue,
            AssetEvent::LoadedWithDependencies { .. } => continue,
        };
        let Some(path) = asset_server.get_path(*asset_id) else {
            continue;
        };
        if !path.path().starts_with(&loaded_mod.mod_path) {
            continue;
        }

        todo!("Full DB reload");
        // let Ok(path) = Utf8PathBuf::from_path_buf(path.path().to_path_buf()) else {
        //     error!(
        //         ?path,
        //         "Asset path contains non-UTF8 symbols, canceling hot-reloading"
        //     );
        //     continue;
        // };
        // let Some(asset) = asset.get(*asset_id) else {
        //     error!(?path, "Failed to fetch updated asset");
        //     continue;
        // };
        // let item = asset.database_item();
        // let id = item.id().clone();
        //
        // match item
        //     .deserialize(&mut loaded_mod.registry)
        //     .with_context(|| format!("While hot reloading item {}", id))
        // {
        //     Err(err) => report_error(err),
        //     Ok((new_id, old)) => {
        //         match (
        //             action,
        //             loaded_mod.assets.get_by_left(&path),
        //             loaded_mod.assets.get_by_right(&new_id).zip(old),
        //         ) {
        //             // New asset is added, but there is already an item with this ID
        //             (Action::Add, _, Some((conflict, old))) => {
        //                 error!(
        //                     item_path = %path,
        //                     conflicting_path = %conflict,
        //                     id = id,
        //                     "Duplicate item, hot reloading canceled"
        //                 );
        //                 loaded_mod.registry.insert(old);
        //             }
        //             // New asset is added, but it was already in a system previously?
        //             // Weird situation, trigger full reload to be sure
        //             (Action::Add, Some(_), _) => {
        //                 todo!("Full DB reload");
        //             }
        //             // New asset is added, resulting in no collisions
        //             (Action::Add, None, None) => {
        //                 info!(id, %path, "Hot reloaded item (new)");
        //                 // New item, assets update is required
        //                 loaded_mod.assets.insert(path, new_id);
        //                 hot_reload_event.send(InternalHotReloadEvent::Single(new_id));
        //             }
        //             // Asset is updated, keeping the same ID and only conflicting with itself
        //             (Action::Update, Some(old_id), Some((conflict, _)))
        //                 if old_id == &new_id && conflict == &path =>
        //             {
        //                 info!(id, %path, "Hot reloaded item (updated)");
        //                 hot_reload_event.send(InternalHotReloadEvent::Single(new_id));
        //             }
        //             // Asset is updated, but ID got changed, trigger full reload
        //             (Action::Update, Some(_), _) => {
        //                 todo!("Full DB reload");
        //             }
        //             // Asset is updated, but no matching asset is already in a system?
        //             // Weird situation, trigger full reload to be sure
        //             (Action::Update, None, _) => {
        //                 todo!("Full DB reload");
        //             }
        //         }
        //     }
        // }
    }
}

macro_rules! typed_events {
    ($($name:ident: $ty:ty),*$(,)?) => {
        #[derive(Debug)]
        struct TypedHotReloadEventsPlugin;

        impl Plugin for TypedHotReloadEventsPlugin {
            fn build(&self, app: &mut App) {
                app.init_resource::<Events<InternalHotReloadEvent>>();
                app.init_resource::<Events<ModUntypedHotReloadEvent>>();
                $(app.init_resource::<Events<ModHotReloadEvent<$ty>>>();)*
            }
        }

        fn hot_reload_events(
            mut evt: EventReader<InternalHotReloadEvent>,
            mut untyped_event: EventWriter<ModUntypedHotReloadEvent>,
            $(mut $name: EventWriter<ModHotReloadEvent<$ty>>,)*
        ) {
            for evt in evt.read() {
                match evt {
                    InternalHotReloadEvent::Full => {
                        untyped_event.send(ModUntypedHotReloadEvent::Full);
                        $($name.send(ModHotReloadEvent::Full);)*
                    }
                    InternalHotReloadEvent::Single(id) => {
                        untyped_event.send(ModUntypedHotReloadEvent::Single(*id));
                        paste::paste! {
                            match id.kind() {
                                $(
                                    DatabaseItemKind::[<$name:camel>] => {
                                        $name.send(ModHotReloadEvent::Single(id.id().as_typed_unchecked()));
                                    }
                                )*
                            }
                        }
                    }
                }
            }
        }

        fn clear_hot_reload_events(
            mut untyped_event: ResMut<Events<ModUntypedHotReloadEvent>>,
            $(mut $name: ResMut<Events<ModHotReloadEvent<$ty>>>,)*
        ) {
            untyped_event.clear();
            $($name.clear();)*
        }
    };
}

call_with_all_models!(typed_events);

#[derive(Debug, Event)]
pub enum InternalHotReloadEvent {
    Full,
    Single(RegistryId),
}

fn construct_mod<'a, 'path>(
    mod_path: PathBuf,
    folder_handle: Handle<LoadedFolder>,
    files: impl IntoIterator<Item = (impl AsRef<Path>, &'a DatabaseAsset)>,
    images: impl IntoIterator<Item = (impl AsRef<Path>, Handle<Image>)>,
) -> Result<ModData, impl Diagnostic + 'static> {
    let registry = match ModRegistry::build(files, images) {
        Ok(data) => data,
        Err(err) => {
            return Err(err.diagnostic());
        }
    };

    // let mut asset_paths: FxBiHashMap<Utf8PathBuf, RegistryId> = Default::default();
    // for (path, asset) in files {
    //     let item = asset.database_item();
    //     let display_id = item.id().to_string();
    //     let (id, old) = item.deserialize(&mut registry)?;
    //     if old.is_some() {
    //         let Some(old_path) = asset_paths.get_by_right(&id) else {
    //             error!(path=path.to_string(),
    //                 id=display_id,
    //                 raw_id=?id,
    //                 "Conflicting mod items detected, \
    //                 but conflicting asset path was not found. What's going on?");
    //             continue;
    //         };
    //         error!(
    //             first_item = old_path.to_string(),
    //             second_item = path.to_string(),
    //             id=display_id,
    //             raw_id=?id,
    //             "Conflicting mod items detected"
    //         )
    //     }
    //     asset_paths.insert(path, id);
    // }
    Ok(ModData {
        registry,
        mod_path,
        folder_handle,
        // assets: asset_paths,
    })
}
