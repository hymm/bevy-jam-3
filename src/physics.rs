use bevy::prelude::*;
use bevy_rapier2d::prelude::{QueryFilter, RapierContext};

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (ground_detection, apply_gravity, apply_velocity)
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

// a ground entity
#[derive(Component)]
pub struct Ground;

impl GravityDirection {
    fn as_vec2(&self) -> Vec2 {
        match self {
            GravityDirection::Down => Vec2::NEG_Y,
            GravityDirection::Up => Vec2::Y,
            GravityDirection::Left => Vec2::NEG_X,
            GravityDirection::Right => Vec2::X,
        }
    }
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct JumpState {
    pub on_ground: bool,
}

fn apply_gravity(
    mut q: Query<(&mut Velocity, &GravityDirection, &JumpState)>,
    time: Res<FixedTime>,
) {
    const GRAVITY: f32 = 500.0;
    for (mut v, dir, jump_state) in q.iter_mut() {
        if jump_state.on_ground {
            // zero velocity in gravity direction. probably need to add some thresholds here too.
            v.0 = v.0 + v.0 * dir.as_vec2();
            continue;
        }

        v.0 += GRAVITY * time.period.as_secs_f32() * dir.as_vec2();
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time_step: Res<FixedTime>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time_step.period.as_secs_f32();
        transform.translation.y += velocity.y * time_step.period.as_secs_f32();
    }
}

fn ground_detection(
    mut jumpers: Query<(&mut JumpState, &Transform, &GravityDirection, &Sprite), Without<Ground>>,
    grounds: Query<Entity, With<Ground>>,
    rapier: Res<RapierContext>,
) {
    for (mut j, t, g, s) in &mut jumpers {
        let Some(sprite_size) = s.custom_size else { panic!("cannot get size of sprite") };
        // left ray when gravity is down
        let ray_origin_1 = t.translation.truncate() - sprite_size / 2.;
        let result = rapier.cast_ray_and_get_normal(
            ray_origin_1,
            g.as_vec2(),
            10.0,
            false,
            QueryFilter::default(),
        );

        if let Some((entity, intersect)) = result {
            if grounds.contains(entity) {
                j.on_ground = true;
            }
        } else {
            j.on_ground = false;
        }
    }
}
