use crate::battle::Level;
use crate::battle_report::LevelReward;
use bevy::prelude::*;
use iyes_loopless::prelude::*;

use rand::{thread_rng, Rng};

use crate::ui::{write_text, AsciiSheet};
use crate::utils::{bounds_check, Attack, Bounds, Health, Speed, UnloadOnExit};
use crate::{drag::Drag, GameState, NewGame, ScreenToWorld};

pub struct SheepPlugin;

impl Plugin for SheepPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system_set(
            GameState::Herding,
            ConditionSet::new()
                .with_system(init_new_game.run_if_resource_exists::<NewGame>())
                .with_system(add_level_reward_sheep.run_if_resource_exists::<LevelReward>())
                .with_system(setup_ui)
                .into(),
        )
        .add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
        .add_system_to_stage(
            CoreStage::PreUpdate,
            grab_sheep.run_in_state(GameState::Herding),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Herding)
                .with_system(sheep_select)
                .with_system(update_select_box)
                .with_system(remove_selected_text)
                .with_system(drop_sheep)
                .with_system(wander)
                .with_system(wobble_sheep)
                .with_system(shrink_sheep_on_drop)
                .with_system(update_sheep_ordering)
                .with_system(keyboard_input)
                .into(),
        )
        .add_system_to_stage(
            CoreStage::PostUpdate,
            bounds_check.run_in_state(GameState::Herding),
        );
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
const SHEEP_DEFAULT_ATTACK: Attack = Attack {
    attack_damage: 0.2,
    attack_range: 1.0,
    spotting_range: 100.0,
};

#[derive(Copy, Clone)]
pub struct SheepLevels {
    base: usize,
    spear: usize,
    tank: usize,
    medic: usize,
}

impl Default for SheepLevels {
    fn default() -> Self {
        Self {
            base: 1,
            spear: 0,
            tank: 0,
            medic: 0,
        }
    }
}

impl std::ops::Add<Self> for SheepLevels {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            base: self.base + rhs.base,
            spear: self.spear + rhs.spear,
            tank: self.tank + rhs.tank,
            medic: self.medic + rhs.medic,
        }
    }
}

#[derive(Component, Default)]
pub struct Sheep {
    // In future we can put all the sheep traits here
    color: f32,
    levels: SheepLevels,
}

impl Sheep {
    fn from_col(color: f32) -> Self {
        Self { color, ..default() }
    }

    fn combine(&self, other: &Self) -> Self {
        let mut rng = thread_rng();
        Self {
            color: 0.1f32.max((self.color + other.color) / 2.0 + rng.gen_range(-0.1..=0.1)),
            levels: self.levels + other.levels,
        }
    }

    /// TODO: Currently we sum all the levels times the SHEEP_DEFAULT_ATTACK to get the
    /// `attack_damage`. Should probably be modified based on the different trait + combine RNG.
    pub fn sum_levels(&self) -> f32 {
        (self.levels.base + self.levels.spear + self.levels.tank + self.levels.medic) as f32
    }

    /// If this component is attached to the `sheep` entity, it will attack the nearest war
    /// machine.
    pub fn attack_component(&self) -> Attack {
        Attack {
            attack_damage: SHEEP_DEFAULT_ATTACK.attack_damage * (self.sum_levels() + 1.0) / 2.0,
            attack_range: SHEEP_DEFAULT_ATTACK.attack_range
                * ((self.sum_levels() / 2.0).log2() + 0.2),
            spotting_range: SHEEP_DEFAULT_ATTACK.spotting_range
                * ((self.sum_levels()).log2() + 1.0),
        }
    }

    /// Diminishing speed
    pub fn speed_component(&self) -> Speed {
        Speed(SHEEP_WANDER_SPEED * (self.sum_levels()).log2() + 1.0)
    }

