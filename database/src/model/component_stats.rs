use crate::model::characteristic::Characteristic;
use crate::model::ItemId;
use database_model_macro::database_model;
use rustc_hash::FxHashMap;
use utils::slab_map::SlabMapId;
use utils::IntMap;

#[database_model]
#[derive(Debug, Clone)]
pub struct ComponentStats {
    #[model(id)]
    pub id: SlabMapId<ComponentStats>,
    #[model(ty=FxHashMap<ItemId, f64>)]
    pub stats: IntMap<SlabMapId<Characteristic>, f64>,
}
