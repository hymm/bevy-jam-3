#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod physics;

use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use physics::{
    Acceleration, Direction, Gravity, GravityDirection, Ground, JumpState, PhysicsPlugin, Velocity,
};

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
        .add_system(player_dies)
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
        Player,
        InputManagerBundle::<JumpAction> {
            action_state: ActionState::default(),
            input_map: InputMap::new([(KeyCode::Space, JumpAction::Jump)]),
        },
        InputManagerBundle::<MovementAction> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (KeyCode::A, MovementAction::Left),
                (KeyCode::D, MovementAction::Right),
                (KeyCode::W, MovementAction::Up),
                (KeyCode::S, MovementAction::Down),
            ]),
        },
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20., 30.)),
                color: Color::rgb(0.0, 0.7, 0.7),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -250., 0.),
            ..default()
        },
        Velocity::default(),
        Acceleration::default(),
        GravityDirection(Direction::Down),
        Gravity(50.0),
        JumpState {
            on_ground: true,
            turned_this_jump: false,
            last_horizontal_movement_dir: Direction::Left,
            last_vertical_movement_dir: Direction::Down,
        },
        Collider::cuboid(10., 15.),
        Sensor,
    ));
}

fn control_jump(
    mut q: Query<(
        &mut Velocity,
        &mut JumpState,
        &mut Gravity,
        &GravityDirection,
        &ActionState<JumpAction>,
    )>,
) {
    const INITIAL_JUMP_SPEED: f32 = 400.0;
    const GRAVITY_PRESSED: f32 = 40.0;
    const GRAVITY_UNPRESSED: f32 = 80.0;
    for (mut v, mut jump_state, mut g, g_dir, action_state) in q.iter_mut() {
        if action_state.just_pressed(JumpAction::Jump) {
            if !jump_state.on_ground {
                return;
            }
            v.0 -= INITIAL_JUMP_SPEED * g_dir.as_vec2();
            jump_state.on_ground = false;
            jump_state.turned_this_jump = false;
        }

        g.0 = if action_state.pressed(JumpAction::Jump) {
            GRAVITY_PRESSED
        } else {
            GRAVITY_UNPRESSED
        }
    }
}

fn control_movement(
    mut q: Query<(
        &mut Velocity,
        &ActionState<MovementAction>,
        &GravityDirection,
    )>,
) {
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

        let val = dir.forward().as_vec2().dot(temp_v);
        if val != 0.0 {
            v.0 = v.0 * dir.as_vec2().abs()
                + (dir.forward().as_vec2() * val).normalize() * HORIZONTAL_SPEED;
        } else {
            v.0 *= dir.as_vec2().abs();
        }
    }
}

fn player_dies(q: Query<(Entity, &Transform), With<Player>>, mut commands: Commands) {
    for (e, t) in &q {
        if t.translation.y < -500. {
            commands.entity(e).despawn();
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

    // a platform
    commands.spawn((
        Ground,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(50., 20.)),
                anchor: Anchor::Center,
                ..default()
            },
            transform: Transform::from_xyz(100.0, -220.0, 0.0),
            ..default()
        },
        Collider::cuboid(25., 10.),
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
