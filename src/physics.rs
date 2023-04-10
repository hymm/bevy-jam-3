use std::f32::consts::PI;

use crate::{constants::PLAYER_DIM, ground::Ground};
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_rapier2d::prelude::{QueryFilter, RapierContext};

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                rotate_gravity,
                ground_detection,
                apply_gravity,
                apply_acceleration,
                apply_velocity,
                top_collision_detection,
                side_collision_detection,
            )
                .chain()
                .in_set(PhysicsSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
        app.add_startup_system(load_physics);
        app.add_system(monitor_physics_changes);
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PhysicsSet;

#[derive(Component)]
pub struct Gravity(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub turned_this_jump: bool,
    pub last_horizontal_movement_dir: Direction,
    pub last_vertical_movement_dir: Direction,
}

#[derive(Resource, serde::Deserialize, TypeUuid, Debug, Clone)]
#[uuid = "4393bc64-8efd-422e-b0b3-873d40261987"]
pub struct PhysicsSettings {
    pub initial_jump_speed: f32,
    pub gravity_pressed: f32,
    pub gravity_unpressed: f32,
    pub horizontal_speed: f32,
    pub max_speed: f32,
}

#[derive(Resource)]
struct PhysicsSettingsHandle(pub Handle<PhysicsSettings>);

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
            // zero velocity in gravity direction.
            v.0 *= dir.forward().as_vec2().abs();
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

fn apply_acceleration(
    mut q: Query<(&mut Velocity, &Acceleration)>,
    time_step: Res<FixedTime>,
    settings: Res<PhysicsSettings>,
) {
    let max_velocity = Vec2::new(settings.max_speed, settings.max_speed);
    for (mut v, a) in &mut q {
        v.0 += a.0 * time_step.period.as_secs_f32();
        v.0 = v.0.clamp(-max_velocity, max_velocity);
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
    let character_height = PLAYER_DIM.y;
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
            QueryFilter::default().exclude_collider(e).exclude_sensors(),
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

fn side_collision_detection(
    mut movers: Query<
        (Entity, &Velocity, &mut Transform, &GravityDirection),
        (With<JumpState>, Without<Ground>),
    >,
    grounds: Query<Entity, With<Ground>>,
    rapier: Res<RapierContext>,
    time: Res<FixedTime>,
) {
    let character_width: f32 = PLAYER_DIM.x;
    for (e, v, mut t, g) in &mut movers {
        let h_move_vec =
            (g.forward().as_vec2().dot(v.0) * g.forward().as_vec2()).normalize_or_zero();

        // left ray when gravity is down
        let ray_origin_1 = t.translation.truncate();
        let result = rapier.cast_ray_and_get_normal(
            ray_origin_1,
            h_move_vec,
            character_width / 2. + 14.,
            false,
            QueryFilter::default().exclude_collider(e).exclude_sensors(),
        );

        if let Some((entity, intersect)) = result {
            let speed = h_move_vec.dot(v.0);

            // if speed == 0. {
            //     return;
            // }
            // toi from rapier seems to be in pixels, so we convert
            let toi = (intersect.toi - character_width / 2.) / speed;

            if grounds.contains(entity) && toi < time.period.as_secs_f32() {
                t.translation += h_move_vec.extend(0.0) * (intersect.toi - character_width / 2.);
            }
        }
    }
}

fn top_collision_detection(
    mut movers: Query<
        (Entity, &Velocity, &mut Transform, &GravityDirection),
        (With<JumpState>, Without<Ground>),
    >,
    grounds: Query<Entity, With<Ground>>,
    rapier: Res<RapierContext>,
    time: Res<FixedTime>,
) {
    let character_height: f32 = PLAYER_DIM.y;
    for (e, v, mut t, g) in &mut movers {
        let v_move_vec = g.reverse().as_vec2();

        // left ray when gravity is down
        let ray_origin_1 = t.translation.truncate();
        let result = rapier.cast_ray_and_get_normal(
            ray_origin_1,
            v_move_vec,
            character_height / 2. + 14.,
            false,
            QueryFilter::default().exclude_collider(e).exclude_sensors(),
        );

        if let Some((entity, intersect)) = result {
            let speed = v_move_vec.dot(v.0);

            // toi from rapier seems to be in pixels, so we convert
            let toi = (intersect.toi - character_height / 2.) / speed;

            if grounds.contains(entity) && toi < time.period.as_secs_f32() {
                t.translation += v_move_vec.extend(0.0) * (intersect.toi - character_height / 2.);
            }
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
            && !jump_state.turned_this_jump
        {
            a.0 = Vec2::ZERO;
            jump_state.turned_this_jump = true;
            g_dir.0 = if current_h_direction == g_dir.forward() {
                t.rotate_z(-PI / 2.);
                g_dir.cw()
            } else {
                t.rotate_z(PI / 2.);
                g_dir.ccw()
            };
        }

        jump_state.last_horizontal_movement_dir = current_h_direction;
        jump_state.last_vertical_movement_dir = current_v_direction;
    }
}

fn load_physics(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load("settings.physics.ron");
    commands.insert_resource(PhysicsSettingsHandle(handle));
}

fn monitor_physics_changes(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<PhysicsSettings>>,
    settings: Res<Assets<PhysicsSettings>>,
) {
    for e in &mut events {
        match e {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let setting = settings.get(handle).unwrap();
                commands.insert_resource(setting.clone())
            }
            _ => {}
        }
    }
}
