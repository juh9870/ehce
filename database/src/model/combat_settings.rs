use crate::model::{CombatSettingsId, FleetOrId};
use database_model_macro::database_model;

#[database_model]
#[derive(Debug, Clone)]
pub struct CombatSettings {
    pub parent: Option<CombatSettingsId>,
    pub player_fleet: FleetOrId,
    pub enemy_fleet: FleetOrId,
}
