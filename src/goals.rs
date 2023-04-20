use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::LdtkEntityAppExt, LdtkEntity};
use bevy_rapier2d::prelude::*;

use crate::{game_state::GameState, player::Player, sfx::SfxHandles};

pub struct GoalPlugin;
impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(goal_collision_detection)
            .add_system(after_goal_spawned.in_schedule(OnEnter(GameState::SpawnLevel)))
            .add_startup_system(load_goal_images)
            .register_ldtk_entity::<GoalBundle>("Goal");
    }
}
#[derive(Component, Default)]
pub struct Goal;

#[derive(Bundle, LdtkEntity, Default)]
pub struct GoalBundle {
    goal: Goal,
    #[sprite_bundle("goal-mouse.png")]
    sprite: SpriteBundle,
    collider: Collider,
    active_events: ActiveEvents,
    sensor: Sensor,
}

#[derive(Resource, Default)]
pub struct GoalHandles {
    pub handles: Vec<Handle<Image>>,
}
impl GoalHandles {
    fn handle_paths() -> Vec<String> {
        vec![
            "goal-ball-yellow.png".to_string(),
            "goal-ball-blue.png".to_string(),
            "goal-ball-red.png".to_string(),
            "goal-mouse.png".to_string(),
            "goal-fish.png".to_string(),
        ]
    }
}

fn after_goal_spawned(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Handle<Image>), With<Goal>>,
) {
    for (e, mut h) in &mut q {
        commands.entity(e).insert(Collider::cuboid(7.5, 7.5));
    }
}

fn goal_collision_detection(
    mut commands: Commands,
    player: Query<Entity, With<Player>>,
    goals: Query<Entity, With<Goal>>,
    rapier: Res<RapierContext>,
    audio: Res<Audio>,
    sfx: Res<SfxHandles>,
) {
    for goal in &goals {
        for (entity1, entity2, _) in rapier.intersections_with(goal) {
            if player.contains(entity1) || player.contains(entity2) {
                commands.entity(goal).despawn();
                audio.play(sfx.goal.clone());
            }
        }
    }
}

fn load_goal_images(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut handles = GoalHandles::default();
    for path in GoalHandles::handle_paths() {
        let handle = asset_server.load(path);
        handles.handles.push(handle);
    }

    commands.insert_resource(handles);
}
