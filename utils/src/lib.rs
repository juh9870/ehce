use std::hash::BuildHasherDefault;

use bimap::BiHashMap;

pub mod miette_ext;

pub type FxBiHashMap<
    L,
    R,
    LS = BuildHasherDefault<rustc_hash::FxHasher>,
    RS = BuildHasherDefault<rustc_hash::FxHasher>,
> = BiHashMap<L, R, LS, RS>;
