use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::sheep;
use crate::utils::{
    bounds_check, despawn_entities_with_component, Attack, BehaviourType, Health, Speed,
    UnloadOnExit,
};
use rand::{thread_rng, Rng};

use crate::ui::{write_text, AsciiSheet};
use crate::GameState;
use health_bars::{create_sheep_hp_bar, update_health_bars};
use war_machines::{animate_war_machine, load_war_machine_graphics, new_war_machine, WarMachine};

mod health_bars;
mod war_machines;

/// Resource for keeping battle timer, after it runs out, there is a tie
pub struct BattleTimer(Timer);

/// Marker component for the battle timer text
#[derive(Component)]
pub struct BattleTimerText;

#[derive(PartialEq)]
pub struct Level(pub usize);

pub const BATTLEFIELD_BOUNDS_X: Vec2 = Vec2::new(-6.2, 6.2);
pub const BATTLEFIELD_BOUNDS_Y: Vec2 = Vec2::new(-6.4, 7.0);

pub const DEFAULT_ROUND_TIME: f32 = 5.0;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_war_machine_graphics)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Battle)
                    .label("update")
                    .with_system(war_machine_move_and_attack)
                    .with_system(sheep_attack)
                    .with_system(update_health_bars)
                    .with_system(remove_dead_sheep)
                    .with_system(remove_dead_war_machines)
                    .with_system(sheep::wander)
                    .with_system(sheep::wobble_sheep)
                    .with_system(sheep::update_sheep_ordering)
                    .with_system(update_round_time)
                    .with_system(animate_war_machine)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Battle)
                    .after("update")
                    .with_system(bounds_check)
                    .with_system(check_end_battle)
                    .into(),
            )
            .add_enter_system_set(
                GameState::Battle,
                ConditionSet::new()
                    .with_system(setup_level1.run_if_resource_equals::<Level>(Level(1)))
                    .with_system(add_health_bars_to_sheep)
                    .into(),
            )
            .add_exit_system_set(
                GameState::Battle,
                ConditionSet::new()
                    .with_system(despawn_entities_with_component::<UnloadOnExit>)
                    .into(),
            );
    }
}

fn add_health_bars_to_sheep(mut commands: Commands, sheep_q: Query<Entity, With<sheep::Sheep>>) {
    sheep_q.for_each(|sheep| create_sheep_hp_bar(sheep, &mut commands));
}

fn war_machine_move_and_attack(
    mut sheep_q: Query<(&mut Health, &mut Transform), (With<sheep::Sheep>, Without<WarMachine>)>,
    mut war_machines_q: Query<
        (
            &Speed,
            &mut Transform,
            &Attack,
            &BehaviourType,
            &mut TextureAtlasSprite,
        ),
        (With<WarMachine>, Without<sheep::Sheep>),
    >,
    time: Res<Time>,
) {
    for (speed, mut wm_transform, attack, behaviour_type, mut sprite) in war_machines_q.iter_mut() {
        // Calculate the distance between the sheep and the current war machine
        let mut sheep = sheep_q
            .iter_mut()
            .filter(|(_, sheep_transform)| {
                wm_transform
                    .translation
                    .truncate()
                    .distance(sheep_transform.translation.truncate())
                    <= attack.spotting_range
            })
            .collect::<Vec<_>>();

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

        // Find the closest sheep
        if let Some((ref mut sheep_health, sheep_transform)) = sheep.get_mut(0) {
            let difference =
                sheep_transform.translation.truncate() - wm_transform.translation.truncate();

            // If the sheep is close enough, attack it
            if difference.length() <= attack.attack_range {
                sheep_health.current -= attack.attack_damage;
            }

            // Move towards the sheep depending on the `behaviour_type`
            match behaviour_type {
                BehaviourType::ChasingClosest => {
                    let direction = difference.normalize_or_zero();

                    if difference.length() >= attack.attack_range * 0.5 {
                        sprite.flip_x = direction.x <= 0.0;

                        wm_transform.translation +=
                            direction.extend(0.0) * speed.0 * time.delta_seconds();
                    }
                }
            }
        }
    }
}

