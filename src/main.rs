use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_kira_audio::prelude::*;
use bincode::{Decode, Encode};
use game::GamePlugin;
use main_menu::MainMenuPlugin;

#[cfg(feature = "inspector")]
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};

mod components;
mod game;
mod main_menu;

#[derive(Component)]
struct MainCamera;

#[derive(States, Default, Clone, Debug, Hash, Eq, PartialEq)]
enum GameState {
    #[default]
    MainMenu,
    Level,
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
}

#[derive(Resource)]
struct AudioAssets {
    bg_music: Handle<bevy_kira_audio::AudioSource>,
    player_jump: Handle<bevy_kira_audio::AudioSource>,
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
    .add_plugin(MainMenuPlugin)
    .add_plugin(GamePlugin)
    .add_startup_systems((
        setup_camera,
        setup_assets,
        setup_game_data,
        setup_audio_channels,
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

    let game_assets = GameAssets {
        player_idle,
        player_run,
        player_jump,
        player_fall,
        platform: asset_server.load("platform/platform.png"),
        pill,
    };

    let audio_assets = AudioAssets {
        bg_music: asset_server.load("music/OST/OST.wav"),
        player_jump: asset_server.load("sounds/jump/jump.wav"),
    };

    commands.insert_resource(ui_assets);
    commands.insert_resource(game_assets);
    commands.insert_resource(audio_assets);
}

fn setup_game_data(mut commands: Commands) {
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

fn setup_audio_channels(
    bgm: Res<AudioChannel<BackgroundMusicChannel>>,
    sfx: Res<AudioChannel<SFXChannel>>,
) {
    bgm.set_volume(0.7);
    sfx.set_volume(1.0);
}
