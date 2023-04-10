use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::prelude::*;

// a ground entity
#[derive(Component)]
pub struct Ground;

#[derive(Bundle)]
pub struct GroundBundle {
    ground: Ground,
    #[bundle]
    sprite: SpriteBundle,
    collider: Collider,
    // sensor: Sensor,
}

impl GroundBundle {
    pub fn new(dimensions: Vec2, position: Vec2) -> GroundBundle {
        GroundBundle {
            ground: Ground,
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(dimensions),
                    anchor: Anchor::Center,
                    color: Color::rgb_u8(92, 114, 125),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(1.0)),
                ..default()
            },
            collider: Collider::cuboid(dimensions.x / 2., dimensions.y / 2.),
            // sensor: Sensor,
        }
    }
}
