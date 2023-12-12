use crate::resources::Resources;
use bevy::prelude::Component;
use ehce_core::database::model::fleet::Fleet;
use ehce_core::database::model::ShipBuildId;
use soa_derive::StructOfArray;

#[derive(Debug, Component)]
pub struct CombatFleet {
    pub units: FleetUnitVec,
}

impl From<&Fleet> for CombatFleet {
    fn from(value: &Fleet) -> Self {
        let units = value.builds.iter().map(|e| FleetUnit::new(*e)).collect();

        Self { units }
    }
}

#[derive(Debug, StructOfArray)]
#[soa_derive(Debug)]
pub struct FleetUnit {
    pub build: ShipBuildId,
    pub resources: Option<Resources>,
    pub alive: bool,
}

impl FleetUnit {
    pub fn new(build: ShipBuildId) -> Self {
        Self {
            build,
            resources: Default::default(),
            alive: true,
        }
    }
}
