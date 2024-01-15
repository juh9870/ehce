use schemars::JsonSchema;
use scrapcore_serialization_macro::registry;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct A {}

impl crate::serialization::SerializationFallback for A {
    type Fallback = ASerialzied;
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ASerialzied {}

#[registry]
struct Model {
    #[model(registry)]
    test: A,
}
