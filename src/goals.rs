use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::player::Player;

pub struct GoalPlugin;
impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(goal_collision_detection)
            .add_startup_system(load_goal_images);
    }
}
#[derive(Component)]
pub struct Goal;

#[derive(Bundle)]
pub struct GoalBundle {
    goal: Goal,
    #[bundle]
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

impl GoalBundle {
    pub fn new(position: &Vec2, texture: &Handle<Image>) -> GoalBundle {
        GoalBundle {
            goal: Goal,
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(15., 15.)),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(0.0)),
                texture: texture.clone(),
                ..default()
            },
            collider: Collider::cuboid(7.5, 7.5),
            active_events: ActiveEvents::COLLISION_EVENTS,
            sensor: Sensor,
        }
    }
}

fn goal_collision_detection(
    mut commands: Commands,
    player: Query<Entity, With<Player>>,
    goals: Query<Entity, With<Goal>>,
    rapier: Res<RapierContext>,
) {
    for goal in &goals {
        for (entity1, entity2, _) in rapier.intersections_with(goal) {
            if player.contains(entity1) || player.contains(entity2) {
                commands.entity(goal).despawn();
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
