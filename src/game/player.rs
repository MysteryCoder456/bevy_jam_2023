use bevy::{prelude::*, sprite::collide_aabb::collide};
use bevy_kira_audio::prelude::*;

use super::{
    patient::Patient,
    pill::{Pill, SpawnPillEvent},
    platform::SpawnPlatformEvent,
    side_effects::ApplySideEffectEvent,
    CollectedLabel, LevelData, Levels, SPRITE_SCALE,
};
use crate::{
    components::{Gravity, RectCollisionShape, Velocity},
    AudioAssets, GameAssets, GameData, GameState, MainCamera, SFXChannel,
};

const ANIMATION_SPEED: f32 = 16.; // frames per second
const RUN_SPEED: f32 = 350.;
const JUMP_SPEED: f32 = 1000.;

#[derive(Component, Reflect, Default)]
pub struct Player {
    pub animation_timer: Timer,
    pub animation_length: usize,
    pub medicines_collected: u32,
    pub jump_multiplier: f32,
    pub speed_multiplier: f32,
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
            .add_systems(OnEnter(GameState::Level), spawn_player)
            .add_systems(OnExit(GameState::Level), despawn_player)
            .add_systems(
                Update,
                (
                    player_state_system,
                    player_atlas_change_system
                        .run_if(state_changed::<PlayerState>())
                        .after(player_state_system),
                    player_animation_system.after(player_atlas_change_system),
                    camera_follow_system,
                    player_jump_system,
                    player_out_of_bounds_system,
                )
                    .run_if(in_state(GameState::Level)),
            )
            .add_systems(
                FixedUpdate,
                (
                    player_movement_system,
                    player_pill_collision_system,
                    player_patient_collision_system,
                )
                    .run_if(in_state(GameState::Level)),
            );

        #[cfg(debug_assertions)]
        app.add_systems(Update, spawn_game_entity.run_if(in_state(GameState::Level)));

        #[cfg(feature = "inspector")]
        app.register_type::<Player>();
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
            medicines_collected: 0,
            jump_multiplier: 1.,
            speed_multiplier: 1.,
        },
        Velocity(Vec2::ZERO),
        Gravity(Vec2::NEG_Y),
        RectCollisionShape {
            size: Vec2::new(14., 32.) * SPRITE_SCALE,
            collide: true,
        },
    ));
}

fn despawn_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn();
    }
}

/// Used for debugging only.
#[cfg(debug_assertions)]
fn spawn_game_entity(
    kb: Res<Input<KeyCode>>,
    query: Query<&Transform, With<Player>>,
    mut platform_events: EventWriter<SpawnPlatformEvent>,
    mut pill_events: EventWriter<SpawnPillEvent>,
) {
    if let Ok(player_tf) = query.get_single() {
        let player_pos = player_tf.translation.truncate();

        if kb.just_pressed(KeyCode::P) {
            platform_events.send(SpawnPlatformEvent(player_pos - Vec2::new(0., 90.)));
        }

        if kb.just_pressed(KeyCode::O) {
            pill_events.send(SpawnPillEvent {
                position: player_pos + Vec2::new(50., 0.),
                side_effect: rand::random(),
            });
        }
    }
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
        let new_atlas = match player_state.get() {
            PlayerState::Idle => game_assets.player_idle.clone(),
            PlayerState::Running => game_assets.player_run.clone(),
            PlayerState::Jumping => game_assets.player_jump.clone(),
            PlayerState::Falling => game_assets.player_fall.clone(),
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

fn camera_follow_system(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    player_query: Query<&Transform, (With<Player>, Without<MainCamera>)>,
) {
    if let Ok(mut camera_tf) = camera_query.get_single_mut() {
        if let Ok(player_tf) = player_query.get_single() {
            camera_tf.translation = player_tf.translation;
        }
    }
}

fn player_pill_collision_system(
    mut commands: Commands,
    sfx: Res<AudioChannel<SFXChannel>>,
    audio_assets: Res<AudioAssets>,
    mut side_effect_events: EventWriter<ApplySideEffectEvent>,
    mut player_query: Query<(&Transform, &RectCollisionShape, &mut Player)>,
    pill_query: Query<(Entity, &Transform, &RectCollisionShape, &Pill), Without<Player>>,
    mut label_query: Query<&mut Text, With<CollectedLabel>>,
) {
    if let Ok((player_tf, player_col, mut player)) = player_query.get_single_mut() {
        for (pill_entity, pill_tf, pill_col, pill) in pill_query.iter() {
            let collision = collide(
                player_tf.translation,
                player_col.size,
                pill_tf.translation,
                pill_col.size,
            );

            if collision.is_some() {
                sfx.play(audio_assets.pill_collect.clone());
                player.medicines_collected += 1;
                commands.entity(pill_entity).despawn();

                if let Ok(mut text) = label_query.get_single_mut() {
                    text.sections[1].value = player.medicines_collected.to_string();
                }

                side_effect_events.send(ApplySideEffectEvent(pill.side_effect));
            }
        }
    }
}

fn player_patient_collision_system(
    mut game_state: ResMut<NextState<GameState>>,
    level_assets: Res<Assets<LevelData>>,
    levels: Res<Levels>,
    game_data: Res<GameData>,
    player_query: Query<(&Transform, &RectCollisionShape, &Player)>,
    patient_query: Query<(&Transform, &RectCollisionShape), (With<Patient>, Without<Player>)>,
) {
    if let Ok((player_tf, player_col, player)) = player_query.get_single() {
        if let Ok((patient_tf, patient_col)) = patient_query.get_single() {
            let collision = collide(
                player_tf.translation,
                player_col.size,
                patient_tf.translation,
                patient_col.size,
            );

            if collision.is_some() {
                let level_handle = levels.0.get(&game_data.current_level).unwrap();
                let level_data = level_assets.get(level_handle).unwrap();

                if player.medicines_collected == level_data.pill_goal {
                    game_state.set(GameState::LevelCompleted);
                }
            }
        }
    }
}

fn player_movement_system(
    kb: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut TextureAtlasSprite, &Player)>,
) {
    if let Ok((mut velocity, mut sprite, player)) = query.get_single_mut() {
        let x_direction = kb.pressed(KeyCode::D) as i32 - kb.pressed(KeyCode::A) as i32;

        if x_direction < 0 {
            sprite.flip_x = true;
        } else if x_direction > 0 {
            sprite.flip_x = false;
        }

        velocity.0.x = x_direction as f32 * RUN_SPEED * player.speed_multiplier;
    }
}

fn player_jump_system(
    kb: Res<Input<KeyCode>>,
    player_state: Res<State<PlayerState>>,
    sfx: Res<AudioChannel<SFXChannel>>,
    audio_assets: Res<AudioAssets>,
    mut query: Query<(&mut Velocity, &Player)>,
) {
    if kb.just_pressed(KeyCode::W) {
        if let Ok((mut velocity, player)) = query.get_single_mut() {
            match player_state.get() {
                PlayerState::Idle | PlayerState::Running => {
                    velocity.0.y = JUMP_SPEED * player.jump_multiplier;
                    sfx.play(audio_assets.player_jump.clone());
                }
                _ => {}
            }
        }
    }
}

fn player_out_of_bounds_system(
    mut game_state: ResMut<NextState<GameState>>,
    query: Query<&Transform, With<Player>>,
) {
    if let Ok(player_tf) = query.get_single() {
        if player_tf.translation.y < -5000. {
            game_state.set(GameState::GameOver);
        }
    }
}
