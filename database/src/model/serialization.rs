use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{BuildHasher, Hash};
use std::path::PathBuf;
use std::sync::Arc;

use bevy::asset::Handle;

use duplicate::{duplicate, duplicate_item};
use exmex::ExError;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use utils::slab_map::{SlabMap, SlabMapDuplicateError, SlabMapId};

use crate::model::{DatabaseItemKind, DatabaseItemTrait, ItemId, ModelKind, PartialModRegistry};

mod diagnostic;

#[derive(Debug, Error, Clone)]
pub enum DeserializationErrorKind {
    #[error("Item {}({}) is missing", .1, .0)]
    MissingItem(ItemId, DatabaseItemKind),
    #[error("Item {}({}) is already declared", .1, .0)]
    DuplicateItem(ItemId, DatabaseItemKind),
    #[error("Image `{}` is missing", .0)]
    MissingImage(String),
    #[error("Image name `{}` is contested by `{}` and `{}`", .name, .path_a.to_string_lossy(), .path_b.to_string_lossy())]
    DuplicateImage {
        name: String,
        path_a: PathBuf,
        path_b: PathBuf,
    },
    #[error("Value is too large, got {} where at most {} is expected.", .got, .limit)]
    ValueTooLarge { limit: f64, got: f64 },
    #[error("Value is too small, got {} where at least {} is expected.", .got, .limit)]
    ValueTooSmall { limit: f64, got: f64 },
    #[error("File at `{}` doesn't have a name", .0.to_string_lossy())]
    MissingName(PathBuf),
    #[error("File path at `{}` is not UTF8", .0.to_string_lossy())]
    NonUtf8Path(PathBuf),
    #[error("Failed to parse an expression: {}", .0)]
    BadExpression(ExError),
}

#[derive(Debug, Clone)]
pub enum DeserializationErrorStackItem {
    Item(ItemId, DatabaseItemKind),
    Field(&'static str),
    Index(usize),
    MapEntry(String), // all JSON keys are strings, so we expect deserialized value to be reasonably displayable
    ExprVariable(String),
}

impl Display for DeserializationErrorStackItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializationErrorStackItem::Item(id, kind) => write!(f, "In item <{kind}>`{id}`"),
            DeserializationErrorStackItem::Field(name) => write!(f, "In field {name}"),
            DeserializationErrorStackItem::Index(i) => write!(f, "In item at position {i}"),
            DeserializationErrorStackItem::MapEntry(name) => {
                write!(f, "In map entry with key `{name}`")
            }
            DeserializationErrorStackItem::ExprVariable(name) => {
                write!(f, "In expression variable `{name}`")
            }
        }
    }
}

#[derive(Debug, Error, Diagnostic, Clone)]
pub struct DeserializationError {
    pub kind: DeserializationErrorKind,
    pub stack: Vec<DeserializationErrorStackItem>,
}

impl Display for DeserializationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;
        for item in &self.stack {
            write!(f, "\n{}", item)?;
        }
        Ok(())
    }
}

impl DeserializationError {
    pub fn context(mut self, item: DeserializationErrorStackItem) -> Self {
        self.stack.push(item);
        self
    }
}

impl From<DeserializationErrorKind> for DeserializationError {
    fn from(value: DeserializationErrorKind) -> Self {
        DeserializationError {
            kind: value,
            stack: Default::default(),
        }
    }
}

impl From<ExError> for DeserializationError {
    fn from(value: ExError) -> Self {
        DeserializationErrorKind::BadExpression(value).into()
    }
}

impl<T: ModelKind> From<SlabMapDuplicateError<ItemId, T>> for DeserializationError {
    fn from(SlabMapDuplicateError(id, _): SlabMapDuplicateError<ItemId, T>) -> Self {
        DeserializationErrorKind::DuplicateItem(id, T::kind()).into()
    }
}

pub(crate) trait ModelDeserializable<T> {
    fn deserialize(self, registry: &mut PartialModRegistry) -> Result<T, DeserializationError>;
}

pub trait ModelDeserializableFallbackType {
    type Serialized;
}

trait PreferredHasherForKey {
    type Hasher;
}

