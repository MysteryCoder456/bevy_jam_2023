use bevy::prelude::*;

use crate::{GameState, UIAssets};

#[derive(Component)]
struct GameOverMenu;

#[derive(Component)]
struct TryAgainButton;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_game_over_menu.in_schedule(OnEnter(GameState::GameOver)));
    }
}

fn spawn_game_over_menu(mut commands: Commands, ui_assets: Res<UIAssets>) {
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
                background_color: BackgroundColor(Color::BLACK),
                ..Default::default()
            },
            GameOverMenu,
        ))
        .with_children(|n| {
            n.spawn(TextBundle::from_section(
                "Game Over :(",
                TextStyle {
                    font: ui_assets.font.clone(),
                    font_size: 50.,
                    color: Color::WHITE,
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
                TryAgainButton,
            ))
            .with_children(|b| {
                b.spawn(TextBundle::from_section(
                    "Try Again?",
                    TextStyle {
                        font: ui_assets.font.clone(),
                        font_size: 38.,
                        color: Color::BLACK,
                    },
                ));
            });
        });
}
