use crate::model::formula::Formula;
use database_model_macro::database_model;
use std::sync::Arc;

#[database_model]
#[derive(Debug, Clone)]
pub struct Resource {
    #[model(id)]
    pub id: ResourceId,
    pub name: String,
    pub computed: Option<Arc<Formula>>,
    pub default: Option<Arc<Formula>>,
}
