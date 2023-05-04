use bevy::prelude::*;

use crate::collisions::CollisionType;

pub const PLAYER_DIM: Vec2 = Vec2::new(30.0, 20.0);

#[derive(Component, Clone)]
pub enum CollisionTypes {
    Player,
    Goal,
    Ground,
}

impl CollisionType for CollisionTypes {}
