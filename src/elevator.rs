use bevy::{ecs::component::Component, math::Vec2};


#[derive(Component)]
pub struct Elevator {
    pub location: Vec2,
    pub active: bool,
    pub segment: usize,
}