use bevy::{app::PluginGroupBuilder, prelude::*};

use player::PlayerPlugin;

mod player;

pub struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PlayerPlugin)
            .build()
    }
}
