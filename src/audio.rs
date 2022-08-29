use bevy::{prelude::*, utils::HashMap};
use bevy_kira_audio::{AudioApp, AudioChannel, AudioSource};
use iyes_loopless::prelude::*;

use crate::GameState;

pub struct MusicChannel;

pub struct EffectsChannel;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_kira_audio::AudioPlugin)
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<EffectsChannel>()
            .add_startup_system(set_audio_channels_volume)
            .add_enter_system(GameState::Battle, play_battle_music)
            .add_exit_system(GameState::Battle, stop_battle_music);
    }
}

pub fn set_audio_channels_volume(
    music_channel: Res<AudioChannel<MusicChannel>>,
    effects_channel: Res<AudioChannel<EffectsChannel>>,
) {
    music_channel.set_volume(0.8);
    effects_channel.set_volume(1.0);
}

pub fn play_battle_music(
    asset_server: Res<AssetServer>,
    music_channel: Res<AudioChannel<MusicChannel>>,
) {
    let battle_music = asset_server.load("audio/war_machines_attacking.mp3");
    music_channel.play(battle_music);
}

pub fn stop_battle_music(music_channel: Res<AudioChannel<MusicChannel>>) {
    music_channel.stop();
}
