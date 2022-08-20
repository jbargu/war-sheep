use bevy::prelude::*;

use crate::ScreenToWorld;

#[derive(Component)]
pub struct Drag;

pub fn drag(
    mut q: Query<&mut Transform, With<Drag>>,
    windows: Res<Windows>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.get_primary().unwrap();
    let mouse_pos = window
        .cursor_position()
        .unwrap()
        .screen_to_world(windows, camera);

    for mut transform in q.iter_mut() {
        transform.translation = mouse_pos.extend(0.0);
    }
}

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(drag);
    }
}
