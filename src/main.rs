use bevy::prelude::*;
use bevy::render::texture::ImageSettings;

mod debug;
mod drag;
mod sheep;

const RESOLUTION: f32 = 16.0 / 9.0;
const WINDOW_HEIGHT: f32 = 900.0;

trait ScreenToWorld {
    // NOTE: if we end up using multiple screens, this will have to be adjusted
    fn screen_to_world(
        &self,
        windows: Res<Windows>,
        camera: Query<(&Camera, &GlobalTransform)>,
    ) -> Self;
}

impl ScreenToWorld for Vec2 {
    fn screen_to_world(
        &self,
        windows: Res<Windows>,
        camera: Query<(&Camera, &GlobalTransform)>,
    ) -> Self {
        // Logic here courtesy of bevy cheat book
        // https://bevy-cheatbook.github.io/cookbook/cursor2world.html
        let window = windows.get_primary().unwrap();
        let (camera, camera_transform) = camera.single();
        let win_size = Vec2::new(window.width() as f32, window.height() as f32);
        let ndc = (*self / win_size) * 2.0 - Vec2::ONE;
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
        let world_pos: Vec2 = ndc_to_world.project_point3(ndc.extend(-1.0)).truncate();
        world_pos
    }
}

fn main() {
    App::new()
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(WindowDescriptor {
            width: WINDOW_HEIGHT * RESOLUTION,
            height: WINDOW_HEIGHT,
            title: "War Sheep".to_string(),
            resizable: false, // I am using tiling WM so this is just easier for time being, can
            ..default()       // adjust later
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(debug::DebugPlugin)
        .add_plugin(sheep::SheepPlugin)
        .add_plugin(drag::DragPlugin)
        .add_startup_system(spawn_camera)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.02,
            ..default()
        },
        ..default()
    });
}
