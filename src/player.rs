use bevy::prelude::*;
use bevy_rapier2d::prelude::Collider;
use leafwing_input_manager::prelude::*;

use crate::{
    constants::PLAYER_DIM,
    game_state::GameState,
    physics::{Acceleration, Direction, Gravity, GravityDirection, JumpState, Velocity},
};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<JumpAction>::default())
            .add_plugin(InputManagerPlugin::<MovementAction>::default())
            .add_systems(
                (
                    control_jump,
                    control_movement,
                    player_dies,
                    sprite_orientation,
                    respawn,
                )
                    .in_set(GameState::Playing),
            );
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum JumpAction {
    Jump,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum MovementAction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Component, Default)]
pub struct Player;

#[derive(Resource)]
pub struct PlayerSprite {
    pub handle: Handle<Image>,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    #[bundle]
    jump_action: InputManagerBundle<JumpAction>,
    #[bundle]
    movement_action: InputManagerBundle<MovementAction>,
    #[bundle]
    sprite: SpriteBundle,
    velocity: Velocity,
    acceleration: Acceleration,
    g_dir: GravityDirection,
    gravity: Gravity,
    jump_state: JumpState,
    collider: Collider,
}

impl PlayerBundle {
    pub fn new(texture: Handle<Image>) -> PlayerBundle {
        PlayerBundle {
            player: Player,
            jump_action: InputManagerBundle::<JumpAction> {
                action_state: ActionState::default(),
                input_map: InputMap::new([(KeyCode::Space, JumpAction::Jump)]),
            },
            movement_action: InputManagerBundle::<MovementAction> {
                action_state: ActionState::default(),
                input_map: InputMap::new([
                    (KeyCode::A, MovementAction::Left),
                    (KeyCode::D, MovementAction::Right),
                    (KeyCode::W, MovementAction::Up),
                    (KeyCode::S, MovementAction::Down),
                    (KeyCode::Left, MovementAction::Left),
                    (KeyCode::Right, MovementAction::Right),
                    (KeyCode::Up, MovementAction::Up),
                    (KeyCode::Down, MovementAction::Down),
                ]),
            },
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(PLAYER_DIM),
                    ..default()
                },
                texture,
                transform: Transform::from_xyz(0.0, -250., 0.),
                ..default()
            },
            velocity: Velocity::default(),
            acceleration: Acceleration::default(),
            g_dir: GravityDirection(Direction::Down),
            gravity: Gravity(0.0),
            jump_state: JumpState {
                on_ground: true,
                turned_this_jump: false,
                last_horizontal_movement_dir: Direction::Left,
                last_vertical_movement_dir: Direction::Down,
            },
            collider: Collider::cuboid(PLAYER_DIM.x / 2., PLAYER_DIM.y / 2.),
        }
    }
}

fn control_jump(
    mut q: Query<(
        &mut Velocity,
        &mut JumpState,
        &mut Gravity,
        &GravityDirection,
        &ActionState<JumpAction>,
    )>,
) {
    const INITIAL_JUMP_SPEED: f32 = 400.0;
    const GRAVITY_PRESSED: f32 = 40.0;
    const GRAVITY_UNPRESSED: f32 = 200.0;
    for (mut v, mut jump_state, mut g, g_dir, action_state) in q.iter_mut() {
        if action_state.just_pressed(JumpAction::Jump) {
            if !jump_state.on_ground {
                return;
            }
            v.0 -= INITIAL_JUMP_SPEED * g_dir.as_vec2();
            jump_state.on_ground = false;
            jump_state.turned_this_jump = false;
        }

        g.0 = if action_state.pressed(JumpAction::Jump) {
            GRAVITY_PRESSED
        } else {
            GRAVITY_UNPRESSED
        }
    }
}

fn control_movement(
    mut q: Query<(
        &mut Velocity,
        &ActionState<MovementAction>,
        &GravityDirection,
    )>,
) {
    const HORIZONTAL_SPEED: f32 = 200.0;
    for (mut v, action, dir) in &mut q {
        let mut temp_v = Vec2::ZERO;
        if action.pressed(MovementAction::Down) {
            temp_v.y -= 1.0;
        }
        if action.pressed(MovementAction::Up) {
            temp_v.y += 1.0;
        }
        if action.pressed(MovementAction::Left) {
            temp_v.x -= 1.0;
        }
        if action.pressed(MovementAction::Right) {
            temp_v.x += 1.0;
        }

        let val = dir.forward().as_vec2().dot(temp_v);
        if val != 0.0 {
            v.0 = v.0 * dir.as_vec2().abs()
                + (dir.forward().as_vec2() * val).normalize() * HORIZONTAL_SPEED;
        } else {
            v.0 *= dir.as_vec2().abs();
        }
    }
}

fn sprite_orientation(
    mut player: Query<(&mut Sprite, &Velocity, &GravityDirection), With<Player>>,
) {
    for (mut s, v, g) in &mut player {
        let forward_speed = g.forward().as_vec2().dot(v.0);
        if forward_speed > 0. {
            s.flip_x = false;
        } else if forward_speed < 0. {
            s.flip_x = true;
        }
    }
}

fn player_dies(q: Query<(Entity, &Transform), With<Player>>, mut commands: Commands) {
    for (e, t) in &q {
        if t.translation.y < -400.
            || t.translation.y > 400.
            || t.translation.x > 400.
            || t.translation.x < -400.
        {
            commands.entity(e).despawn();
        }
    }
}

fn respawn(mut commands: Commands, q: Query<(), With<Player>>, handle: Res<PlayerSprite>) {
    if q.is_empty() {
        commands.spawn(PlayerBundle::new(handle.handle.clone()));
    }
}
