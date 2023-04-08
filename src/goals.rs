use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::Player;

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

impl GoalBundle {
    pub fn new(position: Vec2) -> GoalBundle {
        GoalBundle {
            goal: Goal,
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(15., 15.)),
                    color: Color::rgb(0.8, 0.5, 0.8),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(0.0)),
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
        for (entity, _, _) in rapier.intersections_with(goal) {
            if player.contains(entity) {
                commands.entity(goal).despawn();
            }
        }
    }
}

pub struct GoalPlugin;
impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(goal_collision_detection);
    }
}
