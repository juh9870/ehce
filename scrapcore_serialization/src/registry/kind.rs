use crate::registry::entry::RegistryEntry;
use crate::registry::SerializationHub;

pub trait ItemKindProvider<Item>: SerializationHub {
    fn kind() -> Self::ItemKind;
}

impl<Registry: ItemKindProvider<T>, T> ItemKindProvider<Option<T>> for Registry {
    fn kind() -> Self::ItemKind {
        <Registry as ItemKindProvider<T>>::kind()
    }
}

impl<Registry: ItemKindProvider<T>, T> ItemKindProvider<RegistryEntry<T>> for Registry {
    fn kind() -> Self::ItemKind {
        <Registry as ItemKindProvider<T>>::kind()
    }
}

pub trait AssetKindProvider<Asset>: SerializationHub {
    fn asset_kind() -> Self::AssetKind;
}

impl<Registry: AssetKindProvider<T>, T> AssetKindProvider<Option<T>> for Registry {
    fn asset_kind() -> Self::AssetKind {
        <Registry as AssetKindProvider<T>>::asset_kind()
    }
}
