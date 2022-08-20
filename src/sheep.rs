use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{drag::Drag, ScreenToWorld};

pub struct SheepPlugin;

impl Plugin for SheepPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_sheep).add_system(select_sheep);
    }
}

const X_MAX_POS_OFFSET: f32 = 10.0;
const Y_MAX_POS_OFFSET: f32 = 6.0;
const COUNT_INIT_SHEEP: usize = 10;

#[derive(Component)]
pub struct Sheep;

fn spawn_sheep(mut commands: Commands) {
    let mut rng = thread_rng();

    let mut sheep = Vec::with_capacity(COUNT_INIT_SHEEP);
    for i in 0..=COUNT_INIT_SHEEP {
        sheep.push(
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            rng.gen_range(-X_MAX_POS_OFFSET..=X_MAX_POS_OFFSET),
                            rng.gen_range(-Y_MAX_POS_OFFSET..=Y_MAX_POS_OFFSET),
                            0.0,
                        ),
                        ..default()
                    },
                    sprite: Sprite {
                        color: Color::rgb(1.0, 1.0, 1.0),
                        ..default()
                    },
                    ..default()
                })
                .insert(Sheep)
                .insert(Name::from(format!("Sheep_{i}")))
                .id(),
        );
    }

    commands
        .spawn_bundle(SpatialBundle::default())
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
