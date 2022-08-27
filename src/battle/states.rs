use bevy::prelude::*;

#[derive(Component)]
pub struct Idling;

impl Idling {
    pub const ANIMATION: &'static str = "idling";
}

#[derive(Component)]
pub struct Walking;

impl Walking {
    pub const ANIMATION: &'static str = "walking";
}

#[derive(Component)]
pub struct Attacking;

impl Attacking {
    pub const ANIMATION: &'static str = "attacking";
}

#[derive(Component)]
pub struct Dying;

impl Dying {
    pub const ANIMATION: &'static str = "dying";
}
