use crate::fleet::CombatFleet;
use crate::resources::{ResourceEvaluationError, Resources};
use crate::unit::ship::{calculate_resources, make_ship};
use crate::unit::{Team, Unit, UnitBundle};
use crate::EmitCombatError;
use bevy::log::info;
use bevy::prelude::{Assets, Commands, Image, Query, Res, With};
use bevy_mod_sysfail::sysfail;
use ehce_core::database::model::ship_build::ShipBuildData;
use ehce_core::mods::ModData;
use nohash_hasher::IntSet;

#[sysfail(EmitCombatError)]
pub fn ship_spawn(
    ships: Query<&Team, With<Unit>>,
    mut fleets: Query<(&mut CombatFleet, &Team)>,
    db: Res<ModData>,
    images: Res<Assets<Image>>,
    mut commands: Commands,
) {
    let mut team_has_ships = IntSet::default();

    for ship_team in ships.iter() {
        team_has_ships.insert(*ship_team);
    }

    for (mut fleet, team) in fleets.iter_mut() {
        if team_has_ships.contains(team) {
            continue;
        }

        let Some(next) = fleet.units.iter_mut().find(|e| *e.alive) else {
            continue;
        };

        info!(?team, "Spawning a ship");

        spawn_ship(
            &db,
            &db.registry[*next.build],
            *team,
            std::mem::take(next.resources),
            &images,
            &mut commands,
        )?;
    }
}

utils::bubbled!(
    ShipSpawnError("Failed to spawn a ship") {
        ResourceEvaluationError,
    }
);

fn spawn_ship(
    db: &ModData,
    build: impl AsRef<ShipBuildData>,
    team: Team,
    resources: Option<Resources>,
    images: &Assets<Image>,
    commands: &mut Commands,
) -> Result<(), ShipSpawnError> {
    let build = build.as_ref();
    let ship = &db.registry[build.ship];
    let ship_bundle = make_ship(db, ship, images);
    let resources = if let Some(resources) = resources {
        resources
    } else {
        calculate_resources(db, ship, build)?
    };

    commands.spawn((
        ship_bundle,
        UnitBundle {
            unit: Unit {},
            team,
            resources,
        },
    ));

    Ok(())
}
