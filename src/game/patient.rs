use bevy::prelude::*;

use super::SPRITE_SCALE;
use crate::{components::RectCollisionShape, GameAssets, GameState};

const ANIMATION_SPEED: f32 = 3.;

pub struct SpawnPatientEvent(pub Vec2);

#[derive(Component)]
pub struct Patient {
    animation_timer: Timer,
    animation_length: usize,
}

pub struct PatientPlugin;

impl Plugin for PatientPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPatientEvent>()
            .add_system(
                spawn_patient
                    .in_set(OnUpdate(GameState::Level))
                    .run_if(on_event::<SpawnPatientEvent>()),
            )
            .add_system(despawn_patient.in_schedule(OnExit(GameState::Level)))
            .add_system(patient_animation_system.in_set(OnUpdate(GameState::Level)));
    }
}

fn spawn_patient(
    mut events: EventReader<SpawnPatientEvent>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    for event in events.iter() {
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: game_assets.patient.clone(),
                transform: Transform {
                    translation: event.0.extend(0.),
                    scale: Vec3::ONE * SPRITE_SCALE,
                    ..Default::default()
                },
                ..Default::default()
            },
            Patient {
                animation_timer: Timer::new(
                    std::time::Duration::from_secs_f32(1. / ANIMATION_SPEED),
                    TimerMode::Repeating,
                ),
                animation_length: 4,
            },
            RectCollisionShape {
                size: Vec2::new(14., 32.) * SPRITE_SCALE,
                collide: false,
            },
        ));
    }
}

fn despawn_patient(mut commands: Commands, query: Query<Entity, With<Patient>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn();
    }
}

fn patient_animation_system(
    time: Res<Time>,
    mut query: Query<(&mut TextureAtlasSprite, &mut Patient)>,
) {
    if let Ok((mut sprite, mut patient)) = query.get_single_mut() {
        patient.animation_timer.tick(time.delta());

        if patient.animation_timer.finished() {
            sprite.index = (sprite.index + 1) % patient.animation_length;
        }
    }
}
