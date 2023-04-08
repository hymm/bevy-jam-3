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
    mut collisions: EventReader<CollisionEvent>,
) {
    for collision in &mut collisions {
        dbg!(collision);
        match collision {
            CollisionEvent::Started(maybe_goal, maybe_player, _) => {
                if goals.contains(*maybe_goal) && player.contains(*maybe_player) {
                    commands.entity(*maybe_goal).despawn();
                }
            }
            _ => {}
        }
    }
}

pub struct GoalPlugin;
impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(goal_collision_detection.in_schedule(CoreSchedule::FixedUpdate));
    }
}
