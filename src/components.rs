use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct Velocity(pub Vec2);

#[derive(Component, Reflect)]
pub struct Gravity(pub Vec2);
