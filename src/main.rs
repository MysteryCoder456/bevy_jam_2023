use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod main_menu;

use main_menu::MainMenuPlugin;

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
    .add_startup_systems((setup_camera, setup_assets));

    #[cfg(debug_assertions)]
    app.add_plugin(WorldInspectorPlugin::new());

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
