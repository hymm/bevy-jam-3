use crate::{game_state::GameState, goals::GoalBundle, ground::GroundBundle, player::PlayerSprite};
use bevy::prelude::*;

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_level.in_schedule(OnEnter(GameState::LoadLevel)))
            .add_system(load_done.run_if(in_state(GameState::LoadLevel)));
    }
}

fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PlayerSprite {
        handle: asset_server.load("pixel-cat.png"),
    });

    // bottom
    commands.spawn(GroundBundle::new(
        Vec2::new(600., 20.),
        Vec2::new(0.0, -300.0),
    ));

    // top
    // commands.spawn(GroundBundle::new(Vec2::new(600., 20.), Vec2::new(0., 300.)));

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

    // a goal
    commands.spawn(GoalBundle::new(Vec2::new(0., -45.)));
    // a goal
    commands.spawn(GoalBundle::new(Vec2::new(115., -160.)));
}

fn load_done(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Playing);
}
