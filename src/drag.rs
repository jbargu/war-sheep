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
    if let Some(mouse_pos) = window.cursor_position() {
        let mouse_pos = mouse_pos.screen_to_world(windows, camera);
        for mut transform in q.iter_mut() {
            transform.translation = mouse_pos.extend(transform.translation.z);
        }
    }
}

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(drag);
    }
}
