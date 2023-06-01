use crate::{game_state::GameState, goals::Goal};
use bevy::{asset::LoadState, prelude::*};
use bevy_ecs_ldtk::{LdtkAsset, LdtkWorldBundle, LevelSelection};

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelSelection::Index(0));

        app.add_system(
            restart.run_if(in_state(GameState::Playing).or_else(in_state(GameState::WinScreen))),
        );

        app.add_system(setup_ldtk.in_schedule(OnExit(GameState::StartMenu)))
            .add_system(check_load_status.run_if(in_state(GameState::LoadLevel)));

        app.add_system(spawn_done.run_if(in_state(GameState::SpawnLevel)));
        app.add_systems(
            (level_complete, skip_level).distributive_run_if(in_state(GameState::Playing)),
        );
    }
}

fn setup_ldtk(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("levels/levels.ldtk"),
        ..default()
    });
}

fn check_load_status(
    ldtk_handle: Query<&Handle<LdtkAsset>>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<GameState>>,
) {
    let handle = ldtk_handle.single();
    if asset_server.get_load_state(handle.clone()) != LoadState::Loaded {
        return;
    }

    state.set(GameState::SpawnLevel);
}

fn spawn_done(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Playing);
}

fn level_complete(
    mut commands: Commands,
    q: Query<(), With<Goal>>,
    mut ldtk_events: EventReader<AssetEvent<LdtkAsset>>,
    mut state: ResMut<NextState<GameState>>,
    ldtk_entity: Query<(Entity, &Handle<LdtkAsset>)>,
    ldtks: Res<Assets<LdtkAsset>>,
    mut level_selection: ResMut<LevelSelection>,
    mut skip_level_done: Local<bool>,
) {
    for e in &mut ldtk_events {
        if let AssetEvent::Modified { handle: _ } = e {
            state.set(GameState::LoadLevel);
            *skip_level_done = true;
            return;
        }
    }
    if q.is_empty() && !*skip_level_done {
        if let LevelSelection::Index(index) = *level_selection {
            let (e, h) = ldtk_entity.single();
            let ldtk = ldtks.get(h).unwrap(); // TODO: this line panics on escape sometimes

            let (length, _) = ldtk.iter_levels().size_hint();
            if index + 1 < length {
                // go to next level
                state.set(GameState::SpawnLevel);
                *level_selection = LevelSelection::Index(index + 1);
            } else {
                // no more levels
                commands.entity(e).despawn_recursive();
                state.set(GameState::WinScreen);
            }
        } else {
            panic!("Only LevelSelection::Index is supported");
        }
    } else if q.is_empty() {
        *skip_level_done = false;
    }
}

fn restart(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
    mut level: ResMut<LevelSelection>,
    ldtk: Query<Entity, With<Handle<LdtkAsset>>>,
) {
    if keyboard.pressed(KeyCode::Escape) {
        state.set(GameState::StartMenu);
        *level = LevelSelection::Index(0);
        if !ldtk.is_empty() {
            commands.entity(ldtk.single()).despawn_recursive();
        }
    }
}

fn skip_level(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
    ldtk_entity: Query<(Entity, &Handle<LdtkAsset>)>,
    ldtks: Res<Assets<LdtkAsset>>,
    mut level_selection: ResMut<LevelSelection>,
) {
    if keyboard.just_pressed(KeyCode::Key0) {
        if let LevelSelection::Index(index) = *level_selection {
            let (e, h) = ldtk_entity.single();
            let ldtk = ldtks.get(h).unwrap();

            let (length, _) = ldtk.iter_levels().size_hint();
            if index + 1 < length {
                // go to next level
                state.set(GameState::SpawnLevel);
                *level_selection = LevelSelection::Index(index + 1);
            } else {
                // no more levels
                commands.entity(e).despawn_recursive();
                state.set(GameState::WinScreen);
            }
        } else {
            panic!("Only LevelSelection::Index is supported");
        }
    }
}
