use crate::{game_state::GameState, goals::GoalBundle, ground::GroundBundle, player::PlayerSprite};
use bevy::{prelude::*, reflect::TypeUuid, asset::LoadState};

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_level.in_schedule(OnEnter(GameState::SpawnLevel)))
            .add_system(load_level.in_schedule(OnEnter(GameState::LoadLevel)))
            .add_system(check_load_status.run_if(in_state(GameState::LoadLevel)))
            .add_system(spawn_done.run_if(in_state(GameState::SpawnLevel)));
    }
}

#[derive(Resource)]
pub struct CurrentLevel(pub Handle<Level>);

#[derive(serde::Deserialize, TypeUuid, Debug)]
#[uuid = "fa33f4f6-fdb9-43f7-9df7-06ad8ff035ec"]
pub struct Level {
    pub spawn: Vec2,
    pub grounds: Vec<GroundConfig>,
    pub goals: Vec<Vec2>,
}
#[derive(serde::Deserialize, Debug)]
pub struct GroundConfig {
    pub dim: Vec2,
    pub pos: Vec2,
}

fn load_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CurrentLevel(asset_server.load("level_1.level.ron")));
}

fn check_load_status(
    current_level: Res<CurrentLevel>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<GameState>>,
) {
    if asset_server.get_load_state(current_level.0.clone()) != LoadState::Loaded {
        return;
    }

    state.set(GameState::SpawnLevel);
}

fn spawn_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_level: Res<CurrentLevel>,
    levels: Res<Assets<Level>>,
) {
    commands.insert_resource(PlayerSprite {
        handle: asset_server.load("pixel-cat.png"),
    });

    let level = levels.get(&current_level.0).unwrap();

    for config in &level.grounds {
        commands.spawn(GroundBundle::new(config.dim, config.pos));
    }

    for goal in &level.goals {
        commands.spawn(GoalBundle::new(goal));
    }
}

fn spawn_done(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Playing);
}
