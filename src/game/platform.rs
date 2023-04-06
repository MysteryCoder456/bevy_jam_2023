use bevy::prelude::*;

use super::SPRITE_SCALE;
use crate::{components::RectCollider, GameAssets, GameState};

pub struct SpawnPlatformEvent(pub Vec2);

#[derive(Component)]
struct Platform;

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPlatformEvent>()
            .add_system(spawn_platform.in_set(OnUpdate(GameState::Level)));
    }
}

fn spawn_platform(
    mut events: EventReader<SpawnPlatformEvent>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    for event in events.iter() {
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
            RectCollider {
                size: Vec2::new(64., 16.) * SPRITE_SCALE,
            },
        ));
    }
}
