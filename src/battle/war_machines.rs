use bevy::prelude::*;
use bevy::utils::HashMap;
use iyes_loopless::prelude::*;

use super::{BATTLEFIELD_BOUNDS_X, BATTLEFIELD_BOUNDS_Y};

use super::health_bars::create_war_machine_hp_bar;

use crate::animation::{Animation, Sheet};
use crate::battle::states::{Attacking, Dying, Idling, Walking};
use crate::sheep::Sheep;
use crate::utils::{Attack, Bounds, Health, UnloadOnExit};
use crate::GameState;

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

pub struct WarMachinePlugin;

impl Plugin for WarMachinePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_war_machine_graphics)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Battle)
                    .with_system(idling)
                    .into(),
            );
    }
}

pub fn new_war_machine(
    commands: &mut Commands,
    animations_map: &Res<RobotAnimations>,
    transform: Transform,
) -> Entity {
    let mut transform = transform;
    transform.rotation = Quat::IDENTITY;
    transform.scale = Vec3::splat(0.05);
    let id = commands
        .spawn_bundle(SpriteSheetBundle {
            transform,
            texture_atlas: animations_map
                .0
                .get(Idling::ANIMATION)
                .unwrap()
                .atlas_handle
                .clone(),
            ..default()
        })
        .insert(Animation::new(0.1, animations_map.0.clone()))
        .insert(Idling)
        .insert(WarMachine)
        .insert(UnloadOnExit)
        .insert(Bounds {
            x: (BATTLEFIELD_BOUNDS_X.x, BATTLEFIELD_BOUNDS_X.y),
            y: (BATTLEFIELD_BOUNDS_Y.x, BATTLEFIELD_BOUNDS_Y.y),
        })
        .id();

    create_war_machine_hp_bar(id, commands);

    id
}

pub struct RobotAnimations(HashMap<String, Sheet>);

pub fn load_war_machine_graphics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut animations_map: HashMap<String, Sheet> = HashMap::new();
    let texture_handle = asset_server.load("robot_tileset.png");

    // Add idling animation
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle.clone(),
        Vec2::new(16.0, 32.0),
        7,
        1,
        Vec2::ZERO,
        Vec2::new(288.0, 80.0),
    );
    animations_map.insert(
        Idling::ANIMATION.to_owned(),
        Sheet {
            atlas_handle: texture_atlases.add(texture_atlas),
            length: 7,
            repeating: true,
        },
    );

    // Add walking animation
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle.clone(),
        Vec2::new(16.0, 32.0),
        7,
        1,
        Vec2::ZERO,
        Vec2::new(288.0, 48.0),
    );
    animations_map.insert(
        Walking::ANIMATION.to_owned(),
        Sheet {
            atlas_handle: texture_atlases.add(texture_atlas),
            length: 7,
            repeating: true,
        },
    );

    // Add attacking animation
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle.clone(),
        Vec2::new(16.0, 32.0),
        7,
        1,
        Vec2::ZERO,
        Vec2::new(288.0, 144.0),
    );
    animations_map.insert(
        Attacking::ANIMATION.to_owned(),
        Sheet {
            atlas_handle: texture_atlases.add(texture_atlas),
            length: 7,
            repeating: true,
        },
    );
    commands.insert_resource(RobotAnimations(animations_map));
}

fn idling(
    mut commands: Commands,
    mut sheep_q: Query<(&mut Health, &mut Transform), (With<Sheep>, Without<WarMachine>)>,
    mut war_machines_q: Query<
        (Entity, &Transform, &Attack, &mut Animation),
        (With<Idling>, With<WarMachine>, Without<Sheep>),
    >,
) {
    for (wm_entity, wm_transform, attack, mut animation) in war_machines_q.iter_mut() {
        // Start animation if we have not yet
        if animation.current_animation.as_deref() != Some(Idling::ANIMATION) {
            animation.play(Idling::ANIMATION, true)
        }

        // Check whether any sheep are within spotting_range
        let sheep = sheep_q
            .iter_mut()
            .filter(|(_, sheep_transform)| {
                wm_transform
                    .translation
                    .truncate()
                    .distance(sheep_transform.translation.truncate())
                    <= attack.spotting_range
            })
            .collect::<Vec<_>>();

        // Transition to Walking if any sheep are found
        if !sheep.is_empty() {
            commands.entity(wm_entity).remove::<Idling>();
            commands.entity(wm_entity).insert(Walking);
        }
    }
}
