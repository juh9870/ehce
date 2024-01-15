use bevy::prelude::Component;
use soa_derive::StructOfArray;

use ehce_core::database::model::fleet::Fleet;
use ehce_core::database::model::ShipBuildId;

use crate::variables::Variables;

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
    pub variables: Option<Variables>,
    pub alive: bool,
}

impl FleetUnit {
    pub fn new(build: ShipBuildId) -> Self {
        Self {
            build,
            variables: Default::default(),
            alive: true,
        }
    }
}
