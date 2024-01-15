use crate::registry::entry::RegistryEntrySerialized;
use crate::registry::{
    AssetsHolder, PartialRegistryHolder, PartialSingletonHolder, SerializationHub,
};
use crate::serialization::error::{
    DeserializationError, DeserializationErrorKind, DeserializationErrorStackItem,
};
use crate::serialization::SerializationFallback;
use std::collections::hash_map::Entry;
use std::path::PathBuf;

pub fn registry_insert<
    Registry: SerializationHub + PartialRegistryHolder<T>,
    T: SerializationFallback,
>(
    registry: &mut Registry,
    path: PathBuf,
    item: RegistryEntrySerialized<T::Fallback>,
) -> Result<(), DeserializationError<Registry>> {
    let raw = registry.get_raw_registry();
    match raw.entry(item.id.clone()) {
        Entry::Occupied(entry) => Err(DeserializationErrorKind::DuplicateItem {
            id: item.id.clone(),
            kind: Registry::kind(),
            path_a: entry.get().1.clone(),
            path_b: path.clone(),
        }
        .into_err()
        .context(DeserializationErrorStackItem::ItemByPath(
            path,
            Registry::kind(),
        ))),
        Entry::Vacant(entry) => {
            entry.insert((item, path));
            Ok(())
        }
    }
}

pub fn singleton_insert<
    Registry: SerializationHub + PartialSingletonHolder<T>,
    T: SerializationFallback,
>(
    registry: &mut Registry,
    path: PathBuf,
    item: T::Fallback,
) -> Result<(), DeserializationError<Registry>> {
    let entry = registry.get_singleton();

    if let Some((path_b, _)) = entry.take() {
        return Err(DeserializationErrorKind::DuplicateSingleton {
            kind: Registry::kind(),
            path_a: path_b,
            path_b: path.clone(),
        }
        .into_err()
        .context(DeserializationErrorStackItem::ItemByPath(
            path,
            Registry::kind(),
        )));
    } else {
        *entry = Some((path, item))
    }

    Ok(())
}

pub fn asset_insert<Registry: SerializationHub + AssetsHolder<T>, T>(
    registry: &mut Registry,
    path: PathBuf,
    item: T,
) -> Result<(), DeserializationError<Registry>> {
    let Some(name) = path.file_name() else {
        return Err(DeserializationErrorKind::MissingName(path).into());
    };

    let Some(name) = name.to_str() else {
        return Err(DeserializationErrorKind::NonUtf8Path(path).into());
    };
    let name = name.to_ascii_lowercase();
    let assets = registry.get_assets_mut();
    match assets.entry(name.clone()) {
        Entry::Occupied(entry) => Err(DeserializationErrorKind::DuplicateAsset {
            kind: Registry::asset_kind(),
            name,
            path_a: entry.get().1.clone(),
            path_b: path,
        }
        .into()),
        Entry::Vacant(entry) => {
            entry.insert((item, path));
            Ok(())
        }
    }
}
