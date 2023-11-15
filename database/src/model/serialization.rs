use std::fmt::{Display, Formatter};

use duplicate::duplicate_item;
use miette::Diagnostic;
use thiserror::Error;

use utils::slab_map::{SlabMap, SlabMapDuplicateError, SlabMapId};

use crate::model::{DatabaseItemKind, ItemId, ModelKind, PartialModRegistry};

#[derive(Debug, Error, Diagnostic, Clone)]
pub enum DeserializationErrorKind {
    #[error("Item <{}>`{}` is missing", .1, .0)]
    MissingItem(ItemId, DatabaseItemKind),
    #[error("Item <{}>`{}` is already declared", .1, .0)]
    DuplicateItem(ItemId, DatabaseItemKind),
    #[error("Value is too large, got {} where at most {} is expected.", .got, .limit)]
    ValueTooLarge { limit: f64, got: f64 },
    #[error("Value is too small, got {} where at least {} is expected.", .got, .limit)]
    ValueTooSmall { limit: f64, got: f64 },
}

#[derive(Debug, Clone)]
pub enum DeserializationErrorStackItem {
    Item(ItemId, DatabaseItemKind),
    Field(&'static str),
}

impl Display for DeserializationErrorStackItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializationErrorStackItem::Item(id, kind) => write!(f, "In item <{kind}>`{id}`"),
            DeserializationErrorStackItem::Field(name) => write!(f, "In field {name}"),
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
            write!(f, "{}", item)?;
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

impl<T: ModelKind> From<SlabMapDuplicateError<ItemId, T>> for DeserializationError {
    fn from(SlabMapDuplicateError(id, _): SlabMapDuplicateError<ItemId, T>) -> Self {
        DeserializationErrorKind::DuplicateItem(id, T::kind()).into()
    }
}

pub(crate) trait ModelDeserializable<T> {
    fn deserialize(self, registry: &mut PartialModRegistry) -> Result<T, DeserializationError>;
}

pub(crate) trait ApplyMin: Sized {
    type Num;
    fn apply(self, min: Self::Num) -> Result<Self, DeserializationError>;
}

pub(crate) trait ApplyMax: Sized {
    type Num;
    fn apply(self, max: Self::Num) -> Result<Self, DeserializationError>;
}
#[duplicate_item(
    ty;
    [ String ];
    [ i8 ]; [ i16 ]; [ i32 ]; [ i64 ]; [ i128 ];
    [ u8 ]; [ u16 ]; [ u32 ]; [ u64 ]; [ u128 ];
    [ f32 ]; [ f64 ];
)]
impl ModelDeserializable<ty> for ty {
    #[inline(always)]
    fn deserialize(self, _registry: &mut PartialModRegistry) -> Result<ty, DeserializationError> {
        Ok(self)
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

impl<T: ModelDeserializable<R>, R> ModelDeserializable<Vec<R>> for Vec<T> {
    #[inline]
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<Vec<R>, DeserializationError> {
        self.into_iter().map(|e| e.deserialize(registry)).collect()
    }
}

#[duplicate_item(
    ty trait_name op(a,b);
    duplicate!{
        [
            ty_nested;
            [ i8 ]; [ i16 ]; [ i32 ]; [ i64 ]; [ i128 ];
            [ u8 ]; [ u16 ]; [ u32 ]; [ u64 ]; [ u128 ];
            [ f32 ]; [ f64 ];
        ]
        [ ty_nested ] [ApplyMax] [a > b];
        [ ty_nested ] [ApplyMin] [a < b];
    }
)]
impl trait_name for ty {
    type Num = ty;

    fn apply(self, limit: Self::Num) -> Result<Self, DeserializationError> {
        if op([self], [limit]) {
            #[allow(clippy::unnecessary_cast)]
            return Err(DeserializationErrorKind::ValueTooLarge {
                limit: limit as f64,
                got: self as f64,
            }
            .into());
        }
        Ok(self)
    }
}

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

pub(crate) fn get_reserved_key<T>(
    map: &mut SlabMap<ItemId, Option<T>>,
    key: &ItemId,
) -> Option<SlabMapId<T>> {
    map.key_to_id(key)
        .map(|e| e.as_untyped().as_typed_unchecked())
}
