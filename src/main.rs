#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod physics;

use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use physics::{GravityDirection, Ground, JumpState, PhysicsPlugin, PhysicsSet, Velocity};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin)
        .insert_resource(RapierConfiguration {
            timestep_mode: TimestepMode::Fixed {
                dt: 1. / 60.,
                substeps: 1,
            },
            ..Default::default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InputManagerPlugin::<JumpAction>::default())
        .add_plugin(InputManagerPlugin::<MovementAction>::default())
        .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
        .add_startup_system(setup)
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_ground)
        .add_system(control_jump)
        .add_system(control_movement)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum JumpAction {
    Jump,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum MovementAction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        InputManagerBundle::<JumpAction> {
            action_state: ActionState::default(),
            input_map: InputMap::new([(KeyCode::Space, JumpAction::Jump)]),
        },
        InputManagerBundle::<MovementAction> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (KeyCode::A, MovementAction::Left),
                (KeyCode::D, MovementAction::Right),
                (KeyCode::Q, MovementAction::Up),
                (KeyCode::S, MovementAction::Down),
            ]),
        },
        Player,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20., 30.)),
                color: Color::rgb(0.0, 0.7, 0.7),
                ..default()
            },
            ..default()
        },
        Velocity::default(),
        GravityDirection::Down,
        JumpState { on_ground: true },
        Collider::cuboid(10., 15.),
        Sensor,
    ));
}

fn control_jump(
    mut q: Query<(
        &mut Velocity,
        &mut JumpState,
        &GravityDirection,
        &ActionState<JumpAction>,
    )>,
) {
    const INITIAL_JUMP_SPEED: f32 = 300.0;
    for (mut v, mut jump_state, g_dir, action_state) in q.iter_mut() {
        if action_state.just_pressed(JumpAction::Jump) {
            if !jump_state.on_ground {
                return;
            }
            v.0 -= INITIAL_JUMP_SPEED * g_dir.as_vec2();
            jump_state.on_ground = false;
        }
    }
}

fn control_movement(mut q: Query<(&mut Velocity, &ActionState<MovementAction>, &GravityDirection)>) {
    const HORIZONTAL_SPEED: f32 = 200.0;
    for (mut v, action, dir) in &mut q {
        let mut temp_v = Vec2::ZERO;
        if action.pressed(MovementAction::Down) {
            temp_v.y -= 1.0;
        }
        if action.pressed(MovementAction::Up) {
            temp_v.y += 1.0;
        }
        if action.pressed(MovementAction::Left) {
            temp_v.x -= 1.0;
        }
        if action.pressed(MovementAction::Right) {
            temp_v.x += 1.0;
        }

        let val = dir.forward().dot(temp_v);
        if val != 0.0 {
            v.0 = v.0 * dir.as_vec2().abs() + (dir.forward() * val).normalize() * HORIZONTAL_SPEED;
        } else {
            v.0 *= dir.as_vec2().abs();
        }
    }
}

fn spawn_ground(mut commands: Commands) {
    // bottom
    commands.spawn((
        Ground,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(600., 20.)),
                anchor: Anchor::Center,
                ..default()
            },
            transform: Transform::from_xyz(0.0, -300.0, 0.0),
            ..default()
        },
        Collider::cuboid(300., 10.),
        Sensor,
    ));

    // top
    commands.spawn((
        Ground,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(600., 20.)),
                anchor: Anchor::Center,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 300.0, 0.0),
            ..default()
        },
        Collider::cuboid(300., 10.),
        Sensor,
    ));

    // left
    commands.spawn((
        Ground,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20., 600.)),
                anchor: Anchor::Center,
                ..default()
            },
            transform: Transform::from_xyz(300.0, 0.0, 0.0),
            ..default()
        },
        Collider::cuboid(10., 300.),
        Sensor,
    ));

    // right
    commands.spawn((
        Ground,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20., 600.)),
                anchor: Anchor::Center,
                ..default()
            },
            transform: Transform::from_xyz(-300.0, 0.0, 0.0),
            ..default()
        },
        Collider::cuboid(10., 300.),
        Sensor,
    ));
}
