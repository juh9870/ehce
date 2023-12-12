use crate::model::ComponentStatsId;
use database_model_macro::database_model;

#[database_model]
#[derive(Debug, Clone)]
pub struct Component {
    pub stats: ComponentStatsId,
}
