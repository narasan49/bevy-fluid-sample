use bevy::prelude::*;

#[derive(Component)]
pub struct Circle {
    pub radius: f32,
}

#[derive(Component)]
pub struct Velocity {
    pub u: f32,
    pub v: f32,
}
