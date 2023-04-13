use bevy::prelude::*;
use rand::distributions::{Distribution, Standard};

use super::SPRITE_SCALE;
use crate::{components::RectCollisionShape, GameAssets, GameState};

const ANIMATION_SPEED: f32 = 44.; // frames per second

pub struct SpawnPillEvent {
    pub position: Vec2,
    pub side_effect: SideEffect,
}

#[derive(Clone, Reflect)]
pub enum SideEffect {
    Shrink,
    Speed,
    Slowness,
}

impl Distribution<SideEffect> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> SideEffect {
        match rng.gen_range(0..=2) {
            0 => SideEffect::Shrink,
            1 => SideEffect::Speed,
            _ => SideEffect::Slowness,
        }
    }
}

#[derive(Component, Reflect)]
pub struct Pill {
    animation_timer: Timer,
    animation_length: usize,
    pub side_effect: SideEffect,
}

pub struct PillPlugin;

impl Plugin for PillPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPillEvent>()
            .add_system(
                spawn_pill
                    .in_set(OnUpdate(GameState::Level))
                    .run_if(on_event::<SpawnPillEvent>()),
            )
            .add_system(despawn_pills.in_schedule(OnExit(GameState::Level)))
            .add_system(pill_animation_system.in_set(OnUpdate(GameState::Level)));

        #[cfg(feature = "inspector")]
        app.register_type::<Pill>();
    }
}

fn spawn_pill(
    mut events: EventReader<SpawnPillEvent>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    for event in events.iter() {
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: game_assets.pill.clone(),
                transform: Transform {
                    translation: event.position.extend(0.),
                    scale: Vec3::ONE * SPRITE_SCALE,
                    ..Default::default()
                },
                ..Default::default()
            },
            Pill {
                animation_timer: Timer::new(
                    std::time::Duration::from_secs_f32(1. / ANIMATION_SPEED),
                    TimerMode::Repeating,
                ),
                animation_length: 45,
                side_effect: event.side_effect.clone(),
            },
            RectCollisionShape {
                size: Vec2::new(18., 22.),
                collide: false,
            },
        ));
    }
}

fn despawn_pills(mut commands: Commands, query: Query<Entity, With<Pill>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn pill_animation_system(time: Res<Time>, mut query: Query<(&mut TextureAtlasSprite, &mut Pill)>) {
    for (mut sprite, mut pill) in query.iter_mut() {
        pill.animation_timer.tick(time.delta());

        if pill.animation_timer.finished() {
            sprite.index = (sprite.index + 1) % pill.animation_length;
        }
    }
}
