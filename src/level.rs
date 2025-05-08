use crate::{game_state::GameState, goals::Goal, physics::OnGround, player::Player};
use bevy::{asset::LoadState, prelude::*};
use bevy_ecs_ldtk::{
    assets::LdtkProject, prelude::RawLevelAccessor, LdtkProjectHandle, LdtkWorldBundle,
    LevelSelection,
};

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelSelection::index(0));

        app.add_systems(
            Update,
            restart.run_if(in_state(GameState::Playing).or(in_state(GameState::WinScreen))),
        );

        app.add_systems(OnExit(GameState::StartMenu), setup_ldtk)
            .add_systems(
                Update,
                check_load_status.run_if(in_state(GameState::LoadLevel)),
            );

        app.add_systems(Update, spawn_done.run_if(in_state(GameState::SpawnLevel)));
        app.add_systems(
            Update,
            (level_complete, skip_level).distributive_run_if(in_state(GameState::Playing)),
        );
    }
}

fn setup_ldtk(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("levels/levels.ldtk").into(),
        ..default()
    });
}

fn check_load_status(
    ldtk_handle: Query<&LdtkProjectHandle>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<GameState>>,
) {
    let handle = ldtk_handle.single();
    if matches!(
        asset_server.get_load_state(handle.clone()).unwrap(),
        LoadState::Loaded
    ) {
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
    mut ldtk_events: EventReader<AssetEvent<LdtkProject>>,
    mut state: ResMut<NextState<GameState>>,
    ldtk_entity: Query<(Entity, &LdtkProjectHandle)>,
    ldtks: Res<Assets<LdtkProject>>,
    mut level_selection: ResMut<LevelSelection>,
    mut skip_level_done: Local<bool>,
    player_grounded: Query<&OnGround, With<Player>>,
) {
    for e in ldtk_events.read() {
        if let AssetEvent::Modified { id: _ } = e {
            state.set(GameState::LoadLevel);
            *skip_level_done = true;
            return;
        }
    }
    if q.is_empty()
        && !*skip_level_done
        && player_grounded
            .get_single()
            .map_or(false, |grounded| grounded.0)
    {
        if let LevelSelection::Indices(index) = *level_selection {
            let (e, h) = ldtk_entity.single();
            let ldtk = ldtks.get(h).unwrap(); // TODO: this line panics on escape sometimes

            let (length, _) = ldtk.iter_raw_levels().size_hint();
            if index.level + 1 < length {
                // go to next level
                state.set(GameState::SpawnLevel);
                *level_selection = LevelSelection::index(index.level + 1);
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
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
    mut level: ResMut<LevelSelection>,
    ldtk: Query<Entity, With<LdtkProjectHandle>>,
) {
    if keyboard.pressed(KeyCode::Escape) {
        state.set(GameState::StartMenu);
        *level = LevelSelection::index(0);
        if !ldtk.is_empty() {
            commands.entity(ldtk.single()).despawn_recursive();
        }
    }
}

fn skip_level(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
    ldtk_entity: Query<(Entity, &LdtkProjectHandle)>,
    ldtks: Res<Assets<LdtkProject>>,
    mut level_selection: ResMut<LevelSelection>,
) {
    if keyboard.just_pressed(KeyCode::Digit0) {
        if let LevelSelection::Indices(index) = *level_selection {
            let (e, h) = ldtk_entity.single();
            let ldtk = ldtks.get(h).unwrap();

            let (length, _) = ldtk.iter_raw_levels().size_hint();
            if index.level + 1 < length {
                // go to next level
                state.set(GameState::SpawnLevel);
                *level_selection = LevelSelection::index(index.level + 1);
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
