use bimap::BiHashMap;
use std::hash::BuildHasherDefault;

pub mod miette_ext;
pub mod slab_map;

pub type FxBiHashMap<
    L,
    R,
    LS = BuildHasherDefault<rustc_hash::FxHasher>,
    RS = BuildHasherDefault<rustc_hash::FxHasher>,
> = BiHashMap<L, R, LS, RS>;

pub use nohash_hasher::IntMap;
