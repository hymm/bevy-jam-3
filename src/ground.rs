use crate::{
    collisions::{CollisionData, CollisionEvents, CollisionSets, PositionDelta, Rect},
    constants::CollisionTypes,
    physics::{Acceleration, Direction, Gravity, GravityDirection, OnGround, Velocity},
    player::Player,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::{
    prelude::{LdtkEntityAppExt, LdtkIntCellAppExt},
    LdtkEntity, LdtkIntCell,
};

pub struct GroundPlugin;
impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_int_cell::<GroundBundle>(1)
            .register_ldtk_entity::<FallingGroundBundle>("Falling_Block")
            .add_systems(Startup, load_falling_block_sprite)
            // TODO: make these hooks
            .add_systems(Update, (after_ground_spawned, after_falling_ground_spawned))
            .add_systems(Update, fall_block_after_jump.in_set(CollisionSets::Consume));
    }
}

// a ground entity
#[derive(Component, Default)]
pub struct Ground;

#[derive(Bundle, LdtkIntCell, Default)]
pub struct GroundBundle {
    ground: Ground,
}

fn after_ground_spawned(
    mut commands: Commands,
    q: Query<Entity, (Added<Ground>, Without<FallingGround>)>,
) {
    for e in &q {
        commands
            .entity(e)
            .insert(CollisionTypes::Ground)
            .with_children(|children| {
                children.spawn(Rect(Vec2::new(24.0, 24.0)));
            });
    }
}

#[derive(Component, Default)]
pub struct FallingGround;

#[derive(Bundle, LdtkEntity, Default)]
pub struct FallingGroundBundle {
    falling_ground: FallingGround,
    ground: Ground,
    #[sprite("falling-block.png")]
    sprite: Sprite,
    g_dir: GravityDirection,
    gravity: Gravity,
    on_ground: OnGround,
    velocity: Velocity,
    acceleration: Acceleration,
    player_contact: PlayerContact,
}

#[derive(Component, Default)]
struct PlayerContact {
    pub is_in_contact: bool,
}

/// resource to keep the falling block sprite asset alive
#[derive(Resource)]
pub struct FallingBlockSprite {
    #[allow(unused)]
    sprite: Handle<Image>,
}

fn load_falling_block_sprite(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(FallingBlockSprite {
        sprite: asset_server.load("falling-block.png"),
    });
}

fn after_falling_ground_spawned(
    mut commands: Commands,
    mut q: Query<
        (
            Entity,
            &Transform,
            &mut Gravity,
            &mut GravityDirection,
            &mut OnGround,
        ),
        Added<FallingGround>,
    >,
) {
    for (e, t, mut g, mut g_dir, mut on_ground) in &mut q {
        g.0 = 200.0;
        on_ground.0 = true;
        g_dir.0 = Direction::Down;
        commands
            .entity(e)
            .insert((
                CollisionTypes::Ground,
                CollisionEvents::<CollisionTypes>::new(),
                PositionDelta {
                    origin: t.translation.truncate(),
                    ray: Vec2::ZERO,
                },
            ))
            .with_children(|children| {
                children.spawn(Rect(Vec2::new(71.0, 71.0)));
            });
    }
}

fn fall_block_after_jump(
    player_collisions: Query<
        (
            &OnGround,
            &GravityDirection,
            &CollisionEvents<CollisionTypes>,
        ),
        With<Player>,
    >,
    mut falling_blocks: Query<
        (&mut OnGround, &mut GravityDirection, &mut PlayerContact),
        (With<FallingGround>, Without<Player>),
    >,
    mut last_in_contact: Local<Vec<Entity>>,
) {
    let mut in_contact = Vec::with_capacity(10);
    if let Ok((on_ground, player_g_dir, player_collisions)) = player_collisions.single() {
        if on_ground.0 {
            for collision in &player_collisions.buffer {
                if let Ok((_, mut g_dir, mut player_contact)) =
                    falling_blocks.get_mut(collision.entity)
                {
                    in_contact.push(collision.entity);

                    if let CollisionData::Ray(ref data) = collision.data {
                        if data.toi < 2.0 && -collision.data.normal() == player_g_dir.as_vec2() {
                            player_contact.is_in_contact = true;
                            *g_dir = *player_g_dir;
                        }
                    }
                }
            }
        }
    }

    for e in &last_in_contact {
        if !in_contact.contains(e) {
            if let Ok((mut on_ground, _, mut player_contact)) = falling_blocks.get_mut(*e) {
                player_contact.is_in_contact = false;
                on_ground.0 = false;
            }
        }
    }

    *last_in_contact = in_contact;
}
