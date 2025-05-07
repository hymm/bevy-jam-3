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
            .set(AssetPlugin { ..default() })
            .set(ImagePlugin::default_nearest()),
    )
    .insert_resource(Time::<Fixed>::from_seconds(1.0 / 50.0))
    .add_plugins((
        RonAssetPlugin::<PhysicsSettings>::new(&["physics.ron"]),
        RngPlugin::default(),
        LdtkPlugin,
    ));

    app.add_plugins((
        GameStatePlugin,
        GroundPlugin,
        StartMenuPlugin,
        PhysicsPlugin,
        GoalPlugin,
        LevelPlugin,
        PlayerPlugin,
        WinScreenPlugin,
        SfxPlugin,
        CollisionPlugin::<CollisionTypes>::new(),
        CollisionDebugPlugin,
    ))
    .insert_resource(PhysicsSettings {
        // these are overridden by the setting.ron
        initial_jump_speed: 400.0,
        gravity_pressed: 40.0,
        gravity_unpressed: 200.0,
        horizontal_speed: 200.0,
        max_speed: 700.0,
    })
    .add_systems(Startup, setup);

    #[cfg(debug_assertions)]
    bevy_mod_debugdump::print_schedule_graph(&mut app, PostUpdate);
    let dot = bevy_mod_debugdump::schedule_graph_dot(
        &mut app,
        FixedUpdate,
        &bevy_mod_debugdump::schedule_graph::Settings::default(),
    );
    print!("{dot}");
    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, Transform::from_xyz(360.0, 360.0, 1000.0)));

    // background
    commands.spawn((
        Sprite::from_image(asset_server.load("bg.png")),
        Transform::from_xyz(360., 360., 0.),
    ));
}
