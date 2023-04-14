use bevy::prelude::*;

use crate::GameState;

#[derive(Component, Reflect)]
pub struct Velocity(pub Vec2);

#[derive(Component, Reflect)]
pub struct Gravity(pub Vec2);

#[derive(Component, Reflect)]
pub struct RectCollisionShape {
    pub size: Vec2,
    pub collide: bool,
}

#[derive(Component)]
pub struct ScreenFade {
    pub fade_color: Color,
    pub fade_timer: Timer,
    pub next_state: GameState,
}
