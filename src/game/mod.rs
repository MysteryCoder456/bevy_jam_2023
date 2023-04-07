use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use platform::{PlatformPlugin, SpawnPlatformEvent};
use player::PlayerPlugin;
use serde::{Deserialize, Serialize};

use crate::{
    components::{Gravity, RectCollider, Velocity},
    GameData, GameState,
};

mod platform;
mod player;

const SPRITE_SCALE: f32 = 3.;
const FIXED_TIMESTEP: f32 = 1. / 60.;
const GRAVITY: f32 = 50.;

#[derive(Serialize, Deserialize)]
struct LevelData {
    platforms: Vec<Vec2>,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlayerPlugin)
            .add_plugin(PlatformPlugin)
            .add_system(spawn_world.in_schedule(OnEnter(GameState::Level)))
            .add_systems(
                (gravity_system, velocity_system, collision_system)
                    .chain()
                    .in_set(OnUpdate(GameState::Level))
                    .in_schedule(CoreSchedule::FixedUpdate),
            )
            .insert_resource(FixedTime::new_from_secs(FIXED_TIMESTEP));
    }
}

fn spawn_world(mut platform_events: EventWriter<SpawnPlatformEvent>, game_data: Res<GameData>) {
    let filepath = format!("levels/level{}.json", game_data.current_level);
    let level_file = std::fs::File::open(filepath).unwrap();
    let level_data: LevelData = serde_json::from_reader(level_file).unwrap();

    platform_events.send_batch(
        level_data
            .platforms
            .iter()
            .map(|pos| SpawnPlatformEvent(*pos)),
    );
}

fn velocity_system(time: Res<FixedTime>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut tf, velocity) in query.iter_mut() {
        tf.translation += velocity.0.extend(0.) * time.period.as_secs_f32();
    }
}

fn gravity_system(mut query: Query<(&mut Velocity, &Gravity)>) {
    for (mut velocity, gravity) in query.iter_mut() {
        velocity.0 += gravity.0 * GRAVITY;
    }
}

fn collision_system(
    mut movable_query: Query<(&mut Transform, &mut Velocity, &RectCollider)>,
    static_query: Query<(&Transform, &RectCollider), Without<Velocity>>,
) {
    for (mut movable_tf, mut movable_vel, movable_col) in movable_query.iter_mut() {
        for (static_tf, static_col) in static_query.iter() {
            let collision = collide(
                movable_tf.translation,
                movable_col.size,
                static_tf.translation,
                static_col.size,
            );

            match collision {
                Some(Collision::Top) | Some(Collision::Inside) => {
                    movable_vel.0.y = 0.;
                    movable_tf.translation.y =
                        static_tf.translation.y + (static_col.size.y + movable_col.size.y) / 2.;
                }
                Some(Collision::Bottom) => {
                    movable_vel.0.y = 0.;
                    movable_tf.translation.y =
                        static_tf.translation.y - (static_col.size.y + movable_col.size.y) / 2.;
                }
                Some(Collision::Left) => {
                    movable_vel.0.x = 0.;
                    movable_tf.translation.x =
                        static_tf.translation.x - (static_col.size.x + movable_col.size.x) / 2.;
                }
                Some(Collision::Right) => {
                    movable_vel.0.x = 0.;
                    movable_tf.translation.x =
                        static_tf.translation.x + (static_col.size.x + movable_col.size.x) / 2.;
                }
                None => {}
            }
        }
    }
}
