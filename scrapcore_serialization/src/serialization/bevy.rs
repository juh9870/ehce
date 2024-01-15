use crate::registry::{AssetKindProvider, AssetsHolder, SerializationRegistry};
use crate::serialization::error::{DeserializationError, DeserializationErrorKind};
use crate::serialization::{DeserializeModel, SerializationFallback};
use crate::{AssetName, AssetNameRef};
use bevy_asset::{Asset, Handle};

/// Deserialization for bevy asset handler fields
///
/// Fields are populated with WEAK handles to the asset
/// Currently there is no way to request a strong handle
impl<'a, Registry: SerializationRegistry, A: Asset> DeserializeModel<Handle<A>, Registry>
    for AssetNameRef<'a>
where
    Registry: AssetsHolder<Handle<A>> + AssetKindProvider<Handle<A>>,
{
    fn deserialize(
        self,
        registry: &mut Registry,
    ) -> Result<Handle<A>, DeserializationError<Registry>> {
        let name = self.to_ascii_lowercase();
        if let Some(handle) = registry.get_assets().get(&name) {
            Ok(handle.clone_weak())
        } else {
            Err(DeserializationErrorKind::MissingAsset(name, Registry::asset_kind()).into())
        }
    }
}

impl<A: Asset> SerializationFallback for Handle<A> {
    type Fallback = AssetName;
}
