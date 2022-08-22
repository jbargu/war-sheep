use bevy::prelude::*;

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct AttackValue(pub f32);

#[derive(Component)]
pub struct AttackRange(pub f32);

#[derive(Component)]
pub struct SpottingRange(pub f32);

#[derive(Component)]
pub enum PursuitType {
    ChasingClosest, // the entity will chase the closest enemy entity
}

#[derive(Component)]
pub struct Bounds {
    pub x: (f32, f32),
    pub y: (f32, f32),
}

pub fn bounds_check(mut transforms: Query<(&mut Transform, &Bounds), Changed<Transform>>) {
    for (mut transform, bounds) in transforms.iter_mut() {
        if transform.translation.y > bounds.y.1 {
            transform.translation.y = bounds.y.1;
        } else if transform.translation.y < bounds.y.0 {
            transform.translation.y = bounds.y.0;
        }

        if transform.translation.x > bounds.x.1 {
            transform.translation.x = bounds.x.1;
        } else if transform.translation.x < bounds.x.0 {
            transform.translation.x = bounds.x.0
        }
    }
}
