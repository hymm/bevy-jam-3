use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::LdtkEntityAppExt, LdtkEntity, LdtkLevel, Respawn};
use leafwing_input_manager::{prelude::*, user_input::InputKind};

use crate::{
    collisions::{CollisionEvents, PositionDelta, RayBundle, RectBundle},
    constants::{CollisionTypes, PLAYER_DIM},
    game_state::GameState,
    physics::{
        Acceleration, Direction, Gravity, GravityDirection, JumpState, OnGround, PhysicsSettings,
        Velocity,
    },
    sfx::SfxHandles,
};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<JumpAction>::default())
            .add_plugin(InputManagerPlugin::<MovementAction>::default())
            .configure_set(
                InputProcessing
                    .after(CoreSet::PreUpdateFlush)
                    .before(CoreSet::FixedUpdate),
            )
            .add_system(after_player_spawned.in_schedule(OnEnter(GameState::SpawnLevel)))
            .add_systems((control_jump, control_movement).in_base_set(InputProcessing))
            .add_systems((sprite_orientation, player_dies).in_set(GameState::Playing))
            .add_startup_system(load_player_handle)
            .register_ldtk_entity::<PlayerBundle>("Spawn_Point");
    }
}

#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Default, Debug)]
#[system_set(base)]
struct InputProcessing;

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
    on_ground: OnGround,
    jump_state: JumpState,
}

fn after_player_spawned(mut commands: Commands, q: Query<(Entity, &Transform), Added<Player>>) {
    for (e, t) in &q {
        commands
            .entity(e)
            .insert((
                InputManagerBundle::<JumpAction> {
                    action_state: ActionState::default(),
                    input_map: InputMap::new([
                        (InputKind::Keyboard(KeyCode::Space), JumpAction::Jump),
                        (
                            InputKind::GamepadButton(GamepadButtonType::South),
                            JumpAction::Jump,
                        ),
                    ]),
                },
                InputManagerBundle::<MovementAction> {
                    action_state: ActionState::default(),
                    input_map: InputMap::new([
                        // wasd
                        (InputKind::Keyboard(KeyCode::A), MovementAction::Left),
                        (InputKind::Keyboard(KeyCode::D), MovementAction::Right),
                        (InputKind::Keyboard(KeyCode::W), MovementAction::Up),
                        (InputKind::Keyboard(KeyCode::S), MovementAction::Down),
                        // arrow keys
                        (InputKind::Keyboard(KeyCode::Left), MovementAction::Left),
                        (InputKind::Keyboard(KeyCode::Right), MovementAction::Right),
                        (InputKind::Keyboard(KeyCode::Up), MovementAction::Up),
                        (InputKind::Keyboard(KeyCode::Down), MovementAction::Down),
                        // game pad
                        (
                            InputKind::GamepadButton(GamepadButtonType::DPadLeft),
                            MovementAction::Left,
                        ),
                        (
                            InputKind::GamepadButton(GamepadButtonType::DPadRight),
                            MovementAction::Right,
                        ),
                        (
                            InputKind::GamepadButton(GamepadButtonType::DPadUp),
                            MovementAction::Up,
                        ),
                        (
                            InputKind::GamepadButton(GamepadButtonType::DPadDown),
                            MovementAction::Down,
                        ),
                    ]),
                },
                CollisionTypes::Player,
                CollisionEvents::<CollisionTypes>::new(),
                PositionDelta {
                    origin: t.translation.truncate(),
                    ray: Vec2::ZERO,
                },
            ))
            .with_children(|children| {
                // spawn some ray colliders
                const RAY_LENGTH: f32 = 15.0;
                // point down
                children.spawn(RayBundle::new(
                    Direction::Down.as_vec2() * RAY_LENGTH,
                    Vec2::new(-PLAYER_DIM.x / 2., -PLAYER_DIM.y / 2.),
                ));
                children.spawn(RayBundle::new(
                    Direction::Down.as_vec2() * RAY_LENGTH,
                    Vec2::new(PLAYER_DIM.x / 2., -PLAYER_DIM.y / 2.),
                ));

                // spawn hit box used for player collisions with wall and goals
                children.spawn(RectBundle::new(PLAYER_DIM));
            });
    }
}

fn load_player_handle(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PlayerSprite {
        handle: asset_server.load("pixel-cat.png"),
    });
}

fn control_jump(
    mut q: Query<(
        &mut Velocity,
        &mut OnGround,
        &mut JumpState,
        &mut Gravity,
        &GravityDirection,
        &ActionState<JumpAction>,
    )>,
    settings: Res<PhysicsSettings>,
    audio: Res<Audio>,
    sfx: Res<SfxHandles>,
) {
    for (mut v, mut on_ground, mut jump_state, mut g, g_dir, action_state) in q.iter_mut() {
        if action_state.just_pressed(JumpAction::Jump) {
            if !on_ground.0 {
                return;
            }
            v.0 -= settings.initial_jump_speed * g_dir.as_vec2();
            on_ground.0 = false;
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
