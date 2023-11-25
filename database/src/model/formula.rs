use crate::model::resource::Resource;
use crate::model::serialization::{
    DeserializationError, DeserializationErrorStackItem, DeserializeFrom, ModelDeserializable,
    ModelDeserializableFallbackType,
};
use crate::model::PartialModRegistry;
use exmex::Express;
use itertools::Itertools;
use utils::slab_map::SlabMapId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum SerializedFormula {
    String(String),
    Number(f64),
}

#[derive(Debug, Clone)]
pub struct Formula {
    pub expr: exmex::FlatEx<f64>,
    pub args: Vec<SlabMapId<Resource>>,
}

impl ModelDeserializableFallbackType for Formula {
    type Serialized = SerializedFormula;
}

impl ModelDeserializable<Formula> for SerializedFormula {
    fn deserialize(
        self,
        registry: &mut PartialModRegistry,
    ) -> Result<Formula, DeserializationError> {
        match self {
            SerializedFormula::String(formula) => {
                Formula::deserialize_from(formula.as_str(), registry)
            }
            SerializedFormula::Number(num) => {
                let formula = num.to_string();
                Formula::deserialize_from(formula.as_str(), registry)
            }
        }
    }
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
            .map(|id| {
                SlabMapId::<Resource>::deserialize_from(id.as_str(), registry).map_err(|e| {
                    e.context(DeserializationErrorStackItem::ExprVariable(id.to_string()))
                })
            })
            .try_collect()?;

        Ok(Formula {
            expr: formula,
            args,
        })
    }
}