    pub fn health_component(&self) -> Health {
        let hp = SHEEP_DEFAULT_HEALTH * (self.sum_levels());
        Health::new(hp)
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

    let attack = sheep.attack_component();
    let speed = sheep.speed_component();
    let health = sheep.health_component();

    let sheep = commands
        .spawn_bundle(SpriteSheetBundle {
            transform,
            texture_atlas: texture.0.clone(),
            sprite: TextureAtlasSprite {
                index: 1,
                custom_size: Some(Vec2::new(20.0, 19.0) / 16.0),
                color: Color::WHITE * sheep.color,
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
        .insert(speed)
        .insert(health)
        .insert(attack)
        .id();

    let head = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture.0.clone(),
            transform: Transform {
                translation: Vec2::ZERO.extend(0.001),
                ..default()
            },
            sprite: TextureAtlasSprite {
                index: 2,
                custom_size: Some(Vec2::new(20.0, 19.0) / 16.0),
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

fn init_new_game(
    mut commands: Commands,
    texture: Res<SheepSprites>,
    sheep_parent_q: Query<Entity, With<SheepParent>>,
) {
    // Remove old sheep parents
    sheep_parent_q.for_each(|sheep_parent| commands.entity(sheep_parent).despawn_recursive());

    let sheep = spawn_n_sheep(&mut commands, texture, COUNT_INIT_SHEEP);

    commands
        .spawn_bundle(SpatialBundle::default())
        .insert(SheepParent)
        .insert(Name::from("SheepParent"))
        .push_children(&sheep);

    commands.remove_resource::<NewGame>();
    commands.insert_resource(Level(4));
}

fn add_level_reward_sheep(
    mut commands: Commands,
    texture: Res<SheepSprites>,
    level_reward: Res<LevelReward>,
    sheep_parent: Query<Entity, With<SheepParent>>,
) {
    let sheep = spawn_n_sheep(&mut commands, texture, level_reward.0);

    commands.entity(sheep_parent.single()).push_children(&sheep);
    commands.remove_resource::<LevelReward>();
}

fn setup_ui(mut commands: Commands, ascii_sheet: Res<AsciiSheet>, level: Res<Level>) {
    let start_battle_text = write_text(
        &mut commands,
        &ascii_sheet,
        Vec2::new(-2.6, -3.8).extend(50.0),
        Color::WHITE,
        "Press SPACE to fight!",
    );

    let lvl_string = level.0;
    let level_text = write_text(
        &mut commands,
        &ascii_sheet,
        Vec2::new(3.8, 3.8).extend(50.0),
        Color::WHITE,
        format!("Lvl: {lvl_string}").as_str(),
    );

    commands.entity(start_battle_text).insert(UnloadOnExit);
    commands.entity(level_text).insert(UnloadOnExit);
}

fn spawn_n_sheep(
    commands: &mut Commands,
    texture: Res<SheepSprites>,
    num_sheep: usize,
) -> Vec<Entity> {
    let mut rng = thread_rng();

    let mut sheep = Vec::with_capacity(num_sheep);
    for i in 0..num_sheep {
        let new_sheep = spawn_sheep(
            commands,
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

    sheep
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

#[derive(Component)]
struct SelectedText;

// Would prefer to be called `select_sheep` but there was a previous system of that name (now
// changed to `grab_sheep`) and I didn't want to give confusing merge conflicts
/// Add the little select icon to the sheep when they're selected, this will also display their
/// stats in the future
fn sheep_select(
    mut commands: Commands,
    q: Query<(Entity, &Sheep), Added<Drag>>,
    currently_selected: Query<Entity, With<Select>>,
    selected_text: Query<Entity, With<SelectedText>>,
    assets: Res<AssetServer>,
    ascii_sheet: Res<AsciiSheet>,
) {
    let mut added_this_frame = Vec::new();
    if !q.is_empty() {
        for entity in currently_selected.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }

    for (entity, sheep) in q.iter() {
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
            .insert(UnloadOnExit)
            .insert(Name::from("SelectBox"))
            .id();
        commands.entity(entity).add_child(select_box);

        // Remove old text
        if !selected_text.is_empty() {
            selected_text.for_each(|text| commands.entity(text).despawn_recursive());
        }

        // Add new text
        let lvl_string = sheep.sum_levels();
        let sheep_stats = write_text(
            &mut commands,
            &ascii_sheet,
            Vec2::new(-1.0, 3.8).extend(50.0),
            Color::WHITE,
            format!("Sheep lvl: {lvl_string}").as_str(),
        );
        commands
            .entity(sheep_stats)
            .insert(SelectedText)
            .insert(UnloadOnExit);

        added_this_frame.push(select_box.id());
    }
}

/// Handles orphanes selected text when an entity is deselected
fn remove_selected_text(
    mut commands: Commands,
    selected_text_q: Query<Entity, With<SelectedText>>,
    selected_entity_q: Query<Entity, With<Select>>,
) {
    if selected_entity_q.is_empty() && !selected_text_q.is_empty() {
        commands
            .entity(selected_text_q.single())
            .despawn_recursive();
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

struct SheepSprites(Handle<TextureAtlas>);

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("BaseSheep.png");
    let atlas = TextureAtlas::from_grid_with_padding(
        image,
        Vec2::new(20.0, 19.0),
        5,
        1,
        Vec2::splat(2.0),
        Vec2::ZERO,
    );
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(SheepSprites(atlas_handle));
}

fn keyboard_input(
    mut commands: Commands,
    keys: ResMut<Input<KeyCode>>,
    sheep_q: Query<Entity, With<SheepParent>>,
) {
    if keys.just_released(KeyCode::Space) {
        commands.insert_resource(NextState(GameState::Battle));
    }

    // On `N` start a new game
    if keys.just_released(KeyCode::N) {
        sheep_q.for_each(|ent| commands.entity(ent).despawn_recursive());
        commands.insert_resource(NewGame);
        commands.insert_resource(NextState(GameState::Herding));
    }
}
