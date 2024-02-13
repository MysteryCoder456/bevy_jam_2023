use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
    reflect::TypeUuid,
    sprite::collide_aabb::{collide, Collision},
    utils::HashMap,
};
use floating_label::{FloatingLabelPlugin, SpawnFloatingLabelEvent};
use patient::{PatientPlugin, SpawnPatientEvent};
use pill::{PillPlugin, SpawnPillEvent};
use platform::{PlatformPlugin, SpawnPlatformEvent};
use player::PlayerPlugin;
use serde::Deserialize;
use side_effects::SideEffectsPlugin;
use thiserror::Error;

use crate::{
    components::{Gravity, RectCollisionShape, Velocity},
    GameData, GameState, UIAssets,
};

mod floating_label;
mod patient;
mod pill;
mod platform;
mod player;
mod side_effects;

const SPRITE_SCALE: f32 = 3.;
const FIXED_FREQUENCY: f64 = 60.;
const GRAVITY: f32 = 50.;
const MAX_LEVELS: usize = 5;

#[derive(Resource, Asset, TypePath)]
struct Levels(HashMap<usize, Handle<LevelData>>);

#[derive(Deserialize, TypeUuid, Asset, TypePath)]
#[uuid = "2b2bea01-bf6b-475d-90d6-ccaae422666f"]
struct LevelData {
    platforms: Vec<Vec2>,
    pills: Vec<Vec2>,
    labels: Vec<(String, Vec2)>,
    time_limit: u64,
    pill_goal: u32,
    goal: Vec2,
}

#[derive(Default)]
struct LevelDataLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
enum LevelDataLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not parse JSON: {0}")]
    JsonParseError(#[from] serde_json::error::Error),
}

impl AssetLoader for LevelDataLoader {
    type Asset = LevelData;
    type Settings = ();
    type Error = LevelDataLoaderError;

    fn extensions(&self) -> &[&str] {
        &["json"]
    }

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let data = serde_json::from_slice(&bytes)?;
            Ok(data)
        })
    }
}

#[derive(Component)]
struct HUD;

#[derive(Component)]
struct CollectedLabel;

#[derive(Component)]
struct StopwatchLabel(Timer);

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerPlugin)
            .add_plugins(PlatformPlugin)
            .add_plugins(PillPlugin)
            .add_plugins(FloatingLabelPlugin)
            .add_plugins(PatientPlugin)
            .add_plugins(SideEffectsPlugin)
            .init_asset::<LevelData>()
            .init_asset_loader::<LevelDataLoader>()
            .add_systems(Startup, load_level_data)
            .add_systems(OnEnter(GameState::Level), (spawn_world, spawn_hud))
            .add_systems(OnExit(GameState::Level), despawn_hud)
            .add_systems(
                FixedUpdate,
                (gravity_system, velocity_system, collision_system)
                    .chain()
                    .run_if(in_state(GameState::Level)),
            )
            .add_systems(Update, stopwatch_system.run_if(in_state(GameState::Level)))
            .insert_resource(Time::<Fixed>::from_hz(FIXED_FREQUENCY));
    }
}

fn load_level_data(mut commands: Commands, asset_server: Res<AssetServer>) {
    let levels = (1..=MAX_LEVELS)
        .map(|n| {
            let file_path = format!("levels/level{}.json", n);
            (n, asset_server.load(file_path))
        })
        .collect();

    commands.insert_resource(Levels(levels));
}

fn spawn_world(
    mut platform_events: EventWriter<SpawnPlatformEvent>,
    mut pill_events: EventWriter<SpawnPillEvent>,
    mut label_events: EventWriter<SpawnFloatingLabelEvent>,
    mut patient_events: EventWriter<SpawnPatientEvent>,
    game_data: Res<GameData>,
    level_assets: Res<Assets<LevelData>>,
    levels: Res<Levels>,
) {
    let level_handle = levels.0.get(&game_data.current_level).unwrap();
    let level_data = level_assets.get(level_handle).unwrap();

    platform_events.send_batch(
        level_data
            .platforms
            .iter()
            .map(|pos| SpawnPlatformEvent(*pos)),
    );

    pill_events.send_batch(level_data.pills.iter().map(|pos| SpawnPillEvent {
        position: *pos,
        side_effect: rand::random(),
    }));

    label_events.send_batch(
        level_data
            .labels
            .iter()
            .map(|(text, pos)| SpawnFloatingLabelEvent(text.clone(), *pos)),
    );

    patient_events.send(SpawnPatientEvent(level_data.goal));
}

fn spawn_hud(
    mut commands: Commands,
    ui_assets: Res<UIAssets>,
    game_data: Res<GameData>,
    level_assets: Res<Assets<LevelData>>,
    levels: Res<Levels>,
) {
    let level_handle = levels.0.get(&game_data.current_level).unwrap();
    let level_data = level_assets.get(level_handle).unwrap();

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
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
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
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
                        TextSection::new(format!("/{}", level_data.pill_goal), style.clone()),
                    ]),
                    CollectedLabel,
                ));

                top_row.spawn((
                    TextBundle::from_sections([
                        TextSection::new("Time Left: ", style.clone()),
                        TextSection::new(level_data.time_limit.to_string(), style.clone()),
                        TextSection::new("s", style.clone()),
                    ]),
                    StopwatchLabel(Timer::new(
                        std::time::Duration::from_secs(level_data.time_limit),
                        TimerMode::Once,
                    )),
                ));
            });
        });
}

fn despawn_hud(mut commands: Commands, query: Query<Entity, With<HUD>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

fn stopwatch_system(
    time: Res<Time>,
    mut game_state: ResMut<NextState<GameState>>,
    mut query: Query<(&mut Text, &mut StopwatchLabel)>,
) {
    if let Ok((mut text, mut stopwatch)) = query.get_single_mut() {
        stopwatch.0.tick(time.delta());
        text.sections[1].value = stopwatch.0.remaining().as_secs().to_string();

        if stopwatch.0.finished() {
            game_state.set(GameState::GameOver);
        }
    }
}

fn velocity_system(time: Res<Time<Fixed>>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut tf, velocity) in query.iter_mut() {
        tf.translation += velocity.0.extend(0.) * time.delta_seconds();
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
