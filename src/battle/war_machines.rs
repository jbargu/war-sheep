use crate::audio::AnimationAudioPlayback;
use crate::utils::Speed;
use bevy::prelude::*;
use bevy::utils::HashMap;
use iyes_loopless::prelude::*;

use super::{BATTLEFIELD_BOUNDS_X, BATTLEFIELD_BOUNDS_Y};

use super::health_bars::create_war_machine_hp_bar;

use crate::animation::{Animation, Sheet};
use crate::battle::states::{Attacking, Dying, Idling, Walking};
use crate::sheep::Sheep;
use crate::utils::{Attack, BehaviourType, Bounds, Health, UnloadOnExit};
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
                    .with_system(walking)
                    .with_system(attacking)
                    .with_system(dying)
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
    let texture_handle = asset_server.load("EatingRobot.png");

    // Add idling animation
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle.clone(),
        Vec2::new(16.0, 32.0),
        4,
        1,
        Vec2::ZERO,
        Vec2::new(0.0, 0.0),
    );

    let len = texture_atlas.len();
    animations_map.insert(
        Idling::ANIMATION.to_owned(),
        Sheet {
            atlas_handle: texture_atlases.add(texture_atlas),
            length: len,
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
        Vec2::new(0.0, 32.0),
    );

    let len = texture_atlas.len();
    animations_map.insert(
        Walking::ANIMATION.to_owned(),
        Sheet {
            atlas_handle: texture_atlases.add(texture_atlas),
            length: len,
            repeating: true,
        },
    );

    // Add attacking animation
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle.clone(),
        Vec2::new(16.0, 32.0),
        4,
        1,
        Vec2::ZERO,
        Vec2::new(0.0, 64.0),
    );

    let len = texture_atlas.len();
    animations_map.insert(
        Attacking::ANIMATION.to_owned(),
        Sheet {
            atlas_handle: texture_atlases.add(texture_atlas),
            length: len,
            repeating: true,
        },
    );

    // Add dying animation
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle.clone(),
        Vec2::new(16.0, 32.0),
        4,
        1,
        Vec2::ZERO,
        Vec2::new(0.0, 96.0),
    );

    let len = texture_atlas.len();
    animations_map.insert(
        Dying::ANIMATION.to_owned(),
        Sheet {
            atlas_handle: texture_atlases.add(texture_atlas),
            length: len,
            repeating: false,
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

fn walking(
    mut commands: Commands,
    mut sheep_q: Query<&mut Transform, (With<Sheep>, Without<WarMachine>)>,
    mut war_machines_q: Query<
        (
            Entity,
            &mut Transform,
            &Attack,
            &BehaviourType,
            &Speed,
            &mut Animation,
        ),
        (With<Walking>, With<WarMachine>, Without<Sheep>),
    >,
    time: Res<Time>,
) {
    for (wm_entity, mut wm_transform, attack, behaviour_type, speed, mut animation) in
        war_machines_q.iter_mut()
    {
        // Start animation if we have not yet
        if animation.current_animation.as_deref() != Some(Walking::ANIMATION) {
            animation.play(Walking::ANIMATION, true);

            // Add spotting sound on 2nd frame to not be to obnoxious
            //commands
            //.entity(wm_entity)
            //.insert(AnimationAudioPlayback::new(
            //Walking::ANIMATION.to_owned(),
            //HashMap::from([(2, String::from("audio/robot_engaged.mp3"))]),
            //));
        }

        // Check whether any sheep are within spotting_range
        let mut sheep = sheep_q
            .iter_mut()
            .filter(|sheep_transform| {
                wm_transform
                    .translation
                    .truncate()
                    .distance(sheep_transform.translation.truncate())
                    <= attack.spotting_range
            })
            .collect::<Vec<_>>();

        // Transition to Idling if no sheep are found
        if sheep.is_empty() {
            commands.entity(wm_entity).remove::<Walking>();
            commands.entity(wm_entity).insert(Idling);
            continue;
        }

        // Otherwise check if we can attack any of the sheep
        sheep.sort_by(|transform1, transform2| {
            wm_transform
                .translation
                .truncate()
                .distance(transform1.translation.truncate())
                .partial_cmp(
                    &wm_transform
                        .translation
                        .truncate()
                        .distance(transform2.translation.truncate()),
                )
                .unwrap()
        });

        // Find the closest sheep
        if let Some(sheep_transform) = sheep.get_mut(0) {
            let difference =
                sheep_transform.translation.truncate() - wm_transform.translation.truncate();

            // If the sheep is within attack_range, transition into Attacking state
            if difference.length() <= attack.attack_range {
                commands.entity(wm_entity).remove::<Walking>();
                commands.entity(wm_entity).insert(Attacking::default());
                continue;
            }

            // Oterwise move towards the sheep depending on the `behaviour_type`
            match behaviour_type {
                BehaviourType::ChasingClosest => {
                    let direction = difference.normalize_or_zero();

                    if difference.length() >= attack.attack_range * 0.5 {
                        animation.flip_x = direction.x <= 0.0;

                        wm_transform.translation +=
                            direction.extend(0.0) * speed.0 * time.delta_seconds();
                    }
                }
            }
        }
    }
}

fn attacking(
    mut commands: Commands,
    mut sheep_q: Query<(&mut Health, &mut Transform), (With<Sheep>, Without<WarMachine>)>,
    mut war_machines_q: Query<
        (
            Entity,
            &mut Transform,
            &Attack,
            &mut Animation,
            &mut Attacking,
        ),
        (With<Attacking>, With<WarMachine>, Without<Sheep>),
    >,
) {
    for (wm_entity, wm_transform, attack, mut animation, mut attacking) in war_machines_q.iter_mut()
    {
        if !attacking.has_started {
            attacking.has_started = true;

            animation.play(Attacking::ANIMATION, false);

            // Add eating sound
            commands
                .entity(wm_entity)
                .insert(AnimationAudioPlayback::new(
                    Attacking::ANIMATION.to_owned(),
                    HashMap::from([(1, String::from("audio/robot_eat.mp3"))]),
                ));

            // Check whether any sheep are within attack range
            let mut sheep = sheep_q
                .iter_mut()
                .filter(|(_, sheep_transform)| {
                    wm_transform
                        .translation
                        .truncate()
                        .distance(sheep_transform.translation.truncate())
                        <= attack.attack_range
                })
                .collect::<Vec<_>>();

            // Transition to Idling if no sheep are found
            if sheep.is_empty() {
                commands.entity(wm_entity).remove::<Attacking>();
                commands.entity(wm_entity).insert(Idling);
                continue;
            }

            // Otherwise sort sheep to find the closest one to attack
            sheep.sort_by(|(_, transform1), (_, transform2)| {
                wm_transform
                    .translation
                    .truncate()
                    .distance(transform1.translation.truncate())
                    .partial_cmp(
                        &wm_transform
                            .translation
                            .truncate()
                            .distance(transform2.translation.truncate()),
                    )
                    .unwrap()
            });

            // Attack the sheep
            if let Some((ref mut sheep_health, sheep_transform)) = sheep.get_mut(0) {
                let difference =
                    sheep_transform.translation.truncate() - wm_transform.translation.truncate();

                animation.flip_x = difference.normalize_or_zero().x <= 0.0;
                sheep_health.current -= attack.attack_damage;
            }
        }

        if animation.has_finished() {
            commands.entity(wm_entity).remove::<Attacking>();
            commands.entity(wm_entity).insert(Idling);
        }
    }
}

fn dying(
    mut commands: Commands,
    mut war_machines_q: Query<
        (Entity, &mut Animation, &mut Dying),
        (With<Dying>, With<WarMachine>),
    >,
) {
    for (entity, mut animation, mut dying) in war_machines_q.iter_mut() {
        if !dying.has_started {
            dying.has_started = true;

            animation.play(Dying::ANIMATION, false);
        }

        // Remove war machine if animation expired
        if animation.has_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
