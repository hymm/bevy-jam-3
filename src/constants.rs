use bevy::prelude::*;

pub const PLAYER_DIM: Vec2 = Vec2::new(30.0, 20.0);

#[derive(Component, Clone, PartialEq, Eq)]
pub enum CollisionTypes {
    Player,
    Goal,
    Ground,
}
