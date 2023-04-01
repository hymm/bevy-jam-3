#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod physics;

use bevy::{prelude::*, sprite::Anchor};
use leafwing_input_manager::prelude::*;
use physics::{GravityDirection, PhysicsPlugin, Velocity};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin)
        .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
        .add_startup_system(setup)
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_ground)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Jump,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (KeyCode::Space, Action::Jump),
                (KeyCode::Q, Action::Up),
                (KeyCode::A, Action::Left),
                (KeyCode::S, Action::Down),
                (KeyCode::F, Action::Right),
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
    ));
}

#[derive(Component)]
struct Ground;

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
    ));
}
