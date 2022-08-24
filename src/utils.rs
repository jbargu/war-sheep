use bevy::prelude::*;

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Attack {
    pub attack_damage: f32,
    pub attack_range: f32,
    pub spotting_range: f32,
}

#[derive(Component)]
pub enum BehaviourType {
    ChasingClosest, // the entity will chase the closest enemy entity
}

#[derive(Component)]
pub struct Bounds {
    pub x: (f32, f32),
    pub y: (f32, f32),
}

/// Components tagged with this will be despawn on the stage exit
#[derive(Component)]
pub struct UnloadOnExit;

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

/// Despawns the entities with given component
pub fn despawn_entities_with_component<T: Component>(
    mut commands: Commands,
    q: Query<Entity, With<T>>,
) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}
