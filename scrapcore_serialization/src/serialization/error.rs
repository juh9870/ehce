use crate::registry::kind::ItemKindProvider;
use crate::registry::SerializationRegistry;
use crate::{AssetName, ItemId};
use slabmap::SlabMapDuplicateError;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use thiserror::Error;

#[cfg(feature = "miette")]
mod diagnostic;

#[derive(Debug, Error, Clone)]
pub enum DeserializationErrorKind<Registry: SerializationRegistry> {
    #[error("Item {}({}) is missing", .1, .0)]
    MissingItem(ItemId, Registry::ItemKind),
    #[error("Item {}({}) is declared twice, in `{}` and `{}`", .kind, .id, .path_a.to_string_lossy(), .path_b.to_string_lossy())]
    DuplicateItem {
        id: ItemId,
        kind: Registry::ItemKind,
        path_a: PathBuf,
        path_b: PathBuf,
    },
    #[error("Item {}({}) is already declared", .1, .0)]
    DuplicateItemLowInfo(ItemId, Registry::ItemKind),
    #[error("Image `{}` is missing", .0)]
    MissingAsset(AssetName, Registry::AssetKind),
    #[error("Asset name `{}` is contested by `{}` and `{}`", .name, .path_a.to_string_lossy(), .path_b.to_string_lossy())]
    DuplicateAsset {
        kind: Registry::AssetKind,
        name: AssetName,
        path_a: PathBuf,
        path_b: PathBuf,
    },
    #[error("Singleton item {} is declared twice, in `{}` and `{}`", .kind, .path_a.to_string_lossy(), .path_b.to_string_lossy())]
    DuplicateSingleton {
        kind: Registry::ItemKind,
        path_a: PathBuf,
        path_b: PathBuf,
    },
    #[error("File at `{}` doesn't have a name", .0.to_string_lossy())]
    MissingName(PathBuf),
    #[error("File path at `{}` is not UTF8", .0.to_string_lossy())]
    NonUtf8Path(PathBuf),
    #[error("Value is too large, got {} where at most {} is expected.", .got, .limit)]
    ValueTooLarge { limit: f64, got: f64 },
    #[error("Value is too small, got {} where at least {} is expected.", .got, .limit)]
    ValueTooSmall { limit: f64, got: f64 },
    #[error("{}", .0)]
    Custom(Registry::Error),
}

impl<Registry: SerializationRegistry> DeserializationErrorKind<Registry> {
    pub fn into_err(self) -> DeserializationError<Registry> {
        self.into()
    }
}

#[derive(Debug, Clone)]
pub enum DeserializationErrorStackItem<Registry: SerializationRegistry> {
    ItemByPath(PathBuf, Registry::ItemKind),
    ItemById(ItemId, Registry::ItemKind),
    Field(&'static str),
    Index(usize),
    MapKey(String),
    MapEntry(String),
    // all JSON keys are strings, so we expect deserialized value to be reasonably displayable
    ExprVariable(String),
}

impl<Registry: SerializationRegistry> Display for DeserializationErrorStackItem<Registry> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializationErrorStackItem::ItemByPath(path, kind) => {
                write!(f, "In item <{kind}> at `{}`", path.to_string_lossy())
            }
            DeserializationErrorStackItem::ItemById(id, kind) => {
                write!(f, "In item <{kind}>`{id}`")
            }
            DeserializationErrorStackItem::Field(name) => write!(f, "In field {name}"),
            DeserializationErrorStackItem::Index(i) => write!(f, "In item at position {i}"),
            DeserializationErrorStackItem::MapEntry(name) => {
                write!(f, "In map entry with key `{name}`")
            }
            DeserializationErrorStackItem::MapKey(name) => {
                write!(f, "In map key `{name}`")
            }
            DeserializationErrorStackItem::ExprVariable(name) => {
                write!(f, "In expression variable `{name}`")
            }
        }
    }
}

#[derive(Debug, Error, Clone)]
#[cfg_attr(feature = "miette", derive(miette::Diagnostic))]
pub struct DeserializationError<Registry: SerializationRegistry> {
    pub kind: DeserializationErrorKind<Registry>,
    pub stack: Vec<DeserializationErrorStackItem<Registry>>,
}

impl<Registry: SerializationRegistry> Display for DeserializationError<Registry> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;
        for item in &self.stack {
            write!(f, "\n{}", item)?;
        }
        Ok(())
    }
}

impl<Registry: SerializationRegistry> DeserializationError<Registry> {
    pub fn context(mut self, item: DeserializationErrorStackItem<Registry>) -> Self {
        self.stack.push(item);
        self
    }
}

impl<Registry: SerializationRegistry> From<DeserializationErrorKind<Registry>>
    for DeserializationError<Registry>
{
    fn from(value: DeserializationErrorKind<Registry>) -> Self {
        DeserializationError {
            kind: value,
            stack: Default::default(),
        }
    }
}

// impl From<ExError> for DeserializationError {
//     fn from(value: ExError) -> Self {
//         DeserializationErrorKind::BadExpression(value).into()
//     }
// }

impl<T, Registry: SerializationRegistry> From<SlabMapDuplicateError<ItemId, T>>
    for DeserializationError<Registry>
where
    Registry: ItemKindProvider<T>,
{
    fn from(SlabMapDuplicateError(id, _): SlabMapDuplicateError<ItemId, T>) -> Self {
        DeserializationErrorKind::DuplicateItemLowInfo(id, Registry::kind()).into()
    }
}
