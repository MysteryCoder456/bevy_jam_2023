use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_kira_audio::prelude::*;
use bincode::{Decode, Encode};
use components::ScreenFade;
use game::GamePlugin;
use game_over::GameOverPlugin;
use main_menu::MainMenuPlugin;

#[cfg(feature = "inspector")]
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};

mod components;
mod game;
mod game_over;
mod main_menu;

#[derive(Component)]
struct MainCamera;

#[derive(States, Default, Clone, Debug, Hash, Eq, PartialEq)]
pub enum GameState {
    #[default]
    MainMenu,
    Level,
    GameOver,
    LevelCompleted,
}

#[derive(Resource)]
struct UIAssets {
    font: Handle<Font>,
    button: Handle<Image>,
    button_pressed: Handle<Image>,
}

#[derive(Resource)]
struct GameAssets {
    player_idle: Handle<TextureAtlas>,
    player_run: Handle<TextureAtlas>,
    player_jump: Handle<TextureAtlas>,
    player_fall: Handle<TextureAtlas>,
    platform: Handle<Image>,
    pill: Handle<TextureAtlas>,
    patient: Handle<TextureAtlas>,
}

#[derive(Resource)]
struct AudioAssets {
    bg_music: Handle<bevy_kira_audio::AudioSource>,
    player_jump: Handle<bevy_kira_audio::AudioSource>,
    pill_collect: Handle<bevy_kira_audio::AudioSource>,
}

#[derive(Resource, Encode, Decode, Reflect)]
struct GameData {
    current_level: usize,
}

impl Default for GameData {
    fn default() -> Self {
        Self { current_level: 1 }
    }
}

#[derive(Resource)]
struct BackgroundMusicChannel;

#[derive(Resource)]
struct SFXChannel;

#[derive(Default)]
struct SaveGameData;

struct SpawnScreenFader {
    fade_color: Color,
    fade_time: f32,
    next_state: GameState,
}

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Expiry Date".to_owned(),
                    fit_canvas_to_parent: true,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugin(AudioPlugin)
    .add_audio_channel::<BackgroundMusicChannel>()
    .add_audio_channel::<SFXChannel>()
    .add_state::<GameState>()
    .add_event::<SaveGameData>()
    .add_event::<SpawnScreenFader>()
    .add_plugin(MainMenuPlugin)
    .add_plugin(GamePlugin)
    .add_plugin(GameOverPlugin)
    .add_startup_systems((
        setup_camera,
        setup_assets,
        load_game_data,
        setup_audio_channels,
    ))
    .add_system(save_game_data.run_if(on_event::<SaveGameData>()))
    .add_system(next_level_system.in_schedule(OnEnter(GameState::LevelCompleted)))
    .add_systems((
        button_appearance_system,
        spawn_screen_fader.run_if(on_event::<SpawnScreenFader>()),
        screen_fade_system,
    ));

    #[cfg(feature = "inspector")]
    app.add_plugin(WorldInspectorPlugin::new())
        .register_type::<GameData>()
        .register_type::<components::Velocity>()
        .register_type::<components::Gravity>()
        .register_type::<components::RectCollisionShape>()
        .add_plugin(ResourceInspectorPlugin::<GameData>::default());

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::ANTIQUE_WHITE),
            },
            ..Default::default()
        },
        MainCamera,
    ));
}

