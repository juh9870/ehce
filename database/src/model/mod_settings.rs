use database_model_macro::database_model;

use crate::model::CombatSettingsId;

#[database_model]
#[derive(Debug, Clone)]
pub struct ModSettings {
    pub name: String,
    pub mod_id: String,
    pub defaults: Defaults,
}

#[database_model]
#[derive(Debug, Clone)]
pub struct Defaults {
    pub combat_settings: CombatSettingsId,
}