pub(crate) trait ApplyMin: Sized {
    type Num;
    fn apply(self, min: Self::Num) -> Result<Self, DeserializationError>;
}

pub(crate) trait ApplyMax: Sized {
    type Num;
    fn apply(self, max: Self::Num) -> Result<Self, DeserializationError>;
}

duplicate! {
    [
        ty;
        [ String ];
        [ i8 ]; [ i16 ]; [ i32 ]; [ i64 ]; [ i128 ];
        [ u8 ]; [ u16 ]; [ u32 ]; [ u64 ]; [ u128 ];
        [ f32 ]; [ f64 ];

        [ glam::f32::Vec2 ]; [ glam::f32::Vec3 ]; [ glam::f32::Vec4 ];
        [ glam::f64::DVec2 ]; [ glam::f64::DVec3 ]; [ glam::f64::DVec4 ];
        [ glam::i32::IVec2 ]; [ glam::i32::IVec3 ]; [ glam::i32::IVec4 ];
        [ glam::u32::UVec2 ]; [ glam::u32::UVec3 ]; [ glam::u32::UVec4 ];
        [ glam::i64::I64Vec2 ]; [ glam::i64::I64Vec3 ]; [ glam::i64::I64Vec4 ];
        [ glam::u64::U64Vec2 ]; [ glam::u64::U64Vec3 ]; [ glam::u64::U64Vec4 ];
        [ glam::bool::BVec2 ]; [ glam::bool::BVec3 ]; [ glam::bool::BVec4 ];
    ]
    impl ModelDeserializable<ty> for ty {
        #[inline(always)]
        fn deserialize(self, _registry: &mut PartialModRegistry) -> Result<ty, DeserializationError> {
            Ok(self)
        }
    }

    impl ModelDeserializableFallbackType for ty {
        type Serialized = ty;
    }
}

impl<T: ModelDeserializable<R>, R> ModelDeserializable<Option<R>> for Option<T> {
    #[inline(always)]
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<Option<R>, DeserializationError> {
        self.map(|e| e.deserialize(registry)).transpose()
    }
}

impl<T: ModelDeserializableFallbackType> ModelDeserializableFallbackType for Option<T> {
    type Serialized = Option<T::Serialized>;
}

impl<T: ModelDeserializable<R>, R> ModelDeserializable<Arc<R>> for SerializationBoxingWrapper<T> {
    #[inline(always)]
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<Arc<R>, DeserializationError> {
        self.0.deserialize(registry).map(Arc::new)
    }
}

impl<T: ModelDeserializableFallbackType> ModelDeserializableFallbackType for Arc<T> {
    type Serialized = SerializationBoxingWrapper<T::Serialized>;
}

impl<T: ModelDeserializable<R>, R> ModelDeserializable<Vec<R>> for Vec<T> {
    #[inline]
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<Vec<R>, DeserializationError> {
        self.into_iter()
            .enumerate()
            .map(|(i, e)| {
                e.deserialize(registry)
                    .map_err(|e| e.context(DeserializationErrorStackItem::Index(i)))
            })
            .collect()
    }
}

impl<T: ModelDeserializableFallbackType> ModelDeserializableFallbackType for Vec<T> {
    type Serialized = Vec<T::Serialized>;
}

impl<
        RawKey: ModelDeserializable<Key> + Eq + Hash + Display,
        Key: Eq + Hash,
        RawValue: ModelDeserializable<Value>,
        Value,
        RawHasher: BuildHasher,
        Hasher: BuildHasher + Default,
    > ModelDeserializable<HashMap<Key, Value, Hasher>> for HashMap<RawKey, RawValue, RawHasher>
{
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<HashMap<Key, Value, Hasher>, DeserializationError> {
        self.into_iter()
            .map(|(k, v)| {
                let v = v.deserialize(registry).map_err(|e| {
                    e.context(DeserializationErrorStackItem::MapEntry(k.to_string()))
                })?;
                // TODO: providing context here requires cloning a key, which is
                // less than desirable, but not providing context is pretty bad
                let k = k.deserialize(registry)?;
                Ok((k, v))
            })
            .collect()
    }
}

