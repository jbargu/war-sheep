use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{drag::Drag, ScreenToWorld};

pub struct SheepPlugin;

impl Plugin for SheepPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_sheep)
            .add_system_to_stage(CoreStage::PreUpdate, select_sheep)
            .add_system(drop_sheep)
            .add_system(update_sheep);
    }
}

const X_MAX_POS_OFFSET: f32 = 10.0;
const Y_MAX_POS_OFFSET: f32 = 6.0;
const COUNT_INIT_SHEEP: usize = 10;

#[derive(Component, Default)]
pub struct Sheep {
    // In future we can put all the sheep traits here
    state: u8,
}

fn spawn_sheep(commands: &mut Commands, transform: Transform, sheep: Sheep) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            transform,
            sprite: Sprite {
                color: Color::WHITE,
                ..default()
            },
            ..default()
        })
        .insert(sheep)
        .id()
}

#[derive(Component)]
pub struct SheepParent;

fn init_sheep(mut commands: Commands) {
    let mut rng = thread_rng();

    let mut sheep = Vec::with_capacity(COUNT_INIT_SHEEP);
    for i in 0..=COUNT_INIT_SHEEP {
        let new_sheep = spawn_sheep(
            &mut commands,
            Transform {
                translation: Vec3::new(
                    rng.gen_range(-X_MAX_POS_OFFSET..=X_MAX_POS_OFFSET),
                    rng.gen_range(-Y_MAX_POS_OFFSET..=Y_MAX_POS_OFFSET),
                    0.0,
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

            if let Some((sheep, _)) = sheep.iter().next() {
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
                        .distance(dropped_transform.translation)
                        <= transform.scale.x
                })
                .filter(|(entity, _, _)| entity.id() != drop.id())
                .next()
            {
                commands.entity(drop).despawn_recursive();
                commands.entity(collided).despawn_recursive();

                let new_sheep = spawn_sheep(
                    &mut commands,
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