fn sheep_attack(
    mut sheep_q: Query<(&mut Transform, &Attack), (With<sheep::Sheep>, Without<WarMachine>)>,
    mut war_machines_q: Query<
        (&mut Health, &mut Transform),
        (With<WarMachine>, Without<sheep::Sheep>),
    >,
) {
    for (sheep_transform, sheep_attack) in sheep_q.iter_mut() {
        // Calculate the distance between the sheep and the current war machine
        let mut war_machines = war_machines_q
            .iter_mut()
            .filter(|(_, wm_transform)| {
                sheep_transform
                    .translation
                    .truncate()
                    .distance(wm_transform.translation.truncate())
                    <= sheep_attack.spotting_range
            })
            .collect::<Vec<_>>();

        war_machines.sort_by(|(_, transform1), (_, transform2)| {
            sheep_transform
                .translation
                .truncate()
                .distance(transform1.translation.truncate())
                .partial_cmp(
                    &sheep_transform
                        .translation
                        .truncate()
                        .distance(transform2.translation.truncate()),
                )
                .unwrap()
        });

        // Find the closest war machine
        if let Some((ref mut wm_health, wm_transform)) = war_machines.get_mut(0) {
            let difference =
                wm_transform.translation.truncate() - sheep_transform.translation.truncate();

            // If the sheep is close enough, sheep_attack it
            if difference.length() <= sheep_attack.attack_range {
                wm_health.current -= sheep_attack.attack_damage;
            }
        }
    }
}

fn remove_dead_sheep(
    mut commands: Commands,
    sheep_q: Query<(Entity, &mut Health), (With<sheep::Sheep>, Changed<Health>)>,
) {
    for (sheep, health) in sheep_q.iter() {
        if health.current <= 0.0 {
            commands.entity(sheep).despawn_recursive();
        }
    }
}

fn remove_dead_war_machines(
    mut commands: Commands,
    war_machines_q: Query<(Entity, &mut Health), (With<WarMachine>, Changed<Health>)>,
) {
    for (war_machine, health) in war_machines_q.iter() {
        if health.current <= 0.0 {
            commands.entity(war_machine).despawn_recursive();
        }
    }
}

/// Increases round time and renders it to screen
fn update_round_time(
    mut commands: Commands,
    time: Res<Time>,
    mut battle_timer: ResMut<BattleTimer>,
    ascii_sheet: Res<AsciiSheet>,
    query: Query<Entity, With<BattleTimerText>>,
) {
    battle_timer.0.tick(time.delta());

    // Remove old timer
    query.for_each(|timer_text| commands.entity(timer_text).despawn_recursive());

    let elapsed = battle_timer.0.duration().as_secs_f32() - battle_timer.0.elapsed_secs();
    let round_timer = write_text(
        &mut commands,
        &ascii_sheet,
        Vec2::new(-0.5, -3.8).extend(50.0),
        Color::WHITE,
        format!("{elapsed:.2}").as_str(),
    );
    commands
        .entity(round_timer)
        .insert(BattleTimerText)
        .insert(UnloadOnExit);
}

fn check_end_battle(
    mut commands: Commands,
    battle_timer: Res<BattleTimer>,
    sheep_q: Query<Entity, (With<sheep::Sheep>, Without<WarMachine>)>,
    war_machines_q: Query<Entity, (Without<sheep::Sheep>, With<WarMachine>)>,
    mut _level: ResMut<Level>,
) {
    if battle_timer.0.just_finished() || sheep_q.is_empty() || war_machines_q.is_empty() {
        // TODO: should show battle report, before going straight to Herding
        commands.insert_resource(NextState(GameState::Herding));
        commands.remove_resource::<BattleTimer>();

        // Increase level if all war machines are dead
        // Currently commented, not all levels are defined
        //if war_machines_q.is_empty() {
        //level.0 += 1;
        //}
    }
}

fn setup_level1(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    robot_texture: Res<war_machines::RobotSprites>,
) {
    // Spawn red battlefield to distinguish from the pen
    // TODO: should be replaced with a proper asset
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("SheepFarmBehind.png"),
            sprite: Sprite {
                color: Color::ORANGE_RED,
                custom_size: Some(Vec2::new(550.0, 300.0) / 16.0),
                ..default()
            },
            transform: Transform {
                translation: Vec3::splat(0.0),
                ..default()
            },
            ..default()
        })
        .insert(UnloadOnExit)
        .insert(Name::from("Battlefield"));

    // Add round timer
    commands.insert_resource(BattleTimer(Timer::from_seconds(DEFAULT_ROUND_TIME, false)));

    // Spawn a single war machine
    let mut rng = thread_rng();
    let transform = Transform::from_translation(Vec3::new(
        rng.gen_range(BATTLEFIELD_BOUNDS_X.x..=BATTLEFIELD_BOUNDS_X.y),
        rng.gen_range(BATTLEFIELD_BOUNDS_Y.x..=BATTLEFIELD_BOUNDS_Y.y),
        10.0,
    ));

    let war_machine = new_war_machine(&mut commands, &robot_texture, transform);
    commands
        .entity(war_machine)
        .insert(Speed(4.0))
        .insert(Health {
            current: 30.0,
            max: 30.0,
        })
        .insert(Attack {
            attack_damage: 1.0,
            attack_range: 1.0,
            spotting_range: 1000.0,
        })
        .insert(BehaviourType::ChasingClosest);
}
