use bevy::prelude::*;
use bevy_rapier2d::prelude::{QueryFilter, RapierContext};

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                ground_detection,
                apply_gravity,
                apply_acceleration,
                apply_velocity,
            )
                .chain()
                .in_set(PhysicsSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PhysicsSet;

/// Direction gravity applies to for a specific object,
/// Note: might be better for this to be a vector instead?
#[derive(Component)]
pub enum GravityDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
pub struct Gravity(pub f32);

// a ground entity
#[derive(Component)]
pub struct Ground;

impl GravityDirection {
    pub fn as_vec2(&self) -> Vec2 {
        match self {
            GravityDirection::Down => Vec2::NEG_Y,
            GravityDirection::Up => Vec2::Y,
            GravityDirection::Left => Vec2::NEG_X,
            GravityDirection::Right => Vec2::X,
        }
    }

    pub fn forward(&self) -> Vec2 {
        match self {
            GravityDirection::Down => Vec2::X,
            GravityDirection::Up => Vec2::NEG_X,
            GravityDirection::Left => Vec2::Y,
            GravityDirection::Right => Vec2::NEG_Y,
        }
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct Acceleration(pub Vec2);

#[derive(Component)]
pub struct JumpState {
    pub on_ground: bool,
}

fn apply_gravity(
    mut q: Query<(
        &mut Acceleration,
        &mut Velocity,
        &GravityDirection,
        &Gravity,
        &JumpState,
    )>,
) {
    for (mut a, mut v, dir, gravity, jump_state) in q.iter_mut() {
        if jump_state.on_ground {
            // zero velocity in gravity direction. probably need to add some thresholds here too.
            v.0 = v.0 + v.0 * dir.as_vec2();
            a.0 *= dir.forward().abs();
            continue;
        }

        a.0 += gravity.0 * dir.as_vec2();
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time_step: Res<FixedTime>) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0.extend(0.) * time_step.period.as_secs_f32();
    }
}

fn apply_acceleration(mut q: Query<(&mut Velocity, &Acceleration)>, time_step: Res<FixedTime>) {
    const MAX_VELOCITY: Vec2 = Vec2::new(700.0, 700.0);
    for (mut v, a) in &mut q {
        v.0 += a.0 * time_step.period.as_secs_f32();
        v.0 = v.0.clamp(-MAX_VELOCITY, MAX_VELOCITY);
    }
}

fn ground_detection(
    mut jumpers: Query<
        (
            &mut JumpState,
            &Velocity,
            &Transform,
            &GravityDirection,
            &Sprite,
        ),
        Without<Ground>,
    >,
    grounds: Query<Entity, With<Ground>>,
    rapier: Res<RapierContext>,
    time: Res<FixedTime>,
) {
    for (mut j, v, t, g, s) in &mut jumpers {
        // only check ground detection when moving in the same direction as gravity
        if g.as_vec2().dot(v.0) < 0.0 {
            continue;
        }
        let Some(sprite_size) = s.custom_size else { panic!("cannot get size of sprite") };
        // left ray when gravity is down
        let ray_origin_1 = t.translation.truncate() - sprite_size / 2.;
        let result = rapier.cast_ray_and_get_normal(
            ray_origin_1,
            g.as_vec2(),
            15.0,
            false,
            QueryFilter::default(),
        );

        if let Some((entity, intersect)) = result {
            let speed = g.as_vec2().dot(v.0);

            if speed == 0. {
                return;
            }
            let something = intersect.toi / speed;

            if grounds.contains(entity) && something < time.period.as_secs_f32() {
                j.on_ground = true;
            }
        } else {
            j.on_ground = false;
        }
    }
}
