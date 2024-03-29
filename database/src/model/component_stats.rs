use crate::model::{ItemId, VariableId};
use database_model_macro::database_model;
use nohash_hasher::IntMap;
use rustc_hash::FxHashMap;

#[database_model]
#[derive(Debug, Clone)]
pub struct ComponentStats {
    #[model(ty = FxHashMap < ItemId, f64 >)]
    pub stats: IntMap<VariableId, f64>,
}
