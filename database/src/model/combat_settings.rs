use database_model_macro::database_model;

#[database_model]
#[derive(Debug, Clone)]
pub struct CombatSettings {
    #[model(id)]
    id: CombatSettingsId,
}
