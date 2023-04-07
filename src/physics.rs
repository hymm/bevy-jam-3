use bevy::prelude::*;
use bevy_rapier2d::prelude::{QueryFilter, RapierContext};

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                // rotate_gravity,
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

#[derive(Component)]
pub struct Gravity(pub f32);

// a ground entity
#[derive(Component)]
pub struct Ground;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn reverse(&self) -> Direction {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn as_vec2(&self) -> Vec2 {
        match self {
            Direction::Down => Vec2::NEG_Y,
            Direction::Up => Vec2::Y,
            Direction::Left => Vec2::NEG_X,
            Direction::Right => Vec2::X,
        }
    }

    // rotate 90deg counter clockwise
    pub fn ccw(&self) -> Direction {
        match self {
            Direction::Down => Direction::Right,
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
        }
    }

    // rotate 90deg clockwise
    pub fn cw(&self) -> Direction {
        match self {
            Direction::Down => Direction::Left,
            Direction::Up => Direction::Right,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }
}

/// Direction gravity applies to for a specific object,
/// Note: might be better for this to be a vector instead?
#[derive(Component, Deref, DerefMut)]
pub struct GravityDirection(pub Direction);
impl GravityDirection {
    pub fn forward(&self) -> Direction {
        match self.0 {
            Direction::Down => Direction::Left,
            Direction::Up => Direction::Right,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
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
    pub last_horizontal_movement_dir: Direction,
    pub last_vertical_movement_dir: Direction,
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
            a.0 *= dir.forward().as_vec2().abs();
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
            Entity,
            &mut JumpState,
            &Velocity,
            &mut Transform,
            &GravityDirection,
        ),
        Without<Ground>,
    >,
    grounds: Query<Entity, With<Ground>>,
    rapier: Res<RapierContext>,
    time: Res<FixedTime>,
) {
    let character_height = 30.0;
    for (e, mut j, v, mut t, g) in &mut jumpers {
        // only check ground detection when moving in the same direction as gravity
        if g.as_vec2().dot(v.0) < 0.0 {
            continue;
        }
        // left ray when gravity is down
        let ray_origin_1 = t.translation.truncate();
        let result = rapier.cast_ray_and_get_normal(
            ray_origin_1,
            g.as_vec2(),
            character_height / 2. + 14.,
            false,
            QueryFilter::default().exclude_collider(e),
        );

        if let Some((entity, intersect)) = result {
            let speed = g.as_vec2().dot(v.0);

            // if speed == 0. {
            //     return;
            // }
            // toi from rapier seems to be in pixels, so we convert
            let toi = (intersect.toi - character_height / 2.) / speed;

            if grounds.contains(entity) && toi < time.period.as_secs_f32() {
                t.translation += g.as_vec2().extend(0.0) * (intersect.toi - character_height / 2.);
                j.on_ground = true;
            }
        } else {
            j.on_ground = false;
        }
    }
}

fn rotate_gravity(
    mut q: Query<(
        &mut GravityDirection,
        &mut JumpState,
        &mut Acceleration,
        &mut Transform,
        &Velocity,
    )>,
) {
    for (mut g_dir, mut jump_state, mut a, mut t, v) in &mut q {
        let v_speed = g_dir.as_vec2().dot(v.0);
        let current_v_direction = if v_speed > 0.0 {
            g_dir.0
        } else if v_speed < 0.0 {
            g_dir.0.reverse()
        } else {
            jump_state.last_vertical_movement_dir
        };

        let h_speed = g_dir.forward().as_vec2().dot(v.0);
        let current_h_direction = if h_speed > 0.0 {
            g_dir.forward()
        } else if h_speed < 0.0 {
            g_dir.forward().reverse()
        } else {
            jump_state.last_horizontal_movement_dir
        };

        if current_v_direction != jump_state.last_vertical_movement_dir
            && current_v_direction == g_dir.0
        {
            a.0 = Vec2::ZERO;
            g_dir.0 = if current_h_direction == g_dir.forward() {
                t.rotate_z(90.);
                g_dir.cw()
            } else {
                t.rotate_z(-90.);
                g_dir.ccw()
            };
        }

        jump_state.last_horizontal_movement_dir = current_h_direction;
        jump_state.last_vertical_movement_dir = current_v_direction;
    }
}
