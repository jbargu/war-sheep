use bevy::prelude::*;
use bevy_simple_stat_bars::prelude::*;

use crate::utils::{Health, UnloadOnExit};

pub fn create_war_machine_hp_bar(id: Entity, commands: &mut Commands) {
    commands
        .spawn_bundle((
            StatBarColor(Color::RED),
            StatBarEmptyColor(Color::BLACK),
            StatBarBorder {
                color: Color::DARK_GRAY,
                thickness: 0.1,
            },
            StatBarValue(1.0),
            StatBarSize {
                full_length: 0.9,
                thickness: 0.15,
            },
            StatBarSubject(id),
            StatBarPosition(-0.8 * Vec2::Y),
            component_observer(|hp: &Health| hp.current as f32 / hp.max as f32),
        ))
        .insert(UnloadOnExit);
}

pub fn create_sheep_hp_bar(id: Entity, commands: &mut Commands) {
    commands
        .spawn_bundle((
            StatBarColor(Color::GREEN),
            StatBarEmptyColor(Color::BLACK),
            StatBarBorder {
                color: Color::DARK_GRAY,
                thickness: 0.1,
            },
            StatBarValue(1.0),
            StatBarSize {
                full_length: 0.9,
                thickness: 0.15,
            },
            StatBarSubject(id),
            StatBarPosition(-0.8 * Vec2::Y),
            component_observer(|hp: &Health| hp.current as f32 / hp.max as f32),
        ))
        .insert(UnloadOnExit);
}
