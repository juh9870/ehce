use crate::model::serialization::{
    DeserializationError, DeserializeFrom, ModelDeserializable, ModelDeserializableFallbackType,
};
use crate::model::PartialModRegistry;
use database_model_macro::database_model;
use exmex::Express;
use itertools::Itertools;
use utils::slab_map::SlabMapId;

#[database_model]
#[derive(Debug, Clone)]
pub struct Resource {
    #[model(id)]
    pub id: SlabMapId<Resource>,
    pub name: String,
    pub eval: Option<Formula>,
}

#[derive(Debug, Clone)]
pub struct Formula {
    pub expr: exmex::FlatEx<f64>,
    pub args: Vec<SlabMapId<Resource>>,
}

impl ModelDeserializableFallbackType for Formula {
    type Serialized = String;
}

impl ModelDeserializable<Formula> for &str {
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<Formula, DeserializationError> {
        let formula = exmex::parse::<f64>(self)?;

        let args = formula
            .var_names()
            .iter()
            .map(|id| SlabMapId::<Resource>::deserialize_from(id.as_str(), registry))
            .try_collect()?;

        Ok(Formula {
            expr: formula,
            args,
        })
    }
}
