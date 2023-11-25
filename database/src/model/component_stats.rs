use crate::model::resource::ResourceId;
use crate::model::ItemId;
use database_model_macro::database_model;
use nohash_hasher::IntMap;
use rustc_hash::FxHashMap;

#[database_model]
#[derive(Debug, Clone)]
pub struct ComponentStats {
    #[model(id)]
    pub id: ComponentStatsId,
    #[model(ty=FxHashMap<ItemId, f64>)]
    pub stats: IntMap<ResourceId, f64>,
}
