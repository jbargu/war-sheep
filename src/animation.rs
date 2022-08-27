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
        }
    }

    pub fn play(&mut self, name: &str, repeating: bool) {
        self.current_animation = Some(name.to_owned());
        self.current_frame = 0;
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
}

/// Cycles all animations
pub fn animate(time: Res<Time>, mut query: Query<(&mut Animation, &mut TextureAtlasSprite)>) {
    for (mut animation, mut sprite) in &mut query {
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            animation.timer.reset();

            if animation.is_last_frame() {
                if animation.is_repeating() {
                    animation.current_frame = 0;
                }
            } else if animation.current_animation != None {
                animation.current_frame += 1;
            }
        }

        if let Some(_) = animation.current_animation {
            sprite.index = animation.current_frame;
        }
    }
}
