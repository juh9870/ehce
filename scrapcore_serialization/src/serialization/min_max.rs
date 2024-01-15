use crate::registry::SerializationRegistry;
use crate::serialization::error::{DeserializationError, DeserializationErrorKind};
use duplicate::duplicate_item;

pub(crate) trait ApplyMin<Registry: SerializationRegistry>: Sized {
    type Num;
    fn apply(self, min: Self::Num) -> Result<Self, DeserializationError<Registry>>;
}

pub(crate) trait ApplyMax<Registry: SerializationRegistry>: Sized {
    type Num;
    fn apply(self, max: Self::Num) -> Result<Self, DeserializationError<Registry>>;
}

#[duplicate_item(
ty trait_name err op(a, b);
duplicate ! {
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
impl<Registry: SerializationRegistry> trait_name<Registry> for ty {
    type Num = ty;

    fn apply(self, limit: Self::Num) -> Result<Self, DeserializationError<Registry>> {
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

impl<Registry: SerializationRegistry, T: ApplyMin<Registry>> ApplyMin<Registry> for Option<T> {
    type Num = T::Num;

    fn apply(self, min: Self::Num) -> Result<Self, DeserializationError<Registry>> {
        self.map(|e| e.apply(min)).transpose()
    }
}

impl<Registry: SerializationRegistry, T: ApplyMax<Registry>> ApplyMax<Registry> for Option<T> {
    type Num = T::Num;

    fn apply(self, max: Self::Num) -> Result<Self, DeserializationError<Registry>> {
        self.map(|e| e.apply(max)).transpose()
    }
}
