use crate::collisions::{CollisionEvents, CollisionSets, Rect};
use crate::constants::CollisionTypes;
use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::LdtkEntityAppExt, LdtkEntity};
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::sfx::SfxHandles;

pub struct GoalPlugin;
impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            goal_collision_detection.in_set(CollisionSets::Consume),
        )
        .add_systems(Update, after_goal_spawned)
        .add_systems(Startup, load_goal_images)
        .register_ldtk_entity::<GoalBundle>("Goal");
    }
}
#[derive(Component, Default)]
pub struct Goal;

#[derive(Bundle, LdtkEntity, Default)]
pub struct GoalBundle {
    goal: Goal,
    #[sprite("goal-mouse.png")]
    sprite: Sprite,
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
    mut q: Query<(Entity, &mut Sprite), Added<Goal>>,
    goal_handles: Res<GoalHandles>,
    mut rand: ResMut<GlobalRng>,
) {
    for (e, mut h) in &mut q {
        commands
            .entity(e)
            .insert((
                CollisionTypes::Goal,
                CollisionEvents::<CollisionTypes>::new(),
            ))
            .with_children(|children| {
                children.spawn(Rect(Vec2::new(15., 15.)));
            });

        // set a random image
        let index = rand.u8(0..goal_handles.handles.len() as u8) as usize;
        *h = goal_handles.handles[index].clone().into()
    }
}

fn goal_collision_detection(
    mut commands: Commands,
    mut goals: Query<(Entity, &mut CollisionEvents<CollisionTypes>), With<Goal>>,
    sfx: Res<SfxHandles>,
) {
    for (entity, mut collision_events) in &mut goals {
        for event in collision_events.buffer.drain(..) {
            if event.user_type == CollisionTypes::Player {
                commands.entity(entity).despawn();
                commands.spawn((
                    AudioPlayer::new(sfx.goal.clone()),
                    PlaybackSettings::DESPAWN,
                ));
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
