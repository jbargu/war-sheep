use bevy::prelude::*;

mod debug;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(debug::DebugPlugin)
        .run();
}
