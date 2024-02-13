use bevy::prelude::*;
use rand::distributions::{Distribution, Standard};

use super::player::Player;
use crate::{components::RectCollisionShape, GameState};

#[derive(Clone, Copy, Reflect)]
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

#[derive(Event)]
pub struct ApplySideEffectEvent(pub SideEffect);

pub struct SideEffectsPlugin;

impl Plugin for SideEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ApplySideEffectEvent>().add_systems(
            Update,
            apply_side_effect
                .run_if(in_state(GameState::Level).and_then(on_event::<ApplySideEffectEvent>())),
        );
    }
}

fn apply_side_effect(
    mut player_query: Query<(&mut Transform, &mut RectCollisionShape, &mut Player)>,
    mut events: EventReader<ApplySideEffectEvent>,
) {
    if let Ok((mut player_tf, mut player_col, mut player)) = player_query.get_single_mut() {
        for event in events.read() {
            match event.0 {
                SideEffect::Shrink => {
                    player_tf.scale *= 0.73;
                    player_col.size *= 0.73;
                    player.jump_multiplier *= 0.85;
                }
                SideEffect::Speed => player.speed_multiplier *= 1.5,
                SideEffect::Slowness => player.speed_multiplier *= 0.8,
            }
        }
    }
}
