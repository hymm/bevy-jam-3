use bevy::prelude::*;

#[derive(States, SystemSet, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameState {
    #[default]
    StartMenu,
    LoadLevel,
    SpawnLevel,
    Playing,
    UnloadLevel,
    Respawn,
    WinScreen,
}

pub struct GameStatePlugin;
impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().configure_sets(
            Update,
            GameState::Playing.run_if(in_state(GameState::Playing)),
        );
    }
}
