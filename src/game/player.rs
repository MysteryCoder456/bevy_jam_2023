use bevy::prelude::*;

use super::SPRITE_SCALE;
use crate::{
    components::{Gravity, RectCollider, Velocity},
    GameAssets, GameState,
};

const ANIMATION_SPEED: f32 = 16.; // in frames per second
const RUN_SPEED: f32 = 350.;
const JUMP_SPEED: f32 = 1000.;

#[derive(Component)]
struct Player {
    animation_timer: Timer,
    animation_length: usize,
}

#[derive(States, Default, Clone, Debug, Hash, Eq, PartialEq)]
enum PlayerState {
    #[default]
    Idle,
    Running,
    Jumping,
    Falling,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<PlayerState>()
            .add_system(spawn_player.in_schedule(OnEnter(GameState::Level)))
            .add_systems(
                (
                    player_state_system,
                    player_atlas_change_system.run_if(state_changed::<PlayerState>()),
                    player_animation_system,
                )
                    .chain()
                    .in_set(OnUpdate(GameState::Level)),
            )
            .add_systems(
                (player_movement_system,)
                    .in_set(OnUpdate(GameState::Level))
                    .in_schedule(CoreSchedule::FixedUpdate),
            );
    }
}

fn spawn_player(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: game_assets.player_idle.clone(),
            transform: Transform {
                scale: Vec3::ONE * SPRITE_SCALE,
                ..Default::default()
            },
            ..Default::default()
        },
        Player {
            animation_timer: Timer::new(
                std::time::Duration::from_secs_f32(1. / ANIMATION_SPEED),
                TimerMode::Repeating,
            ),
            animation_length: 15,
        },
        Velocity(Vec2::ZERO),
        Gravity(Vec2::NEG_Y),
        RectCollider {
            size: Vec2::new(14., 32.) * SPRITE_SCALE,
        },
    ));
}

fn player_state_system(
    mut player_state: ResMut<NextState<PlayerState>>,
    query: Query<&Velocity, (With<Player>, Changed<Velocity>)>,
) {
    if let Ok(velocity) = query.get_single() {
        let next_state = if velocity.0.y > 0. {
            PlayerState::Jumping
        } else if velocity.0.y < 0. {
            PlayerState::Falling
        } else if velocity.0.x != 0. {
            PlayerState::Running
        } else {
            PlayerState::Idle
        };

        player_state.set(next_state);
    }
}

fn player_atlas_change_system(
    player_state: Res<State<PlayerState>>,
    game_assets: Res<GameAssets>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut Handle<TextureAtlas>,
        &mut TextureAtlasSprite,
        &mut Player,
    )>,
) {
    if let Ok((mut atlas, mut sprite, mut player)) = query.get_single_mut() {
        // FIXME: change the atlases for jumping and falling once the assets are made
        let new_atlas = match player_state.0 {
            PlayerState::Idle => game_assets.player_idle.clone(),
            PlayerState::Running => game_assets.player_run.clone(),
            PlayerState::Jumping => game_assets.player_run.clone(),
            PlayerState::Falling => game_assets.player_run.clone(),
        };

        if *atlas != new_atlas {
            player.animation_length = texture_atlases.get(&new_atlas).unwrap().textures.len();
            *atlas = new_atlas;
            sprite.index = 0;
        }
    }
}

fn player_animation_system(
    time: Res<Time>,
    mut query: Query<(&mut TextureAtlasSprite, &mut Player)>,
) {
    if let Ok((mut sprite, mut player)) = query.get_single_mut() {
        player.animation_timer.tick(time.delta());

        if player.animation_timer.finished() {
            sprite.index = (sprite.index + 1) % player.animation_length;
        }
    }
}

fn player_movement_system(
    kb: Res<Input<KeyCode>>,
    player_state: Res<State<PlayerState>>,
    mut query: Query<(&mut Velocity, &mut TextureAtlasSprite), With<Player>>,
) {
    if let Ok((mut velocity, mut sprite)) = query.get_single_mut() {
        let x_direction = kb.pressed(KeyCode::D) as i32 - kb.pressed(KeyCode::A) as i32;

        if x_direction < 0 {
            sprite.flip_x = true;
        } else if x_direction > 0 {
            sprite.flip_x = false;
        }

        velocity.0.x = x_direction as f32 * RUN_SPEED;

        if kb.just_pressed(KeyCode::W) {
            match player_state.0 {
                PlayerState::Idle | PlayerState::Running => velocity.0.y = JUMP_SPEED,
                _ => {}
            }
        }
    }
}
