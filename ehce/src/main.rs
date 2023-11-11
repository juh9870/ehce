use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(AssetPlugin {
            mode: AssetMode::Processed,
            ..Default::default()
        }),))
        .add_plugins(ehce_core::CorePlugin)
        .add_plugins(combat::CombatPlugin)
        .run()
}
