use bevy::prelude::*;

use rand::{thread_rng, Rng};

use crate::utils::{bounds_check, Bounds, Health, Speed};
use crate::{drag::Drag, GameState, ScreenToWorld};

pub struct SheepPlugin;

impl Plugin for SheepPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_sheep)
            // If you want the sheep to respawn after Battle, uncomment below, and comment above
            //app.add_system_set(SystemSet::on_enter(GameState::Herding).with_system(init_sheep))
            .add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .add_system_set(
                SystemSet::on_update(GameState::Herding)
                    .label("update")
                    .with_system(sheep_select)
                    .with_system(update_select_box)
                    .with_system(drop_sheep)
                    .with_system(wander)
                    .with_system(wobble_sheep)
                    .with_system(shrink_sheep_on_drop)
                    .with_system(update_sheep_ordering)
                    .with_system(keyboard_input),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Herding)
                    .after("update")
                    .with_system(bounds_check)
                    .with_system(update_sheep),
            )
            .add_system_to_stage(CoreStage::PreUpdate, grab_sheep);
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

const SHEEP_DEFAULT_HEALTH: f32 = 20.0;

#[derive(Component, Default)]
pub struct Sheep {
    // In future we can put all the sheep traits here
    col: f32,
    speed_mod: f32,
}

impl Sheep {
    fn from_col(col: f32) -> Self {
        Self { col, ..default() }
    }

