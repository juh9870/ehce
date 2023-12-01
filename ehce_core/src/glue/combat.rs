use bevy::prelude::Resource;
use database::model::combat_settings::CombatSettingsData;
use database::model::fleet::FleetData;

#[derive(Debug, Clone, Resource)]
pub struct CombatInit {
    pub player_fleet: FleetData,
    pub enemy_fleet: FleetData,
    pub combat_settings: CombatSettingsData,
}
