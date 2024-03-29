use bevy::prelude::*;

use crate::{GameState, SpawnScreenFader, UIAssets};

#[derive(Component)]
struct GameOverMenu;

#[derive(Component)]
struct TryAgainButton;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), spawn_game_over_menu)
            .add_systems(OnExit(GameState::GameOver), despawn_game_over_menu)
            .add_systems(
                Update,
                try_again_system.run_if(in_state(GameState::GameOver)),
            );
    }
}

fn spawn_game_over_menu(mut commands: Commands, ui_assets: Res<UIAssets>) {
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

fn despawn_game_over_menu(mut commands: Commands, query: Query<Entity, With<GameOverMenu>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

fn try_again_system(
    mut events: EventWriter<SpawnScreenFader>,
    query: Query<&Interaction, (With<TryAgainButton>, Changed<Interaction>)>,
) {
    if let Ok(interaction) = query.get_single() {
        match *interaction {
            Interaction::Pressed => events.send(SpawnScreenFader {
                fade_color: Color::BLACK,
                fade_time: 0.8,
                next_state: GameState::Level,
            }),
            _ => {}
        }
    }
}
