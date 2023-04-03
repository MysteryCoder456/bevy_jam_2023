use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

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
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Expired!".to_owned(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_state::<GameState>()
        .add_startup_systems((setup_camera, setup_assets))
        .add_system(spawn_main_menu.in_schedule(OnEnter(GameState::MainMenu)))
        .run();
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
    };

    commands.insert_resource(ui_assets);
}

fn spawn_main_menu(mut commands: Commands, ui_assets: Res<UIAssets>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|n| {
            n.spawn(TextBundle::from_section(
                "Some Text",
                TextStyle {
                    font: ui_assets.font.clone(),
                    font_size: 40.,
                    color: Color::BLACK,
                },
            ));
        });
}