fn setup_assets(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    let ui_assets = UIAssets {
        font: asset_server.load("fonts/Neucha-Regular.ttf"),
        button: asset_server.load("ui/button.png"),
        button_pressed: asset_server.load("ui/button_pressed.png"),
    };

    let player_idle_texture: Handle<Image> = asset_server.load("player/player_idle.png");
    let player_idle_atlas =
        TextureAtlas::from_grid(player_idle_texture, Vec2::new(32., 32.), 20, 1, None, None);
    let player_idle = texture_atlases.add(player_idle_atlas);

    let player_run_texture: Handle<Image> = asset_server.load("player/player_run.png");
    let player_run_atlas =
        TextureAtlas::from_grid(player_run_texture, Vec2::new(32., 32.), 5, 1, None, None);
    let player_run = texture_atlases.add(player_run_atlas);

    let player_jump_texture: Handle<Image> = asset_server.load("player/player_jump.png");
    let player_jump_atlas =
        TextureAtlas::from_grid(player_jump_texture, Vec2::new(32., 32.), 5, 1, None, None);
    let player_jump = texture_atlases.add(player_jump_atlas);

    let player_fall_texture: Handle<Image> = asset_server.load("player/player_fall.png");
    let player_fall_atlas =
        TextureAtlas::from_grid(player_fall_texture, Vec2::new(32., 32.), 3, 1, None, None);
    let player_fall = texture_atlases.add(player_fall_atlas);

    let pill_texture: Handle<Image> = asset_server.load("pill/pill.png");
    let pill_atlas = TextureAtlas::from_grid(pill_texture, Vec2::new(32., 32.), 45, 1, None, None);
    let pill = texture_atlases.add(pill_atlas);

    let patient_texture: Handle<Image> = asset_server.load("patient/patient.png");
    let patient_atlas =
        TextureAtlas::from_grid(patient_texture, Vec2::new(32., 32.), 4, 1, None, None);
    let patient = texture_atlases.add(patient_atlas);

    let game_assets = GameAssets {
        player_idle,
        player_run,
        player_jump,
        player_fall,
        platform: asset_server.load("platform/platform.png"),
        pill,
        patient,
    };

    let audio_assets = AudioAssets {
        bg_music: asset_server.load("music/OST/OST.wav"),
        player_jump: asset_server.load("sounds/jump/jump.wav"),
        pill_collect: asset_server.load("sounds/collect/collect.wav"),
    };

    commands.insert_resource(ui_assets);
    commands.insert_resource(game_assets);
    commands.insert_resource(audio_assets);
}

fn load_game_data(mut commands: Commands) {
    let game_data_path = "game_data.bin";
    let config = bincode::config::standard();

    let game_data: GameData = if let Ok(mut file) = std::fs::File::open(game_data_path) {
        bincode::decode_from_std_read(&mut file, config).unwrap_or_default()
    } else {
        let default_game_data = GameData::default();
        let encoded = bincode::encode_to_vec(&default_game_data, config).unwrap();
        _ = std::fs::write(game_data_path, &encoded);
        default_game_data
    };

    commands.insert_resource(game_data);
}

fn save_game_data(game_data: Res<GameData>) {
    let game_data_path = "game_data.bin";
    let config = bincode::config::standard();

    let encoded = bincode::encode_to_vec(game_data.into_inner(), config).unwrap();
    std::fs::write(game_data_path, &encoded).unwrap();
}

fn setup_audio_channels(
    bgm: Res<AudioChannel<BackgroundMusicChannel>>,
    sfx: Res<AudioChannel<SFXChannel>>,
) {
    bgm.set_volume(0.6);
    sfx.set_volume(1.0);
}

fn next_level_system(
    mut game_state: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameData>,
    mut events: EventWriter<SaveGameData>,
) {
    game_data.current_level += 1;
    game_state.set(GameState::Level);
    events.send_default();
}

fn button_appearance_system(
    mut query: Query<(&mut UiImage, &Interaction), (With<Button>, Changed<Interaction>)>,
    ui_assets: Res<UIAssets>,
) {
    for (mut ui_image, interaction) in query.iter_mut() {
        let new_image = match *interaction {
            Interaction::Clicked | Interaction::Hovered => ui_assets.button_pressed.clone(),
            _ => ui_assets.button.clone(),
        };

        *ui_image = UiImage::new(new_image);
    }
}

fn spawn_screen_fader(mut events: EventReader<SpawnScreenFader>, mut commands: Commands) {
    for event in events.iter() {
        commands.spawn((
            NodeBundle {
                background_color: BackgroundColor(Color::BLACK.with_a(0.)),
                style: Style {
                    position_type: PositionType::Absolute,
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    ..Default::default()
                },
                z_index: ZIndex::Global(5),
                ..Default::default()
            },
            ScreenFade {
                fade_color: event.fade_color,
                fade_timer: Timer::from_seconds(event.fade_time, TimerMode::Once),
                next_state: event.next_state.clone(),
            },
        ));
    }
}

fn screen_fade_system(
    time: Res<Time>,
    mut game_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut BackgroundColor, &mut components::ScreenFade)>,
) {
    for (entity, mut bg_color, mut fade) in query.iter_mut() {
        fade.fade_timer.tick(time.delta());
        bg_color.0 = fade
            .fade_color
            .with_a(fade.fade_timer.elapsed_secs() / fade.fade_timer.duration().as_secs_f32());

        if fade.fade_timer.finished() {
            game_state.set(fade.next_state.clone());
            commands.entity(entity).despawn_recursive();
        }
    }
}