    fn combine(&self, other: &Self) -> Self {
        let mut rng = thread_rng();
        Self {
            col: 0.1f32.max((self.col + other.col) / 2.0 + rng.gen_range(-0.1..=0.1)),
            speed_mod: 0.0f32.max(
                match rand::random() {
                    true => self.speed_mod,
                    false => other.speed_mod,
                } + rng.gen_range(0.0..=0.2),
            ),
        }
    }
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
    texture: &SheepSprites,
    transform: Transform,
    sheep: Sheep,
) -> Entity {
    let mut transform = transform;
    transform.rotation = Quat::IDENTITY;
    let sheep = commands
        .spawn_bundle(SpriteSheetBundle {
            transform,
            texture_atlas: texture.0.clone(),
            sprite: TextureAtlasSprite {
                index: 0,
                custom_size: Some(Vec2::new(1.0, 1.0)),
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
        .insert(Health {
            current: SHEEP_DEFAULT_HEALTH,
            max: SHEEP_DEFAULT_HEALTH,
        })
        .id();

    let head = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture.0.clone(),
            transform: Transform {
                translation: Vec2::ZERO.extend(0.001),
                ..default()
            },
            sprite: TextureAtlasSprite {
                index: 1,
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..default()
            },
            ..default()
        })
        .id();

    commands.entity(sheep).add_child(head);

    sheep
}

#[derive(Component)]
pub struct SheepParent;

fn init_sheep(mut commands: Commands, texture: Res<SheepSprites>) {
    let mut rng = thread_rng();

    let mut sheep = Vec::with_capacity(COUNT_INIT_SHEEP);
    for i in 0..COUNT_INIT_SHEEP {
        let new_sheep = spawn_sheep(
            &mut commands,
            &texture,
            Transform {
                translation: Vec3::new(
                    rng.gen_range(PEN_BOUNDS_X.x..=PEN_BOUNDS_X.y),
                    rng.gen_range(PEN_BOUNDS_Y.x..=PEN_BOUNDS_Y.y),
                    10.0,
                ),
                ..default()
            },
            Sheep::from_col(if rng.gen_range(0.0..=1.0) >= 0.2 {
                // White sheep more likely than black sheep
                rng.gen_range(0.8..=1.0)
            } else {
                rng.gen_range(0.1..=0.3)
            }),
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

fn grab_sheep(
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

#[derive(Component)]
struct Selected;

fn drop_sheep(
    mut commands: Commands,
    texture: Res<SheepSprites>,
    dropped: RemovedComponents<Drag>,
    sheep: Query<(Entity, &Sheep, &Transform)>,
    sheep_parent: Query<Entity, With<SheepParent>>,
) {
    for drop in dropped.iter() {
        if let Ok((_, sheep_component, dropped_transform)) = sheep.get(drop) {
            if let Some((collided, collided_sheep_component, collided_transform)) = sheep
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
                    &texture,
                    *collided_transform,
                    sheep_component.combine(collided_sheep_component),
                );

                commands.entity(sheep_parent.single()).add_child(new_sheep);
            }
        }
    }
}

#[derive(Component)]
struct Select;

// Would prefer to be called `select_sheep` but there was a previous system of that name (now
// changed to `grab_sheep`) and I didn't want to give confusing merge conflicts
/// Add the little select icon to the sheep when they're selected, this will also display their
/// stats in the future
fn sheep_select(
    mut commands: Commands,
    q: Query<Entity, (With<Sheep>, Added<Drag>)>,
    currently_selected: Query<Entity, With<Select>>,
    assets: Res<AssetServer>,
) {
    let mut added_this_frame = Vec::new();
    if !q.is_empty() {
        for entity in currently_selected.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }

    for entity in q.iter() {
        // NOTE: This needs some work. Namely, it shouldn't rotate with the sheep - but the only
        // way I can think of to achieve that would be to have the sheep's body sprite be a child of
        // the sheep object, which is some refactoring I don't want to do right now, but will have
        // to do at some point unless we can think of another way
        let select_box = commands
            .spawn_bundle(SpriteBundle {
                texture: assets.load("OutlineBox.png"),
                sprite: Sprite {
                    // TODO: Fix the size scaling issue
                    custom_size: Some(Vec2::splat(20.0) / 16.0),
                    ..default()
                },
                transform: Transform {
                    translation: Vec2::ZERO.extend(30.0),
                    ..default()
                },
                ..default()
            })
            .insert(Select)
            .insert(Name::from("SelectBox"))
            .id();
        commands.entity(entity).add_child(select_box);

        added_this_frame.push(select_box.id());
    }
}

// NOTE: This only works if we preserve the invariant that only one entity is being dragged at any
// given time.
fn update_select_box(mut q: Query<&mut Visibility, With<Select>>, dragged: Query<&Drag>) {
    if !dragged.is_empty() {
        for mut vis in q.iter_mut() {
            vis.is_visible = false;
        }
    } else {
        for mut vis in q.iter_mut() {
            vis.is_visible = true;
        }
    }
}

pub fn wander(
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

// Wobble when they're picked up
pub fn wobble_sheep(mut transforms: Query<&mut Transform, With<Drag>>, time: Res<Time>) {
    for mut transform in transforms.iter_mut() {
        transform.scale = Vec2::splat(1.2).extend(1.0);
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
pub fn update_sheep_ordering(
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

pub fn update_sheep(mut q: Query<(&mut TextureAtlasSprite, &mut Speed, &Sheep), Changed<Sheep>>) {
    for (mut sprite, mut speed, sheep) in q.iter_mut() {
        sprite.color = Color::WHITE * sheep.col;
        speed.0 = 1.0 + sheep.speed_mod;
    }
}

struct SheepSprites(Handle<TextureAtlas>);

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("BaseSheep.png");
    let atlas = TextureAtlas::from_grid_with_padding(
        image,
        Vec2::new(16.0, 16.0),
        2,
        1,
        Vec2::splat(2.0),
        Vec2::ZERO,
    );
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(SheepSprites(atlas_handle));
}

fn keyboard_input(mut keys: ResMut<Input<KeyCode>>, mut game_state: ResMut<State<GameState>>) {
    if keys.just_released(KeyCode::Key1) {
        game_state.set(GameState::Battle).unwrap();
        keys.reset(KeyCode::Key1);
    }
}
