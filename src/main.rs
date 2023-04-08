#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod constants;
mod goals;
mod ground;
mod physics;

use crate::goals::GoalBundle;
use crate::goals::GoalPlugin;
use crate::ground::GroundBundle;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use constants::PLAYER_DIM;
use leafwing_input_manager::prelude::*;
use physics::{
    Acceleration, Direction, Gravity, GravityDirection, JumpState, PhysicsPlugin, Velocity,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin)
        .add_plugin(GoalPlugin)
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
        .add_startup_system(spawn_level)
        .add_system(control_jump)
        .add_system(control_movement)
        .add_system(player_dies)
        .add_system(sprite_orientation)
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

fn spawn_player(mut commands: Commands, asset_server: ResMut<AssetServer>) {
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
                custom_size: Some(PLAYER_DIM),
                ..default()
            },
            texture: asset_server.load("pixel-cat.png"),
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
        Collider::cuboid(PLAYER_DIM.x / 2., PLAYER_DIM.y / 2.),
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
    const GRAVITY_UNPRESSED: f32 = 200.0;
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

fn sprite_orientation(
    mut player: Query<(&mut Sprite, &Velocity, &GravityDirection), With<Player>>,
) {
    for (mut s, v, g) in &mut player {
        let forward_speed = g.forward().as_vec2().dot(v.0);
        if forward_speed > 0. {
            s.flip_x = false;
        } else if forward_speed < 0. {
            s.flip_x = true;
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

fn spawn_level(mut commands: Commands) {
    // bottom
    commands.spawn(GroundBundle::new(
        Vec2::new(600., 20.),
        Vec2::new(0.0, -300.0),
    ));

    // top
    commands.spawn(GroundBundle::new(Vec2::new(600., 20.), Vec2::new(0., 300.)));

    // left
    commands.spawn(GroundBundle::new(Vec2::new(20., 600.), Vec2::new(300., 0.)));

    // right
    commands.spawn(GroundBundle::new(
        Vec2::new(20., 600.),
        Vec2::new(-300., 0.),
    ));

    // a platform
    commands.spawn(GroundBundle::new(
        Vec2::new(60., 60.),
        Vec2::new(160., -160.),
    ));

    // a platform
    commands.spawn(GroundBundle::new(Vec2::new(60., 60.), Vec2::new(0., -0.)));

    // a platform
    commands.spawn(GroundBundle::new(
        Vec2::new(60., 60.),
        Vec2::new(-160., 160.),
    ));

    // a goal
    commands.spawn(GoalBundle::new(Vec2::new(-160., 115.)));
}
