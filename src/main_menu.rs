use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{AudioAssets, BackgroundMusicChannel, GameState, SpawnScreenFader, UIAssets};

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
enum ButtonType {
    Play,
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(GameState::MainMenu), despawn_main_menu)
            .add_systems(
                Update,
                button_action_system.run_if(in_state(GameState::MainMenu)),
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
    bgm.play(audio_assets.bg_music.clone()).looped();

    // Spawn in the main menu bundles
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    display: Display::Flex,
                    row_gap: Val::Px(8.),
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

fn button_action_system(
    mut events: EventWriter<SpawnScreenFader>,
    query: Query<(&ButtonType, &Interaction), Changed<Interaction>>,
) {
    for (btn, interaction) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *btn {
            ButtonType::Play => events.send(SpawnScreenFader {
                fade_color: Color::ANTIQUE_WHITE,
                fade_time: 0.8,
                next_state: GameState::Level,
            }),
        }
    }
}
