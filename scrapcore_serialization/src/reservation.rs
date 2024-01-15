use crate::ItemId;
use slabmap::{SlabMap, SlabMapDuplicateError, SlabMapId};
use std::borrow::Borrow;
use std::hash::Hash;

pub(crate) struct SlabMapReservation<T>(SlabMapId<T>);

pub(crate) fn reserve<T>(
    map: &mut SlabMap<ItemId, Option<T>>,
    key: ItemId,
) -> Result<SlabMapReservation<T>, SlabMapDuplicateError<ItemId, Option<T>>> {
    map.insert_new(key, None)
        .map(|e| SlabMapReservation(e.as_untyped().as_typed_unchecked()))
}

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
    map: &SlabMap<ItemId, Option<T>>,
    key: &Q,
) -> Option<SlabMapId<T>>
where
    ItemId: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
{
    map.key_to_id(key)
        .map(|e| e.as_untyped().as_typed_unchecked())
}
