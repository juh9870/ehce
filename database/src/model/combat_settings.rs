use database_model_macro::database_model;

#[database_model]
#[derive(Debug, Clone)]
pub struct CombatSettings {
    #[model(id)]
    pub id: CombatSettingsId,
    #[model_serde(flatten)]
    #[model(as_ref)]
    pub data: CombatSettingsData,
}
#[database_model]
#[derive(Debug, Clone)]
pub struct CombatSettingsData {}
