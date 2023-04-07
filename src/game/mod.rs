use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use floating_label::{FloatingLabelPlugin, SpawnFloatingLabelEvent};
use pill::{PillPlugin, SpawnPillEvent};
use platform::{PlatformPlugin, SpawnPlatformEvent};
use player::PlayerPlugin;
use serde::{Deserialize, Serialize};

use crate::{
    components::{Gravity, RectCollisionShape, Velocity},
    GameData, GameState, UIAssets,
};

mod floating_label;
mod pill;
mod platform;
mod player;

const SPRITE_SCALE: f32 = 3.;
const FIXED_TIMESTEP: f32 = 1. / 60.;
const GRAVITY: f32 = 50.;

#[derive(Serialize, Deserialize)]
struct LevelData {
    platforms: Vec<Vec2>,
    pills: Vec<Vec2>,
    labels: Vec<(String, Vec2)>,
}

#[derive(Component)]
struct HUD;

#[derive(Component)]
struct CollectedLabel;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlayerPlugin)
            .add_plugin(PlatformPlugin)
            .add_plugin(PillPlugin)
            .add_plugin(FloatingLabelPlugin)
            .add_systems((spawn_world, spawn_hud).in_schedule(OnEnter(GameState::Level)))
            .add_systems(
                (gravity_system, velocity_system, collision_system)
                    .chain()
                    .in_set(OnUpdate(GameState::Level))
                    .in_schedule(CoreSchedule::FixedUpdate),
            )
            .insert_resource(FixedTime::new_from_secs(FIXED_TIMESTEP));
    }
}

fn spawn_world(
    mut platform_events: EventWriter<SpawnPlatformEvent>,
    mut pill_events: EventWriter<SpawnPillEvent>,
    mut label_events: EventWriter<SpawnFloatingLabelEvent>,
    game_data: Res<GameData>,
) {
    let filepath = format!("levels/level{}.json", game_data.current_level);
    let level_file = std::fs::File::open(filepath).unwrap(); // FIXME: doesn't work on WASM
    let level_data: LevelData = serde_json::from_reader(level_file).unwrap();

    platform_events.send_batch(
        level_data
            .platforms
            .iter()
            .map(|pos| SpawnPlatformEvent(*pos)),
    );

    pill_events.send_batch(level_data.pills.iter().map(|pos| SpawnPillEvent(*pos)));

    label_events.send_batch(
        level_data
            .labels
            .iter()
            .map(|(text, pos)| SpawnFloatingLabelEvent(text.clone(), *pos)),
    );
}

fn spawn_hud(mut commands: Commands, ui_assets: Res<UIAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    padding: UiRect::all(Val::Px(10.)),
                    ..Default::default()
                },
                ..Default::default()
            },
            HUD,
        ))
        .with_children(|hud| {
            hud.spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Auto),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexEnd,
                    ..Default::default()
                },
                ..Default::default()
            })
            .with_children(|top_row| {
                let style = TextStyle {
                    font: ui_assets.font.clone(),
                    font_size: 30.,
                    color: Color::BLACK,
                };

                top_row.spawn((
                    TextBundle::from_sections([
                        TextSection::new("Collected: ", style.clone()),
                        TextSection::new("0", style.clone()),
                    ]),
                    CollectedLabel,
                ));
            });
        });
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
    mut movable_query: Query<(&mut Transform, &mut Velocity, &RectCollisionShape)>,
    static_query: Query<(&Transform, &RectCollisionShape), Without<Velocity>>,
) {
    for (mut movable_tf, mut movable_vel, movable_col) in movable_query.iter_mut() {
        for (static_tf, static_col) in static_query.iter() {
            if !(movable_col.collide && static_col.collide) {
                continue;
            }

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
