use crate::GameState;
use bevy::prelude::*;
use bevy::utils::HashMap;
use iyes_loopless::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::Last,
            ConditionSet::new()
                .run_in_state(GameState::Battle)
                .with_system(animate)
                .into(),
        );
    }
}

#[derive(Component)]
pub struct Animation {
    animations: HashMap<String, Sheet>,
    pub current_animation: Option<String>,
    current_frame: usize,
    pub timer: Timer,
    pub flip_x: bool,
    pub played_once: bool,
}

#[derive(Clone)]
pub struct Sheet {
    pub atlas_handle: Handle<TextureAtlas>,
    pub length: usize,
    pub repeating: bool,
}

impl Animation {
    pub fn new(duration: f32, animations: HashMap<String, Sheet>) -> Self {
        Self {
            animations,
            current_animation: None,
            current_frame: 0,
            timer: Timer::from_seconds(duration, false),
            flip_x: false,
            played_once: false,
        }
    }

    pub fn play(&mut self, name: &str, repeating: bool) {
        self.current_animation = Some(name.to_owned());
        self.current_frame = 0;
        self.flip_x = false;
        self.played_once = false;
        self.timer.reset();
        self.timer.unpause();
        self.timer.set_repeating(repeating);
    }

    pub fn is_repeating(&self) -> bool {
        if let Some(curr) = &self.current_animation {
            if let Some(sheet) = self.animations.get(curr) {
                return sheet.repeating;
            }
        }
        false
    }

    pub fn is_last_frame(&self) -> bool {
        if let Some(curr) = &self.current_animation {
            if let Some(sheet) = self.animations.get(curr) {
                return self.current_frame + 1 >= sheet.length;
            }
        }
        false
    }

    pub fn has_finished(&self) -> bool {
        self.played_once
    }
}

/// Cycles all animations
pub fn animate(
    time: Res<Time>,
    mut query: Query<(
        &mut Animation,
        &mut TextureAtlasSprite,
        &mut Handle<TextureAtlas>,
    )>,
) {
    for (mut animation, mut sprite, mut sprite_handle) in &mut query {
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            animation.timer.reset();

            if animation.is_last_frame() {
                animation.played_once = true;
                if animation.is_repeating() {
                    animation.current_frame = 0;
                }
            } else if animation.current_animation != None {
                animation.current_frame += 1;
            }
        }

        if let Some(curr) = &animation.current_animation {
            *sprite_handle = animation.animations.get(curr).unwrap().atlas_handle.clone();
            sprite.index = animation.current_frame;
            sprite.flip_x = animation.flip_x;
        }
    }
}
