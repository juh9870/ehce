use crate::model::component_stats::ComponentStatsId;
use database_model_macro::database_model;

#[database_model]
#[derive(Debug, Clone)]
pub struct Component {
    #[model(id)]
    pub id: ComponentId,
    pub stats: ComponentStatsId,
}
