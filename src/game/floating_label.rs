use bevy::prelude::*;

use crate::{GameState, UIAssets};

#[derive(Event)]
pub struct SpawnFloatingLabelEvent(pub String, pub Vec2);

#[derive(Component)]
struct FloatingLabel;

pub struct FloatingLabelPlugin;

impl Plugin for FloatingLabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnFloatingLabelEvent>()
            .add_systems(
                Update,
                spawn_label.run_if(
                    in_state(GameState::Level).and_then(on_event::<SpawnFloatingLabelEvent>()),
                ),
            )
            .add_systems(OnExit(GameState::Level), despawn_labels);
    }
}

fn spawn_label(
    ui_assets: Res<UIAssets>,
    mut commands: Commands,
    mut events: EventReader<SpawnFloatingLabelEvent>,
) {
    for event in events.read() {
        commands.spawn((
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        event.0.clone(),
                        TextStyle {
                            font: ui_assets.font.clone(),
                            font_size: 30.,
                            color: Color::BLACK,
                        },
                    )],
                    alignment: TextAlignment::Center,
                    ..Default::default()
                },
                transform: Transform {
                    translation: event.1.extend(0.),
                    ..Default::default()
                },
                ..Default::default()
            },
            FloatingLabel,
        ));
    }
}

fn despawn_labels(mut commands: Commands, query: Query<Entity, With<FloatingLabel>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
