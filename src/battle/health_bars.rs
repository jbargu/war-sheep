use bevy::prelude::*;
use bevy_simple_stat_bars::prelude::*;

use crate::utils::{Health, UnloadOnExit};

#[derive(Component)]
pub struct StatBars {
    pub hp: Entity,
}

pub fn create_war_machine_hp_bar(entity: Entity, commands: &mut Commands) {
    let hp_bar = commands
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
            StatBarSubject(entity),
            StatBarPosition(-0.8 * Vec2::Y),
        ))
        .insert(UnloadOnExit)
        .id();

    commands.entity(entity).insert(StatBars { hp: hp_bar });
}

pub fn create_sheep_hp_bar(entity: Entity, commands: &mut Commands) {
    let hp_bar = commands
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
            StatBarSubject(entity),
            StatBarPosition(-0.8 * Vec2::Y),
        ))
        .insert(UnloadOnExit)
        .id();

    commands.entity(entity).insert(StatBars { hp: hp_bar });
}

pub fn update_health_bars(
    mut stats: Query<(&mut Health, &StatBars)>,
    mut stat_bars: Query<&mut StatBarValue>,
) {
    stats.for_each_mut(|(hp, bars)| {
        if let Ok(mut hp_bar) = stat_bars.get_mut(bars.hp) {
            hp_bar.0 = (hp.current as f32 / hp.max as f32).clamp(0.0, 1.0);
        }
    });
}
