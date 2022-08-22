use bevy::prelude::*;

use super::{BATTLEFIELD_BOUNDS_X, BATTLEFIELD_BOUNDS_Y};

use super::health_bars::create_war_machine_hp_bar;

use crate::utils::Bounds;

// Every WarMachine is defined by:
// - `SpottingRange`: if a sheep is found within this radius, it will be pursued
// - `AttackRange`: if a sheep is within this radius, it will be attacked by `AttackValue`
// - `AttackValue`: attack damage value
// - `Health`:  if health value falls below 0, it dies
// - `Speed`: how fast it moves
// - `PursuitType`: how it selects the next sheep to hunt
// - any other traits that may alter behaviour
#[derive(Component, Default)]
pub struct WarMachine;

pub fn new_war_machine(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    transform: Transform,
) -> Entity {
    let mut transform = transform;
    transform.rotation = Quat::IDENTITY;
    let id = commands
        .spawn_bundle(SpriteBundle {
            transform,
            texture: asset_server.load("BaseSheep.png"),
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::splat(1.0)),
                ..default()
            },
            ..default()
        })
        .insert(WarMachine)
        .insert(Bounds {
            x: (BATTLEFIELD_BOUNDS_X.x, BATTLEFIELD_BOUNDS_X.y),
            y: (BATTLEFIELD_BOUNDS_Y.x, BATTLEFIELD_BOUNDS_Y.y),
        })
        .id();

    create_war_machine_hp_bar(id, commands);

    id
}
