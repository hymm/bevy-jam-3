#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod constants;
mod game_state;
mod goals;
mod ground;
mod level;
mod physics;
mod player;
mod start_menu;

use crate::goals::GoalPlugin;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier2d::prelude::*;
use game_state::GameStatePlugin;
use level::{CurrentLevel, Level, LevelPlugin, GroundConfig};
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use start_menu::StartMenuPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cat Spin".to_string(),
                resolution: WindowResolution::new(720., 720.),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(GameStatePlugin)
        .add_plugin(StartMenuPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(GoalPlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(PlayerPlugin)
        .insert_resource(RapierConfiguration {
            timestep_mode: TimestepMode::Fixed {
                dt: 1. / 60.,
                substeps: 1,
            },
            ..Default::default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        // .add_plugin(RapierDebugRenderPlugin::default())
        .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(CurrentLevel(Level {
        spawn: Vec2::new(0., -250.),
        grounds: vec![
            // bottom
            GroundConfig {
                dim: Vec2::new(600., 20.),
                pos: Vec2::new(0.0, -300.0),
            },
            // top
            // GroundConfig {
            //     dim: Vec2::new(600., 20.),
            //     pos: Vec2::new(0.0, 300.0),
            // },
            // left
            GroundConfig {
                dim: Vec2::new(20., 600.),
                pos: Vec2::new(300., 0.),
            },
            // right
            GroundConfig {
                dim: Vec2::new(20., 600.),
                pos: Vec2::new(-300., 0.),
            },
            // platform
            GroundConfig {
                dim: Vec2::new(60., 60.),
                pos: Vec2::new(160., -160.),
            }, // platform
            GroundConfig {
                dim: Vec2::new(60., 60.),
                pos: Vec2::new(0., 0.),
            }, // platform
            GroundConfig {
                dim: Vec2::new(60., 60.),
                pos: Vec2::new(-160., 160.),
            },
        ],
        goals: vec![
            Vec2::new(-160., 115.),
            Vec2::new(0., -45.),
            Vec2::new(115., -160.),
        ],
    }));
}
