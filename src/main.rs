use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bincode::{Decode, Encode};
use main_menu::MainMenuPlugin;

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};

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

#[derive(Resource, Encode, Decode, Reflect)]
struct GameData {
    current_level: u32,
}

impl Default for GameData {
    fn default() -> Self {
        Self { current_level: 1 }
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Expired!".to_owned(),
            ..Default::default()
        }),
        ..Default::default()
    }))
    .add_state::<GameState>()
    .add_plugin(MainMenuPlugin)
    .add_startup_systems((setup_camera, setup_assets, setup_game_data));

    #[cfg(debug_assertions)]
    app.add_plugin(WorldInspectorPlugin::new())
        .register_type::<GameData>()
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

fn setup_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ui_assets = UIAssets {
        font: asset_server.load("fonts/Neucha-Regular.ttf"),
        button: asset_server.load("ui/button.png"),
        button_pressed: asset_server.load("ui/button_pressed.png"),
    };

    commands.insert_resource(ui_assets);
}

fn setup_game_data(mut commands: Commands) {
    let game_data_path = "game_data.bin";
    let config = bincode::config::standard();

    let game_data: GameData = if let Ok(mut file) = std::fs::File::open(game_data_path) {
        bincode::decode_from_std_read(&mut file, config).unwrap_or_default()
    } else {
        let default_game_data = GameData::default();
        let encoded = bincode::encode_to_vec(&default_game_data, config).unwrap();
        std::fs::write(game_data_path, &encoded).unwrap();
        default_game_data
    };

    commands.insert_resource(game_data);
}
