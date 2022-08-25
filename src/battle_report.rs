use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::utils::UnloadOnExit;

use crate::ui::{write_text, AsciiSheet};
use crate::utils::despawn_entities_with_component;
use crate::{GameState, NewGame};

#[derive(Default, Eq, PartialEq)]
pub enum BattleStatus {
    #[default]
    StillPlaying,
    Victory,
    GameOver,
    Draw,
}

impl BattleStatus {}

#[derive(Default)]
pub struct BattleResult {
    pub battle_status: BattleStatus,
    pub war_machines_slain: usize,
    pub sheep_slain: usize,
    pub level_reward_sheep_gained: usize,
}

impl BattleResult {
    /// Return the final text depending on the BattleStatus
    pub fn status_text(&self) -> String {
        match self.battle_status {
            BattleStatus::Victory => {
                format!(
                    "            You won!\n\n  Baaaa bye angry war machines!\n\n\n       You gain {} new sheep!\n\n\n     Press SPACE to continue.",
                    self.level_reward_sheep_gained
                )
            }
            BattleStatus::GameOver => {
                format!("           Game over! :(\n\n\n   Press SPACE to start a new game!")
            }
            BattleStatus::Draw => {
                format!("           Time ran out!\n\nYou can face the war machines again\n  until all of your sheep are gone!\n\n\n     Press SPACE to continue.")
            }
            _ => {
                format!("Something unexpected happen. You should still be playing the game!")
            }
        }
    }
}

/// If this resource is present, `.0` many sheep will be added to the pen
pub struct LevelReward(pub usize);

pub struct BattleReportPlugin;

impl Plugin for BattleReportPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::BattleReport)
                .with_system(keyboard_input)
                .into(),
        )
        .add_enter_system(GameState::BattleReport, setup_result_text)
        .add_exit_system_set(
            GameState::BattleReport,
            ConditionSet::new()
                .with_system(despawn_entities_with_component::<UnloadOnExit>)
                .into(),
        );
    }
}

fn setup_result_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ascii_sheet: Res<AsciiSheet>,
    battle_result: Res<BattleResult>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("BattleReportBackground.png"),
            sprite: Sprite {
                color: Color::ORANGE_RED,
                custom_size: Some(Vec2::new(550.0, 300.0) / 16.0),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 100.0),
                ..default()
            },
            ..default()
        })
        .insert(UnloadOnExit)
        .insert(Name::from("BattleReportBackground"));

    let text = write_text(
        &mut commands,
        &ascii_sheet,
        Vec2::new(-4.0, 2.0).extend(120.0),
        Color::WHITE,
        &battle_result.status_text(),
    );
    commands.entity(text).insert(UnloadOnExit);

    if battle_result.battle_status == BattleStatus::Victory {
        commands.insert_resource(LevelReward(battle_result.level_reward_sheep_gained));
    } else if battle_result.battle_status == BattleStatus::GameOver {
        commands.insert_resource(NewGame);
    }

    commands.remove_resource::<BattleResult>();
}

fn keyboard_input(mut commands: Commands, keys: ResMut<Input<KeyCode>>) {
    if keys.just_released(KeyCode::Space) {
        commands.insert_resource(NextState(GameState::Herding));
    }
}
