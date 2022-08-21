use bevy::prelude::*;

use rand::{thread_rng, Rng};

use crate::{drag::Drag, ScreenToWorld};

pub struct SheepPlugin;

impl Plugin for SheepPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_sheep)
            .add_system_to_stage(CoreStage::PreUpdate, select_sheep)
            .add_system_to_stage(CoreStage::PostUpdate, bounds_check)
            .add_system(drop_sheep)
            .add_system(update_sheep)
            .add_system(wander)
            .add_system(wobble_sheep)
            .add_system(shrink_sheep_on_drop)
            .add_system(update_sheep_ordering);
    }
}

const PEN_BOUNDS_X: Vec2 = Vec2::new(-6.2, 6.2);
const PEN_BOUNDS_Y: Vec2 = Vec2::new(-6.4, 7.0);

const COUNT_INIT_SHEEP: usize = 10;

const WANDER_TIME_SECS: f32 = 3.0;
const IDLE_TIME_SECS: f32 = 5.0;
const MAX_WANDER_TIME_DEVIANCE_PERCENT: f32 = 0.2;

const SHEEP_WANDER_SPEED: f32 = 1.0;
const SHEEP_ROT_AMPLITUDE_RAD: f32 = 10.0 * (std::f32::consts::PI / 180.0);
const SHEEP_ROT_WAVELENGTH_SECS_INV: f32 = 8.0;
const SHEEP_WOBBLE_DRAGGED_SECS_INV: f32 = 24.0;

#[derive(Component, Default)]
pub struct Sheep {
    // In future we can put all the sheep traits here
    state: u8,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum WanderState {
    Wandering,
    Idling,
}

#[derive(Component)]
pub struct Wander {
    wander_time_s: f32,
    idle_time_s: f32,
    time_deviance: f32,
    state: WanderState,
    timer: Timer,
    wander_dir: Vec2,
}

impl Wander {
    fn new(wander_time_s: f32, idle_time_s: f32, time_deviance: f32, state: WanderState) -> Self {
        let mut rng = thread_rng();

        Self {
            wander_time_s,
            idle_time_s,
            time_deviance,
            state,
            timer: Timer::from_seconds(
                match state {
                    WanderState::Wandering => {
                        wander_time_s * (1.0 + rng.gen_range(-time_deviance..=time_deviance))
                    }
                    WanderState::Idling => {
                        idle_time_s * (1.0 + rng.gen_range(-time_deviance..=time_deviance))
                    }
                },
                false,
            ),
            wander_dir: Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0))
                .normalize_or_zero(),
        }
    }
}

fn spawn_sheep(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    transform: Transform,
    sheep: Sheep,
) -> Entity {
    let mut transform = transform;
    transform.rotation = Quat::IDENTITY;
    commands
        .spawn_bundle(SpriteBundle {
            transform,
            texture: asset_server.load("BaseSheep.png"),
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::splat(1.0)),
                ..default()
            },
            ..default()
        })
        .insert(sheep)
        .insert(Wander::new(
            WANDER_TIME_SECS,
            IDLE_TIME_SECS,
            MAX_WANDER_TIME_DEVIANCE_PERCENT,
            match rand::random() {
                true => WanderState::Wandering,
                false => WanderState::Idling,
            },
        ))
        .insert(Bounds {
            x: (PEN_BOUNDS_X.x, PEN_BOUNDS_X.y),
            y: (PEN_BOUNDS_Y.x, PEN_BOUNDS_Y.y),
        })
        .insert(Speed(SHEEP_WANDER_SPEED))
        .id()
}

#[derive(Component)]
pub struct SheepParent;

fn init_sheep(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut rng = thread_rng();

    let mut sheep = Vec::with_capacity(COUNT_INIT_SHEEP);
    for i in 0..COUNT_INIT_SHEEP {
        let new_sheep = spawn_sheep(
            &mut commands,
            &asset_server,
            Transform {
                translation: Vec3::new(
                    rng.gen_range(PEN_BOUNDS_X.x..=PEN_BOUNDS_X.y),
                    rng.gen_range(PEN_BOUNDS_Y.x..=PEN_BOUNDS_Y.y),
                    10.0,
                ),
                ..default()
            },
            Sheep::default(),
        );

        sheep.push(
            commands
                .entity(new_sheep)
                .insert(Name::from(format!("Sheep_{i}")))
                .id(),
        );
    }

    commands
        .spawn_bundle(SpatialBundle::default())
        .insert(SheepParent)
        .insert(Name::from("SheepParent"))
        .push_children(&sheep);
}

fn select_sheep(
    mut commands: Commands,
    sheep_q: Query<(Entity, &Transform), With<Sheep>>,
    mouse_btn: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.get_primary().unwrap();

    if mouse_btn.just_pressed(MouseButton::Left) {
        if let Some(mouse_pos) = window.cursor_position() {
            // Convert screen coordinates to world coordinates
            let mouse_pos = mouse_pos.screen_to_world(windows, camera);

            // Detect sheep
            let mut sheep = sheep_q
                .iter()
                .filter(|(_, transform)| {
                    mouse_pos.distance(transform.translation.truncate()) <= transform.scale.x / 2.0
                })
                .collect::<Vec<_>>();

            sheep.sort_by(|(_, transform1), (_, transform2)| {
                mouse_pos
                    .distance(transform1.translation.truncate())
                    .partial_cmp(&mouse_pos.distance(transform2.translation.truncate()))
                    .unwrap()
            });

            if let Some((sheep, _)) = sheep.get(0) {
                commands.entity(*sheep).insert(Drag);
            }
        }
    } else if mouse_btn.just_released(MouseButton::Left) {
        for (sheep, _) in &sheep_q {
            commands.entity(sheep).remove::<Drag>();
        }
    }
}

