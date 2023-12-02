use bevy::prelude::*;

fn main() {
    color_backtrace::install();
    App::new()
        .add_plugins((DefaultPlugins.set(AssetPlugin {
            mode: AssetMode::Unprocessed,
            file_path: "mods".to_string(),
            processed_file_path: "tmp".to_string(),
            ..Default::default()
        }),))
        .add_plugins(ehce_core::CorePlugin)
        .add_plugins(combat::CombatPlugin)
        .run()
}
