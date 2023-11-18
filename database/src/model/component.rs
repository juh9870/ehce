use crate::model::component_stats::ComponentStats;
use database_model_macro::database_model;
use utils::slab_map::SlabMapId;

#[database_model]
#[derive(Debug, Clone)]
pub struct Component {
    #[model(id)]
    pub id: SlabMapId<Component>,
    pub stats: SlabMapId<ComponentStats>,
}
