use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*};
use bevy_aseprite_ultra::prelude::{Animation, AnimationState, AseSpriteAnimation, Aseprite};
use bevy_ecs_ldtk::{prelude::LdtkEntityAppExt, LdtkEntity, LdtkProjectHandle, Respawn};
use leafwing_input_manager::prelude::*;

use crate::{
    collisions::{CollisionEvents, PositionDelta, Ray, Rect},
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
        app.init_schedule(InputProcessing);
        let mut order = app.world_mut().resource_mut::<MainScheduleOrder>();
        order.insert_after(PreUpdate, InputProcessing);

        app.add_plugins(InputManagerPlugin::<JumpAction>::default())
            .add_plugins(InputManagerPlugin::<MovementAction>::default())
            .add_systems(Update, after_player_spawned)
            .add_systems(InputProcessing, (control_jump, control_movement))
            .add_systems(
                Update,
                (sprite_orientation, player_dies, animate_player).in_set(GameState::Playing),
            )
            .add_systems(Startup, load_player_handle)
            .register_ldtk_entity::<PlayerBundle>("Spawn_Point");
    }
}

#[derive(ScheduleLabel, Hash, Eq, PartialEq, Clone, Default, Debug)]
struct InputProcessing;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum JumpAction {
    Jump,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum MovementAction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Component, Default, Reflect)]
pub struct Player;

#[derive(Resource)]
pub struct PlayerSprite {
    // used to keep the player sprite asset loaded
    #[allow(unused)]
    pub handle: Handle<Aseprite>,
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct PlayerBundle {
    player: Player,
    #[sprite("pixel-cat.png")]
    sprite: Sprite,
    velocity: Velocity,
    acceleration: Acceleration,
    g_dir: GravityDirection,
    gravity: Gravity,
    on_ground: OnGround,
    jump_state: JumpState,
}

fn after_player_spawned(
    mut commands: Commands,
    q: Query<(Entity, &Transform), Added<Player>>,
    asset_server: Res<AssetServer>,
) {
    for (e, t) in &q {
        commands
            .entity(e)
            .insert((
                Name::new("Player"),
                AseSpriteAnimation {
                    aseprite: asset_server.load("pixel-cat.aseprite"),
                    animation: Animation::tag("idle"),
                },
                InputManagerBundle::<JumpAction> {
                    action_state: ActionState::default(),
                    input_map: InputMap::new([(JumpAction::Jump, KeyCode::Space)])
                        .with_multiple([(JumpAction::Jump, GamepadButton::South)]),
                },
                InputManagerBundle::<MovementAction> {
                    action_state: ActionState::default(),
                    input_map: InputMap::new([
                        // wasd
                        (MovementAction::Left, KeyCode::KeyA),
                        (MovementAction::Right, KeyCode::KeyD),
                        (MovementAction::Up, KeyCode::KeyW),
                        (MovementAction::Down, KeyCode::KeyS),
                        // arrow keys
                        (MovementAction::Left, KeyCode::ArrowLeft),
                        (MovementAction::Right, KeyCode::ArrowRight),
                        (MovementAction::Up, KeyCode::ArrowUp),
                        (MovementAction::Down, KeyCode::ArrowDown),
                    ])
                    .with_multiple([
                        // game pad
                        (MovementAction::Left, GamepadButton::DPadLeft),
                        (MovementAction::Right, GamepadButton::DPadRight),
                        (MovementAction::Up, GamepadButton::DPadUp),
                        (MovementAction::Down, GamepadButton::DPadDown),
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
                children.spawn((
                    Ray(Direction::Down.as_vec2() * RAY_LENGTH),
                    Transform::from_translation(
                        Vec2::new(-PLAYER_DIM.x / 2., -PLAYER_DIM.y / 2.).extend(0.0),
                    ),
                ));
                children.spawn((
                    Ray(Direction::Down.as_vec2() * RAY_LENGTH),
                    Transform::from_translation(
                        Vec2::new(PLAYER_DIM.x / 2., -PLAYER_DIM.y / 2.).extend(0.0),
                    ),
                ));

                // spawn hit box used for player collisions with wall and goals
                children.spawn(Rect(PLAYER_DIM));
            });
    }
}

fn load_player_handle(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PlayerSprite {
        handle: asset_server.load("pixel-cat.aseprite"),
    });
}

fn control_jump(
    mut commands: Commands,
    mut q: Query<(
        &mut Velocity,
        &mut OnGround,
        &mut JumpState,
        &mut Gravity,
        &GravityDirection,
        &ActionState<JumpAction>,
    )>,
    settings: Res<PhysicsSettings>,
    sfx: Res<SfxHandles>,
) {
    for (mut v, mut on_ground, mut jump_state, mut g, g_dir, action_state) in q.iter_mut() {
        if action_state.just_pressed(&JumpAction::Jump) {
            if !on_ground.0 {
                return;
            }
            v.0 -= settings.initial_jump_speed * g_dir.as_vec2();
            on_ground.0 = false;
            jump_state.turned_this_jump = false;
            commands.spawn((
                AudioPlayer::new(sfx.jump.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }

        g.0 = if action_state.pressed(&JumpAction::Jump) {
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
        if action.pressed(&MovementAction::Down) {
            temp_v.y -= 1.0;
        }
        if action.pressed(&MovementAction::Up) {
            temp_v.y += 1.0;
        }
        if action.pressed(&MovementAction::Left) {
            temp_v.x -= 1.0;
        }
        if action.pressed(&MovementAction::Right) {
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

// TODO: consider combining this with the control movement system
fn animate_player(
    mut commands: Commands,
    player: Query<
        (
            Entity,
            Ref<Velocity>,
            Ref<OnGround>,
            &mut AseSpriteAnimation,
        ),
        With<Player>,
    >,
    mut moving: Local<bool>,
    sprite: Res<PlayerSprite>,
) {
    let Ok((player, velocity, on_ground, animation)) = player.get_single() else {
        return;
    };
    let currently_moving = velocity.0.length_squared() > 0.0;
    if *moving != currently_moving || on_ground.is_changed() {
        match (currently_moving, on_ground.0) {
            (true, true) => {
                // walk
                commands.entity(player).insert(AseSpriteAnimation {
                    aseprite: sprite.handle.clone(),
                    animation: Animation::tag("walk"),
                });
            }
            (false, true) => {
                // idle
                commands.entity(player).insert(AseSpriteAnimation {
                    aseprite: sprite.handle.clone(),
                    animation: Animation::tag("idle"),
                });
            }
            (true, false) => {
                // transition to jumping animation
            }
            (false, false) => {
                // do nothing
            }
        }
    }
    *moving = currently_moving;
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
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    sfx: Res<SfxHandles>,
    level: Query<Entity, With<LdtkProjectHandle>>,
    mut state: ResMut<NextState<GameState>>,
) {
    for t in &player {
        if t.translation.y < -100.
            || t.translation.y > 800.
            || t.translation.x > 800.
            || t.translation.x < -100.
        {
            commands.spawn((
                AudioPlayer::new(sfx.death.clone()),
                PlaybackSettings::DESPAWN,
            ));
            for e in &level {
                commands.entity(e).insert(Respawn);
            }
            state.set(GameState::SpawnLevel);
        }
    }
}
