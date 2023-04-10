use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{AudioAssets, BackgroundMusicChannel, GameState, UIAssets};

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
enum ButtonType {
    Play,
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_main_menu.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(despawn_main_menu.in_schedule(OnExit(GameState::MainMenu)))
            .add_systems(
                (button_appearance_system, button_action_system)
                    .in_set(OnUpdate(GameState::MainMenu)),
            );
    }
}

fn spawn_main_menu(
    mut commands: Commands,
    bgm: Res<AudioChannel<BackgroundMusicChannel>>,
    ui_assets: Res<UIAssets>,
    audio_assets: Res<AudioAssets>,
) {
    // Plays the background music on repeat
    //bgm.play(audio_assets.bg_music.clone()).looped();

    // Spawn in the main menu bundles
    commands
        .spawn((
            NodeBundle {
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
            },
            MainMenu,
        ))
        .with_children(|n| {
            n.spawn(TextBundle::from_section(
                "EXPIRY DATE",
                TextStyle {
                    font: ui_assets.font.clone(),
                    font_size: 60.,
                    color: Color::BLACK,
                },
            ));

            n.spawn((
                ButtonBundle {
                    image: UiImage::new(ui_assets.button.clone()),
                    style: Style {
                        padding: UiRect::new(
                            Val::Px(25.),
                            Val::Px(25.),
                            Val::Px(14.),
                            Val::Px(14.),
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ButtonType::Play,
            ))
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

fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenu>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

fn button_appearance_system(
    mut query: Query<(&mut UiImage, &Interaction), Changed<Interaction>>,
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

fn button_action_system(
    mut state: ResMut<NextState<GameState>>,
    query: Query<(&ButtonType, &Interaction), Changed<Interaction>>,
) {
    for (btn, interaction) in query.iter() {
        if *interaction != Interaction::Clicked {
            continue;
        }

        // TODO: Add fade effect when transitioning states
        match *btn {
            ButtonType::Play => state.set(GameState::Level),
        }
    }
}
