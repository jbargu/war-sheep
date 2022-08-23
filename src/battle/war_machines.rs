use bevy::prelude::*;

use super::{BATTLEFIELD_BOUNDS_X, BATTLEFIELD_BOUNDS_Y};

use super::health_bars::create_war_machine_hp_bar;

use crate::utils::{Bounds, UnloadOnExit};

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
    texture: &Res<RobotSprites>,
    transform: Transform,
) -> Entity {
    let mut transform = transform;
    transform.rotation = Quat::IDENTITY;
    transform.scale = Vec3::splat(0.05);
    let id = commands
        .spawn_bundle(SpriteSheetBundle {
            transform,
            texture_atlas: texture.0.clone(),
            ..default()
        })
        .insert(WarMachine)
        .insert(UnloadOnExit)
        .insert(Bounds {
            x: (BATTLEFIELD_BOUNDS_X.x, BATTLEFIELD_BOUNDS_X.y),
            y: (BATTLEFIELD_BOUNDS_Y.x, BATTLEFIELD_BOUNDS_Y.y),
        })
        .insert(AnimationTimer(Timer::from_seconds(0.1, true)))
        .id();

    create_war_machine_hp_bar(id, commands);

    id
}

pub struct RobotSprites(Handle<TextureAtlas>);

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

pub fn animate_war_machine(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

pub fn load_war_machine_graphics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("robot_tileset.png");
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle,
        Vec2::new(16.0, 32.0),
        7,
        1,
        Vec2::ZERO,
        Vec2::new(288.0, 80.0),
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands.insert_resource(RobotSprites(texture_atlas_handle));
}
