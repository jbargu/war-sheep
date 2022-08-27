// Sprite z-axis ordering
//
// 0    - background
// ...
//  |   - sheep
// ...
// 10
// 20   - foreground

#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy_simple_stat_bars::prelude::*;
use iyes_loopless::prelude::*;
use utils::{despawn_entities_with_component, UnloadOnExit};

mod animation;
mod battle;
mod battle_report;
mod debug;
mod drag;
mod sheep;
mod ui;
mod utils;

const RESOLUTION: f32 = 16.0 / 9.0;
const WINDOW_HEIGHT: f32 = 900.0;
const WINDOW_WIDTH: f32 = WINDOW_HEIGHT * RESOLUTION;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    MainMenu,
    Herding,
    Battle,
    BattleReport,
    Paused,
}

pub struct NewGame;

trait ScreenToWorld {
    fn screen_to_world(
        &self,
        windows: Res<Windows>,
        camera: Query<(&Camera, &GlobalTransform)>,
    ) -> Self;
}

impl ScreenToWorld for Vec2 {
    // NOTE: if we end up using multiple screens, this will have to be adjusted
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
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            title: "War Sheep".to_string(),
            resizable: false, // I am using tiling WM so this is just easier for time being, can
            ..default()       // adjust later
        })
        .insert_resource(battle::Level(1))
        .insert_resource(NewGame)
        .add_loopless_state(GameState::Herding)
        .add_plugins(DefaultPlugins)
        .add_plugin(animation::AnimationPlugin)
        .add_plugin(battle::war_machines::WarMachinePlugin)
        .add_plugin(debug::DebugPlugin)
        .add_plugin(sheep::SheepPlugin)
        .add_plugin(drag::DragPlugin)
        .add_plugin(battle::BattlePlugin)
        .add_plugin(battle_report::BattleReportPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(StatBarsPlugin)
        .add_startup_system(spawn_camera)
        .add_enter_system(GameState::Herding, spawn_farm_scene)
        .add_exit_system(
            GameState::Herding,
            despawn_entities_with_component::<UnloadOnExit>,
        )
        .run();
}

fn spawn_farm_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn farm backgrounds
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("SheepFarmBehind.png"),
            sprite: Sprite {
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
        .insert(Name::from("FarmBehind"));

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("SheepFarmInfront.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(550.0, 300.0) / 16.0),
                ..default()
            },
            transform: Transform {
                translation: Vec2::splat(0.0).extend(20.0),
                ..default()
            },
            ..default()
        })
        .insert(UnloadOnExit)
        .insert(Name::from("FarmFront"));
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