impl ModelDeserializable<Handle<bevy::prelude::Image>> for String {
    fn deserialize(
        mut self,
        registry: &mut PartialModRegistry,
    ) -> Result<Handle<bevy::prelude::Image>, DeserializationError> {
        self.make_ascii_lowercase();
        if let Some(handle) = registry.assets.images.get(&self) {
            Ok(handle.1.clone_weak())
        } else {
            Err(DeserializationErrorKind::MissingImage(self).into())
        }
    }
}

impl ModelDeserializableFallbackType for Handle<bevy::prelude::Image> {
    type Serialized = String;
}

impl<T: DatabaseItemTrait> ModelDeserializableFallbackType for SlabMapId<T> {
    type Serialized = ItemId;
}

impl<T> ModelDeserializable<T> for String
where
    for<'a> &'a str: ModelDeserializable<T>,
{
    fn deserialize(self, registry: &mut PartialModRegistry) -> Result<T, DeserializationError> {
        self.as_str().deserialize(registry)
    }
}

pub(crate) trait DeserializeFrom: Sized {
    fn deserialize_from<U>(
        data: U,
        registry: &mut PartialModRegistry,
    ) -> Result<Self, DeserializationError>
    where
        U: ModelDeserializable<Self>,
    {
        data.deserialize(registry)
    }
}

impl<T> DeserializeFrom for T {}

#[duplicate_item(
    ty trait_name err op(a,b);
    duplicate!{
        [
            ty_nested;
            [ i8 ]; [ i16 ]; [ i32 ]; [ i64 ]; [ i128 ];
            [ u8 ]; [ u16 ]; [ u32 ]; [ u64 ]; [ u128 ];
            [ f32 ]; [ f64 ];
        ]
        [ ty_nested ] [ ApplyMax ] [ ValueTooLarge ] [a > b];
        [ ty_nested ] [ ApplyMin ] [ ValueTooSmall ] [a < b];
    }
)]
impl trait_name for ty {
    type Num = ty;

    fn apply(self, limit: Self::Num) -> Result<Self, DeserializationError> {
        if op([self], [limit]) {
            #[allow(clippy::unnecessary_cast)]
            return Err(DeserializationErrorKind::err {
                limit: limit as f64,
                got: self as f64,
            }
            .into());
        }
        Ok(self)
    }
}

impl<T: ApplyMin> ApplyMin for Option<T> {
    type Num = T::Num;

    fn apply(self, min: Self::Num) -> Result<Self, DeserializationError> {
        self.map(|e| e.apply(min)).transpose()
    }
}

impl<T: ApplyMax> ApplyMax for Option<T> {
    type Num = T::Num;

    fn apply(self, max: Self::Num) -> Result<Self, DeserializationError> {
        self.map(|e| e.apply(max)).transpose()
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct SerializationBoxingWrapper<T>(T);

pub(crate) fn reserve<T>(
    map: &mut SlabMap<ItemId, Option<T>>,
    key: ItemId,
) -> Result<SlabMapReservation<T>, SlabMapDuplicateError<ItemId, Option<T>>> {
    map.insert_new(key, None)
        .map(|e| SlabMapReservation(e.as_untyped().as_typed_unchecked()))
}

pub(crate) struct SlabMapReservation<T>(SlabMapId<T>);

impl<T> SlabMapReservation<T> {
    pub fn raw(&self) -> SlabMapId<T> {
        self.0
    }
}

pub(crate) fn insert_reserved<T>(
    map: &mut SlabMap<ItemId, Option<T>>,
    reservation: SlabMapReservation<T>,
    item: T,
) -> SlabMapId<T> {
    *map.get_by_raw_mut(reservation.0.raw())
        .expect("Invalid reservation") = Some(item);
    reservation.0
}

pub(crate) fn get_reserved_key<T, Q>(
    map: &mut SlabMap<ItemId, Option<T>>,
    key: &Q,
) -> Option<SlabMapId<T>>
where
    ItemId: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
{
    map.key_to_id(key)
        .map(|e| e.as_untyped().as_typed_unchecked())
}
