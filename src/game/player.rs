use bevy::prelude::*;

use crate::{
    components::{Gravity, Velocity},
    GameAssets, GameState,
};

const ANIMATION_SPEED: f32 = 12.; // in frames per second
const PLAYER_SPEED: f32 = 400.;

#[derive(Component)]
struct Player {
    animation_timer: Timer,
    animation_length: usize,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_player.in_schedule(OnEnter(GameState::Level)))
            .add_systems((player_animation_system,).in_set(OnUpdate(GameState::Level)))
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
                scale: Vec3::new(3., 3., 1.),
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
    ));
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
    mut query: Query<(&mut Velocity, &mut TextureAtlasSprite), With<Player>>,
) {
    if let Ok((mut velocity, mut sprite)) = query.get_single_mut() {
        let x_direction = kb.pressed(KeyCode::D) as i32 - kb.pressed(KeyCode::A) as i32;

        if x_direction < 0 {
            sprite.flip_x = true;
        } else if x_direction > 0 {
            sprite.flip_x = false;
        }

        velocity.0.x = x_direction as f32 * PLAYER_SPEED;
    }
}
