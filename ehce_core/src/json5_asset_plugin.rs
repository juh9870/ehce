use bevy::app::{App, Plugin};
use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetApp, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext};
use std::marker::PhantomData;
use thiserror::Error;
use tracing::error;

/// Plugin to load your asset type `A` from json files.
pub struct Json5AssetPlugin<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>,
}

impl<A> Plugin for Json5AssetPlugin<A>
where
    for<'de> A: serde::Deserialize<'de> + serde::Serialize + Asset,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<A>()
            .register_asset_loader(JsonAssetLoader::<A> {
                extensions: self.extensions.clone(),
                _marker: PhantomData,
            });
    }
}

impl<A> Json5AssetPlugin<A>
where
    for<'de> A: serde::Deserialize<'de> + Asset,
{
    /// Create a new plugin that will load assets from files with the given extensions.
    pub fn new(extensions: &[&'static str]) -> Self {
        Self {
            extensions: extensions.to_owned(),
            _marker: PhantomData,
        }
    }
}

struct JsonAssetLoader<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>,
}

/// Possible errors that can be produced by [`JsonAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum JsonLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// A [JSON Error](serde_json::error::Error)
    #[error("Could not parse the JSON: {0}")]
    JsonError(#[from] serde_json5::Error),
}

impl<A> AssetLoader for JsonAssetLoader<A>
where
    for<'de> A: serde::Deserialize<'de> + serde::Serialize + Asset,
{
    type Asset = A;
    type Settings = ();
    type Error = JsonLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            match serde_json5::from_slice::<A>(&bytes) {
                Ok(data) => Ok(data),
                Err(err) => {
                    error!("Failed to load {}. {}", load_context.asset_path(), err);
                    Err(err.into())
                }
            }
        })
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}
