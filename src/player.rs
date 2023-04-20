use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::LdtkEntityAppExt, LdtkEntity, LdtkLevel, Respawn};
use bevy_rapier2d::prelude::Collider;
use leafwing_input_manager::prelude::*;

use crate::{
    constants::PLAYER_DIM,
    game_state::GameState,
    physics::{Acceleration, Gravity, GravityDirection, JumpState, PhysicsSettings, Velocity},
    sfx::SfxHandles,
};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<JumpAction>::default())
            .add_plugin(InputManagerPlugin::<MovementAction>::default())
            .add_system(after_player_spawned.in_schedule(OnEnter(GameState::SpawnLevel)))
            .add_systems(
                (
                    control_jump,
                    control_movement,
                    sprite_orientation,
                    player_dies,
                )
                    .in_set(GameState::Playing),
            )
            .add_startup_system(load_player_handle)
            .register_ldtk_entity::<PlayerBundle>("Spawn_Point");
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

#[derive(Bundle, LdtkEntity)]
pub struct PlayerBundle {
    player: Player,
    #[sprite_bundle("pixel-cat.png")]
    sprite: SpriteBundle,
    velocity: Velocity,
    acceleration: Acceleration,
    g_dir: GravityDirection,
    gravity: Gravity,
    jump_state: JumpState,
}

fn after_player_spawned(mut commands: Commands, q: Query<Entity, Added<Player>>) {
    for e in &q {
        commands.entity(e).insert((
            InputManagerBundle::<JumpAction> {
                action_state: ActionState::default(),
                input_map: InputMap::new([(KeyCode::Space, JumpAction::Jump)]),
            },
            InputManagerBundle::<MovementAction> {
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
            Collider::cuboid(PLAYER_DIM.x / 2., PLAYER_DIM.y / 2.),
        ));
    }
}

fn load_player_handle(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PlayerSprite {
        handle: asset_server.load("pixel-cat.png"),
    })
}

fn control_jump(
    mut q: Query<(
        &mut Velocity,
        &mut JumpState,
        &mut Gravity,
        &GravityDirection,
        &ActionState<JumpAction>,
    )>,
    settings: Res<PhysicsSettings>,
    audio: Res<Audio>,
    sfx: Res<SfxHandles>,
) {
    for (mut v, mut jump_state, mut g, g_dir, action_state) in q.iter_mut() {
        if action_state.just_pressed(JumpAction::Jump) {
            if !jump_state.on_ground {
                return;
            }
            v.0 -= settings.initial_jump_speed * g_dir.as_vec2();
            jump_state.on_ground = false;
            jump_state.turned_this_jump = false;
            audio.play(sfx.jump.clone());
        }

        g.0 = if action_state.pressed(JumpAction::Jump) {
            settings.gravity_pressed
        } else {
            settings.gravity_unpressed
        };
    }
}

fn control_movement(
    mut q: Query<(
        &mut Velocity,
        &ActionState<MovementAction>,
        &GravityDirection,
    )>,
    settings: Res<PhysicsSettings>,
) {
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
                + (dir.forward().as_vec2() * val).normalize() * settings.horizontal_speed;
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

fn player_dies(
    player: Query<&Transform, With<Player>>,
    mut commands: Commands,
    audio: Res<Audio>,
    sfx: Res<SfxHandles>,
    level: Query<Entity, With<Handle<LdtkLevel>>>,
    mut state: ResMut<NextState<GameState>>,
) {
    for t in &player {
        if t.translation.y < -100.
            || t.translation.y > 800.
            || t.translation.x > 800.
            || t.translation.x < -100.
        {
            audio.play(sfx.death.clone());
            for e in &level {
                commands.entity(e).insert(Respawn);
            }
            state.set(GameState::SpawnLevel);
        }
    }
}
