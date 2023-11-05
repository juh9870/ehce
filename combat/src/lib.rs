use bevy::ecs::prelude::States;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
struct CombatState;

impl States for CombatState {}
