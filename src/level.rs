use crate::{game_state::GameState, goals::GoalBundle, ground::GroundBundle, player::PlayerSprite};
use bevy::{prelude::*, reflect::TypeUuid};

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_level.in_schedule(OnEnter(GameState::LoadLevel)))
            .add_system(load_done.run_if(in_state(GameState::LoadLevel)));
    }
}

#[derive(Resource)]
pub struct CurrentLevel(pub Level);

#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "fa33f4f6-fdb9-43f7-9df7-06ad8ff035ec"]
pub struct Level {
    pub spawn: Vec2,
    pub grounds: Vec<GroundConfig>,
    pub goals: Vec<Vec2>,
}
#[derive(serde::Deserialize)]
pub struct GroundConfig {
    pub dim: Vec2,
    pub pos: Vec2,
}

fn spawn_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_level: Res<CurrentLevel>,
) {
    commands.insert_resource(PlayerSprite {
        handle: asset_server.load("pixel-cat.png"),
    });

    for config in &current_level.0.grounds {
        commands.spawn(GroundBundle::new(config.dim, config.pos));
    }

    for goal in &current_level.0.goals {
        commands.spawn(GoalBundle::new(goal));
    }
}

fn load_done(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Playing);
}
