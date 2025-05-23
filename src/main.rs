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
use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
use bevy::window::WindowResolution;
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_aseprite_ultra::AsepriteUltraPlugin;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ecs_ldtk::{LdtkPlugin, LdtkSettings, LevelBackground};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_turborand::prelude::*;

use collisions::{CollisionDebugPlugin, CollisionPlugin, DebugCollisions};
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
    .insert_resource(LdtkSettings {
        level_background: LevelBackground::Nonexistent,
        ..default()
    })
    .add_plugins((
        RonAssetPlugin::<PhysicsSettings>::new(&["physics.ron"]),
        RngPlugin::default(),
        LdtkPlugin,
        AsepriteUltraPlugin,
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
        WorldInspectorPlugin::new().run_if(|condition: Res<DebugCollisions>| **condition),
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
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        toggle_debug.run_if(input_just_pressed(KeyCode::Tab)),
    );

    #[cfg(debug_assertions)]
    {
        bevy_mod_debugdump::print_schedule_graph(&mut app, PostUpdate);
        let dot = bevy_mod_debugdump::schedule_graph_dot(
            &mut app,
            FixedUpdate,
            &bevy_mod_debugdump::schedule_graph::Settings::default(),
        );
        print!("{dot}");
    }

    // configure_ambiguity_detection(app.main_mut());

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

fn toggle_debug(mut collisions: ResMut<DebugCollisions>) {
    **collisions = !**collisions;
}

#[allow(unused)]
fn configure_ambiguity_detection(sub_app: &mut SubApp) {
    let mut schedules = sub_app.world_mut().resource_mut::<Schedules>();
    for (_, schedule) in schedules.iter_mut() {
        schedule.set_build_settings(ScheduleBuildSettings {
            // NOTE: you can change this to `LogLevel::Ignore` to easily see the current number of ambiguities.
            ambiguity_detection: LogLevel::Warn,
            use_shortnames: false,
            ..default()
        });
    }
}
