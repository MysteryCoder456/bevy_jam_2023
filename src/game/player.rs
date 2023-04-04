use bevy::prelude::*;

use crate::{GameAssets, GameState};

const ANIMATION_SPEED: f32 = 12.; // in frames per second

#[derive(Component)]
struct Player {
    animation_timer: Timer,
    animation_length: usize,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_player.in_schedule(OnEnter(GameState::Level)))
            .add_systems((player_animation_system,).in_set(OnUpdate(GameState::Level)));
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
