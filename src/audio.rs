use bevy::{prelude::*, utils::HashMap};
use bevy_kira_audio::{AudioApp, AudioChannel};
use iyes_loopless::prelude::*;

use crate::animation::Animation;
use crate::GameState;

pub struct MusicChannel;

pub struct EffectsChannel;

pub struct AudioPlugin;

/// For readability.
const IMPOSSIBLE_ANIMATION_I: usize = usize::MAX;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_kira_audio::AudioPlugin)
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<EffectsChannel>()
            .add_startup_system(set_audio_channels_volume)
            .add_enter_system(GameState::Herding, play_herding_music)
            .add_exit_system(GameState::Herding, stop_herding_music)
            .add_enter_system(GameState::Battle, play_battle_music)
            .add_exit_system(GameState::Battle, stop_battle_music)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                animation_audio_playback.run_in_state(GameState::Battle),
            );
    }
}

pub fn set_audio_channels_volume(
    music_channel: Res<AudioChannel<MusicChannel>>,
    effects_channel: Res<AudioChannel<EffectsChannel>>,
) {
    music_channel.set_volume(1.0);
    effects_channel.set_volume(0.8);
}

pub fn play_herding_music(
    asset_server: Res<AssetServer>,
    music_channel: Res<AudioChannel<MusicChannel>>,
) {
    let battle_music = asset_server.load("audio/sheep_herding.mp3");
    music_channel.play_looped(battle_music);
}

pub fn stop_herding_music(music_channel: Res<AudioChannel<MusicChannel>>) {
    music_channel.stop();
}

pub fn play_battle_music(
    asset_server: Res<AssetServer>,
    music_channel: Res<AudioChannel<MusicChannel>>,
) {
    let battle_music = asset_server.load("audio/war_machines_attacking.mp3");
    music_channel.play_looped(battle_music);
}

pub fn stop_battle_music(music_channel: Res<AudioChannel<MusicChannel>>) {
    music_channel.stop();
}

/// Add this to a sprite, when want to play sound effects attached to certain animation indexes.
#[derive(Component)]
pub struct AnimationAudioPlayback {
    pub animation_name: String,
    pub effects: HashMap<usize, String>,
    pub last_played: Option<usize>,
}

impl AnimationAudioPlayback {
    pub fn new(animation_name: String, effects: HashMap<usize, String>) -> Self {
        Self {
            animation_name,
            effects,
            last_played: None,
        }
    }
}

pub fn animation_audio_playback(
    mut commands: Commands,
    mut query: Query<(Entity, &Animation, &mut AnimationAudioPlayback)>,
    effects_channel: Res<AudioChannel<EffectsChannel>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, animation, mut state_effects) in query.iter_mut() {
        if animation.current_animation.as_ref() != Some(&state_effects.animation_name) {
            commands.entity(entity).remove::<AnimationAudioPlayback>();

            continue;
        }

        if let Some(_) = &animation.current_animation {
            if let Some(audio_file) = state_effects.effects.get(&animation.current_frame) {
                if state_effects.last_played.unwrap_or(IMPOSSIBLE_ANIMATION_I)
                    != animation.current_frame
                {
                    effects_channel.play(asset_server.load(audio_file));
                    state_effects.last_played = Some(animation.current_frame);
                }
            }
        }
    }
}
