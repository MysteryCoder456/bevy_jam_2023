use bevy::prelude::*;

use crate::{GameState, UIAssets};

pub struct SpawnFloatingLabelEvent(pub String, pub Vec2);

pub struct FloatingLabelPlugin;

impl Plugin for FloatingLabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnFloatingLabelEvent>().add_system(
            spawn_label
                .in_set(OnUpdate(GameState::Level))
                .run_if(on_event::<SpawnFloatingLabelEvent>()),
        );
    }
}

fn spawn_label(
    ui_assets: Res<UIAssets>,
    mut commands: Commands,
    mut events: EventReader<SpawnFloatingLabelEvent>,
) {
    for event in events.iter() {
        commands.spawn(Text2dBundle {
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
        });
    }
}
