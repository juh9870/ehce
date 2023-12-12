use bevy::prelude::Resource;
use database::model::combat_settings::CombatSettings;
use database::model::fleet::Fleet;

#[derive(Debug, Clone, Resource)]
pub struct CombatInit {
    pub player_fleet: Fleet,
    pub enemy_fleet: Fleet,
    pub combat_settings: CombatSettings,
}
