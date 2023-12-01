use crate::resources::Resources;
use bevy::prelude::{Bundle, Component};
use std::hash::{Hash, Hasher};

pub mod ship;

/// Basic combat unit
#[derive(Debug, Clone, Component)]
pub struct Unit {}

// region Team

/// Component denoting team of the affected unit
#[derive(Debug, Copy, Clone, Eq, PartialEq, Component)]
pub struct Team(usize);

impl Team {
    pub fn new_unchecked_do_not_use_directly_its_bad_really_will_be_very_hard_to_migrate_later(
        id: usize,
    ) -> Team {
        Self(id)
    }
}

impl Hash for Team {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl nohash_hasher::IsEnabled for Team {}

// endregion

#[derive(Bundle)]
pub struct UnitBundle {
    pub unit: Unit,
    pub team: Team,
    pub resources: Resources,
}
