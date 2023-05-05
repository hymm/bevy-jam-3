use crate::collisions::Rect;
use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::LdtkIntCellAppExt, LdtkIntCell};

use crate::game_state::GameState;

pub struct GroundPlugin;
impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_int_cell::<GroundBundle>(1)
            .add_system(after_ground_spawned.in_schedule(OnEnter(GameState::SpawnLevel)));
    }
}

// a ground entity
#[derive(Component, Default)]
pub struct Ground;

#[derive(Bundle, LdtkIntCell, Default)]
pub struct GroundBundle {
    ground: Ground,
}

fn after_ground_spawned(mut commands: Commands, q: Query<Entity, Added<Ground>>) {
    for e in &q {
        commands.entity(e).with_children(|children| {
            children.spawn(Rect::new(24.0, 24.0));
        });
    }
}