fn drop_sheep(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    dropped: RemovedComponents<Drag>,
    sheep: Query<(Entity, &Sheep, &Transform)>,
    sheep_parent: Query<Entity, With<SheepParent>>,
) {
    for drop in dropped.iter() {
        if let Ok((_, sheep_component, dropped_transform)) = sheep.get(drop) {
            if let Some((collided, _, collided_transform)) = sheep
                .iter()
                .filter(|(_, _, transform)| {
                    transform
                        .translation
                        .truncate()
                        .distance(dropped_transform.translation.truncate())
                        <= transform.scale.x
                })
                .find(|(entity, _, _)| entity.id() != drop.id())
            {
                commands.entity(drop).despawn_recursive();
                commands.entity(collided).despawn_recursive();

                let new_sheep = spawn_sheep(
                    &mut commands,
                    &asset_server,
                    *collided_transform,
                    Sheep {
                        // In here we would have the actual trait mutation / combination rather
                        // than just incrementing a state value
                        state: sheep_component.state + 1,
                    },
                );

                commands.entity(sheep_parent.single()).add_child(new_sheep);
            }
        }
    }
}

#[derive(Component)]
pub struct Speed(f32);

fn wander(
    mut sheeps: Query<(Entity, &mut Wander, &mut Transform, &Speed), (With<Sheep>, Without<Drag>)>,
    time: Res<Time>,
) {
    for (entity, mut sheep, mut transform, speed) in sheeps.iter_mut() {
        sheep.timer.tick(time.delta());

        if sheep.timer.just_finished() {
            *sheep = Wander::new(
                sheep.wander_time_s,
                sheep.idle_time_s,
                sheep.time_deviance,
                match sheep.state {
                    WanderState::Wandering => {
                        transform.rotation = Quat::IDENTITY;
                        WanderState::Idling
                    }
                    WanderState::Idling => WanderState::Wandering,
                },
            );
        }

        if sheep.state == WanderState::Wandering {
            transform.translation += sheep.wander_dir.extend(0.0) * speed.0 * time.delta_seconds();
            transform.rotation = Quat::from_rotation_z(
                SHEEP_ROT_AMPLITUDE_RAD
                    * (entity.id() as f32
                        + sheep.timer.elapsed_secs() as f32 * SHEEP_ROT_WAVELENGTH_SECS_INV)
                        .sin(),
            );
        }
    }
}

#[derive(Component)]
struct Bounds {
    x: (f32, f32),
    y: (f32, f32),
}

// This system (and `Bounds` component) are pretty generic and should probably be moved to a
// different module if another type of entity ends up using it
fn bounds_check(mut transforms: Query<(&mut Transform, &Bounds), Changed<Transform>>) {
    for (mut transform, bounds) in transforms.iter_mut() {
        if transform.translation.y > bounds.y.1 {
            transform.translation.y = bounds.y.1;
        } else if transform.translation.y < bounds.y.0 {
            transform.translation.y = bounds.y.0;
        }

        if transform.translation.x > bounds.x.1 {
            transform.translation.x = bounds.x.1;
        } else if transform.translation.x < bounds.x.0 {
            transform.translation.x = bounds.x.0
        }
    }
}

// Wobble when they're picked up
fn wobble_sheep(mut transforms: Query<&mut Transform, With<Drag>>, time: Res<Time>) {
    for mut transform in transforms.iter_mut() {
        transform.scale = Vec2::splat(1.2).extend(0.0);
        transform.rotation = Quat::from_rotation_z(
            SHEEP_ROT_AMPLITUDE_RAD
                * (time.seconds_since_startup() as f32 * SHEEP_WOBBLE_DRAGGED_SECS_INV).sin(),
        );
    }
}

fn shrink_sheep_on_drop(
    mut sheeps: Query<&mut Transform, With<Sheep>>,
    dropped: RemovedComponents<Drag>,
) {
    for dropped in dropped.iter() {
        if let Ok(mut transform) = sheeps.get_mut(dropped) {
            transform.scale = Vec3::splat(1.0);
            transform.rotation = Quat::IDENTITY;
        }
    }
}

// Update the z axis of all the sheep so that they are ordered closer to the camera if they are
// further down the screen
// Some magic numbers in here*, end of the day, I'm getting tired, sorry!
//
// *refer to `main.rs#L1`
fn update_sheep_ordering(
    mut q: Query<(Entity, &mut Transform), (With<Sheep>, Changed<Transform>)>,
    dragged: Query<&Drag>,
) {
    for (entity, mut transform) in q.iter_mut() {
        transform.translation.z = match dragged.get(entity) {
            Ok(_) => 9.9,
            Err(_) => {
                9.9 - ((transform.translation.y - PEN_BOUNDS_Y.x).abs()
                    / (PEN_BOUNDS_Y.y - PEN_BOUNDS_Y.x).abs())
                    * 9.79
            }
        }
    }
}

fn update_sheep(mut q: Query<(&mut Sprite, &Sheep), Changed<Sheep>>) {
    for (mut sprite, sheep) in q.iter_mut() {
        sprite.color = match sheep.state {
            0 => Color::rgb(1.0, 1.0, 1.0),
            1 => Color::rgb(1.0, 0.0, 0.0),
            2 => Color::rgb(0.0, 1.0, 0.0),
            3 => Color::rgb(0.0, 0.0, 1.0),
            _ => Color::PURPLE,
        }
    }
}
