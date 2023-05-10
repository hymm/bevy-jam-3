use std::f32::consts::PI;

use crate::{
    collisions::{CollisionData, CollisionEvents, PositionDelta},
    constants::CollisionTypes,
};
use bevy::{prelude::*, reflect::TypeUuid};

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                // rotate_gravity,
                falling_detection,
                ground_detection,
                apply_gravity,
                apply_acceleration,
                apply_velocity,
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

#[derive(Component, Default)]
pub struct Gravity(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    Up,
    Down,
    #[default]
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

impl Default for GravityDirection {
    fn default() -> Self {
        Self(Direction::Down)
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

impl Default for JumpState {
    fn default() -> JumpState {
        JumpState {
            on_ground: false,
            turned_this_jump: false,
            last_horizontal_movement_dir: Direction::Left,
            last_vertical_movement_dir: Direction::Down,
        }
    }
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

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity, Option<&mut PositionDelta>)>,
    time_step: Res<FixedTime>,
) {
    for (mut transform, velocity, delta) in &mut query {
        let last_translation = transform.translation.truncate();
        transform.translation += velocity.0.extend(0.) * time_step.period.as_secs_f32();
        if let Some(mut delta) = delta {
            delta.origin = last_translation;
            delta.ray = transform.translation.truncate() - last_translation;
        }
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

/// marker component for a ray that controls ground collisions
#[derive(Component)]
pub struct GroundRay;

// if all ground rays are not on the ground then the entity should be falling
fn falling_detection(
    mut jumpers: Query<(
        &mut JumpState,
        &CollisionEvents<CollisionTypes>,
        &GravityDirection,
    )>,
) {
    for (mut j, ev, g) in &mut jumpers {
        if !j.on_ground {
            continue;
        }

        let mut touching_ground = false;

        for event in &ev.buffer {
            let CollisionData::Ray(ref ray_data) = event.data else { continue; };
            // check if ray points "down" and intersects a ground collision
            if event.user_type == CollisionTypes::Ground
                && ray_data.ray_direction.angle_between(g.as_vec2()) == 0.0
            {
                touching_ground = true;
                break;
            }
        }

        if !touching_ground {
            j.on_ground = false;
        }
    }
}

// if all ground rays are not on the ground then the entity should be falling
fn ground_detection(
    mut jumpers: Query<(
        &mut JumpState,
        &mut Transform,
        &mut Velocity,
        &mut Acceleration,
        &CollisionEvents<CollisionTypes>,
        &GravityDirection,
    )>,
) {
    for (mut j, mut t, mut v, mut a, ev, g) in &mut jumpers {
        let mut touching_ground = false;
        let mut collision = None;
        for event in &ev.buffer {
            // ignore other types of collision other than Aabb collisions
            let CollisionData::Aabb(ref sweep) = event.data else { continue; };
            match event.user_type {
                CollisionTypes::Ground => {
                    if collision.is_none() {
                        collision = Some(sweep);
                    } else {
                        if sweep.time < collision.unwrap().time {
                            collision = Some(sweep);
                        }
                    }

                    // check if ground collision is a "floor"
                    if sweep.normal.angle_between(g.reverse().as_vec2()) == 0.0 {
                        touching_ground = true;
                    }
                }
                _ => {}
            }
        }

        if let Some(collision) = collision {
            // set position outside of ground
            // note: this would be incorrect if jumper is a child of another transform
            t.translation = collision.position.extend(t.translation.z);

            // set velocity and acceleration in direction ground to zero
            if v.0.dot(Vec2::from_angle(PI).rotate(collision.normal)) > 0. {
                v.0 *= Vec2::from_angle(PI / 2.).rotate(collision.normal).abs();
            }
            if a.0.dot(Vec2::from_angle(PI).rotate(collision.normal)) > 0. {
                a.0 *= Vec2::from_angle(PI / 2.).rotate(collision.normal).abs();
            }
        }

        if touching_ground {
            j.on_ground = true;
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
