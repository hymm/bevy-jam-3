use crate::{
    game_state::GameState,
    goals::{Goal, GoalBundle},
    ground::{Ground, GroundBundle},
    player::{Player, PlayerBundle, PlayerSprite},
};
use bevy::{asset::LoadState, prelude::*, reflect::TypeUuid};

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Levels>();

        app.add_system(load_level.in_schedule(OnEnter(GameState::LoadLevel)))
            .add_system(check_load_status.run_if(in_state(GameState::LoadLevel)));

        app.add_system(spawn_level.in_schedule(OnEnter(GameState::SpawnLevel)))
            .add_system(spawn_done.run_if(in_state(GameState::SpawnLevel)));

        app.add_system(despawn_level.in_schedule(OnExit(GameState::Playing)))
            .add_systems(
                (monitor_level_changes, level_complete)
                    .distributive_run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource)]
pub struct Levels {
    pub current_level: usize,
    pub levels: Vec<String>,
}
impl Default for Levels {
    fn default() -> Self {
        Self {
            current_level: 0,
            levels: vec![
                "learn_to_move".to_string(),
                "box".to_string(),
                "fall_for_it".to_string(),
                "stairs".to_string(),
                "upsidedown-u".to_string(),
                "3_die".to_string(),
                "5_floating_boxes".to_string(),
            ],
        }
    }
}
impl Levels {
    fn current_level(&self) -> &String {
        &self.levels[self.current_level]
    }

    // returns false if there are no more levels
    fn advance_level(&mut self) -> bool {
        if self.current_level + 1 < self.levels.len() {
            self.current_level += 1;
            true
        } else {
            false
        }
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

fn load_level(mut commands: Commands, asset_server: Res<AssetServer>, levels: Res<Levels>) {
    commands.insert_resource(CurrentLevel(
        asset_server.load("levels/".to_string() + levels.current_level() + ".level.ron"),
    ));
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
    let player_handle = asset_server.load("pixel-cat.png");
    commands.insert_resource(PlayerSprite {
        handle: player_handle.clone(),
    });

    let level = levels.get(&current_level.0).unwrap();

    commands.spawn(PlayerBundle::new(player_handle, level.spawn));

    for config in &level.grounds {
        commands.spawn(GroundBundle::new(config.dim, config.pos));
    }

    for goal in &level.goals {
        commands.spawn(GoalBundle::new(goal));
    }
}

fn despawn_level(
    mut commands: Commands,
    grounds: Query<Entity, With<Ground>>,
    player: Query<Entity, With<Player>>,
    goals: Query<Entity, With<Goal>>,
) {
    for e in &grounds {
        commands.entity(e).despawn();
    }

    for e in &player {
        commands.entity(e).despawn();
    }

    for e in &goals {
        commands.entity(e).despawn();
    }
}

fn spawn_done(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Playing);
}

fn monitor_level_changes(
    mut events: EventReader<AssetEvent<Level>>,
    current_level: Res<CurrentLevel>,
    mut state: ResMut<NextState<GameState>>,
) {
    for e in &mut events {
        if let AssetEvent::Modified { handle } = e {
            if *handle == current_level.0 {
                state.set(GameState::SpawnLevel);
            }
        }
    }
}

fn level_complete(
    q: Query<(), With<Goal>>,
    mut state: ResMut<NextState<GameState>>,
    mut levels: ResMut<Levels>,
) {
    if q.is_empty() {
        if levels.advance_level() {
            state.set(GameState::LoadLevel);
        } else {
            state.set(GameState::WinScreen);
        }
    }
}
