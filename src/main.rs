use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

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
    .add_startup_systems((setup_camera, setup_assets))
    .add_system(spawn_main_menu.in_schedule(OnEnter(GameState::MainMenu)))
    .add_systems((button_appearance_system,).in_set(OnUpdate(GameState::MainMenu)));

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

fn spawn_main_menu(mut commands: Commands, ui_assets: Res<UIAssets>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                display: Display::Flex,
                gap: Size::new(Val::Undefined, Val::Px(8.)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|n| {
            n.spawn(TextBundle::from_section(
                "EXPIRED!",
                TextStyle {
                    font: ui_assets.font.clone(),
                    font_size: 60.,
                    color: Color::BLACK,
                },
            ));

            n.spawn(ButtonBundle {
                image: UiImage::new(ui_assets.button.clone()),
                style: Style {
                    padding: UiRect::new(Val::Px(25.), Val::Px(25.), Val::Px(14.), Val::Px(14.)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .with_children(|b| {
                b.spawn(TextBundle::from_section(
                    "Play",
                    TextStyle {
                        font: ui_assets.font.clone(),
                        font_size: 38.,
                        color: Color::BLACK,
                    },
                ));
            });
        });
}

fn button_appearance_system(
    mut query: Query<(&mut UiImage, &Interaction), Changed<Interaction>>,
    ui_assets: Res<UIAssets>,
) {
    for (mut ui_image, interaction) in query.iter_mut() {
        let new_image = match *interaction {
            Interaction::Clicked => ui_assets.button_pressed.clone(),
            _ => ui_assets.button.clone(),
        };

        *ui_image = UiImage::new(new_image);
    }
}
