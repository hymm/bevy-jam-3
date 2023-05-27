#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod collisions;
mod constants;
mod game_state;
mod goals;
mod ground;
mod level;
mod physics;
mod player;
mod sfx;
mod start_menu;
mod win_screen;

use crate::goals::GoalPlugin;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ecs_ldtk::LdtkPlugin;
use bevy_turborand::prelude::*;
use collisions::{CollisionDebugPlugin, CollisionPlugin};
use constants::CollisionTypes;
use game_state::GameStatePlugin;
use ground::GroundPlugin;
use level::LevelPlugin;
use physics::{PhysicsPlugin, PhysicsSettings};
use player::PlayerPlugin;
use sfx::SfxPlugin;
use start_menu::StartMenuPlugin;
use win_screen::WinScreenPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Cats Always Land on their Feet".to_string(),
                    resolution: WindowResolution::new(720., 720.),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes: true,
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugin(RonAssetPlugin::<PhysicsSettings>::new(&["physics.ron"]))
    .add_plugin(RngPlugin::default())
    .insert_resource(FixedTime::new_from_secs(1.0 / 60.0));

    app.add_plugin(LdtkPlugin);

    app.add_plugin(GameStatePlugin)
        .add_plugin(GroundPlugin)
        .add_plugin(StartMenuPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(GoalPlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(WinScreenPlugin)
        .add_plugin(SfxPlugin)
        .add_plugin(CollisionPlugin::<CollisionTypes>::new())
        // .add_plugin(CollisionDebugPlugin)
        .insert_resource(PhysicsSettings {
            // these are overridden by the setting.ron
            initial_jump_speed: 400.0,
            gravity_pressed: 40.0,
            gravity_unpressed: 200.0,
            horizontal_speed: 200.0,
            max_speed: 700.0,
        })
        .add_startup_system(setup);

    // #[cfg(debug_assertions)]
    // bevy_mod_debugdump::print_main_schedule(&mut app);
    // let dot = bevy_mod_debugdump::schedule_graph_dot(
    //     &mut app,
    //     CoreSchedule::FixedUpdate,
    //     &bevy_mod_debugdump::schedule_graph::Settings::default(),
    // );
    // print!("{dot}");
    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(360.0, 360.0, 1000.0),
        ..default()
    });

    // background
    commands.spawn(SpriteBundle {
        texture: asset_server.load("bg.png"),
        transform: Transform::from_xyz(360., 360., 0.),
        ..default()
    });
}
