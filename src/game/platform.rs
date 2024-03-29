use bevy::prelude::*;

use super::SPRITE_SCALE;
use crate::{components::RectCollisionShape, GameAssets, GameState};

#[derive(Event)]
pub struct SpawnPlatformEvent(pub Vec2);

#[derive(Component)]
struct Platform;

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPlatformEvent>()
            .add_systems(
                Update,
                spawn_platform
                    .run_if(in_state(GameState::Level).and_then(on_event::<SpawnPlatformEvent>())),
            )
            .add_systems(OnExit(GameState::Level), despawn_platforms);
    }
}

fn spawn_platform(
    mut events: EventReader<SpawnPlatformEvent>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    for event in events.read() {
        commands.spawn((
            SpriteBundle {
                texture: game_assets.platform.clone(),
                transform: Transform {
                    translation: event.0.extend(0.),
                    scale: Vec3::ONE * SPRITE_SCALE,
                    ..Default::default()
                },
                ..Default::default()
            },
            Platform,
            RectCollisionShape {
                size: Vec2::new(64., 16.) * SPRITE_SCALE,
                collide: true,
            },
        ));
    }
}

fn despawn_platforms(mut commands: Commands, query: Query<Entity, With<Platform>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
