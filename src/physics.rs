use bevy::prelude::*;

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (apply_gravity, apply_velocity)
                .chain()
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

/// Direction gravity applies to for a specific object,
/// Note: might be better for this to be a vector instead?
#[derive(Component)]
pub enum GravityDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

fn apply_gravity(mut q: Query<(&mut Velocity, &GravityDirection)>, time: Res<FixedTime>) {
    const GRAVITY: f32 = 500.0;
    for (mut v, dir) in q.iter_mut() {
        match dir {
            GravityDirection::Up => v.y += GRAVITY * time.period.as_secs_f32(),
            GravityDirection::Down => v.y -= GRAVITY * time.period.as_secs_f32(),
            GravityDirection::Left => v.x += GRAVITY * time.period.as_secs_f32(),
            GravityDirection::Right => v.x -= GRAVITY * time.period.as_secs_f32(),
        }
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time_step: Res<FixedTime>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time_step.period.as_secs_f32();
        transform.translation.y += velocity.y * time_step.period.as_secs_f32();
    }
}
