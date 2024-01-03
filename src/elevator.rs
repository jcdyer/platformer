#![allow(clippy::if_same_then_else)]

use bevy::{
    app::{Plugin, Update},
    asset::Handle,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query},
    },
    hierarchy::BuildChildren,
    math::{Vec2, Vec3},
    render::color::Color,
    sprite::{Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    transform::components::Transform,
};
use bevy_rapier2d::{
    control::KinematicCharacterController,
    dynamics::{RigidBody, Velocity},
    geometry::Collider,
};

use crate::level::ElevatorDefinition;

pub struct ElevatorPlugin;

impl Plugin for ElevatorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (update,));
    }
}

#[derive(Component, Debug)]
pub enum State {
    MovingForward,
    Stopped,
    MovingBackward,
}

#[derive(Component, Debug)]
pub struct Elevator {
    pub start: Vec2,
    pub end_y: f32,
}

pub fn setup(
    commands: &mut Commands,
    ground_atlas: &Handle<TextureAtlas>,
    // server: Res<AssetServer>,
    level: Entity,
    elevator: &ElevatorDefinition,
) {
    const ELEVATOR_LEFT_SPRITE_INDEX: usize = 13; // 1 * 7 + 6;
    const ELEVATOR_RIGHT_SPRITE_INDEX: usize = 110; // 15 * 7 + 5;
    const ELEVATOR_WIDTH: f32 = 2.0;

    let mut left_sprite = TextureAtlasSprite::new(ELEVATOR_LEFT_SPRITE_INDEX);
    left_sprite.custom_size = Some(Vec2::new(1.0, 1.0));
    let mut right_sprite = TextureAtlasSprite::new(ELEVATOR_RIGHT_SPRITE_INDEX);
    right_sprite.custom_size = Some(Vec2::new(1.0, 1.0));

    let location = elevator.start_location.extend(1.0);

    // collider is half the width of the elevator. May want to tweak the height.
    let collider = Collider::cuboid(ELEVATOR_WIDTH / 2.0, 0.5);

    let left = Vec3 {
        x: location.x + 0.5,
        ..location
    };
    let right = Vec3 {
        x: left.x + 1.0,
        ..left
    };

    commands.entity(level).with_children(|children| {
        children
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(1.0, 0.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(ELEVATOR_WIDTH, 1.0)),
                    ..Sprite::default()
                },
                transform: Transform::from_translation(Vec3::new(
                    location.x + ELEVATOR_WIDTH / 2.0, // Add half the elevator
                    location.y,
                    1.0,
                )),
                ..SpriteBundle::default()
            })
            .insert(RigidBody::KinematicVelocityBased)
            .insert(crate::elevator::Elevator {
                start: elevator.start_location,
                end_y: elevator.end_y,
            })
            .insert(State::MovingForward)
            .insert(Velocity::linear(Vec2::default()))
            .insert(collider)
            .insert(KinematicCharacterController::default())
            .with_children(|children| {
                children.spawn(SpriteSheetBundle {
                    sprite: left_sprite.clone(),
                    texture_atlas: ground_atlas.clone(),
                    transform: Transform::from_translation(Vec3::new(-0.5, 0.0, 0.0)),
                    ..SpriteSheetBundle::default()
                });
                children.spawn(SpriteSheetBundle {
                    sprite: right_sprite.clone(),
                    texture_atlas: ground_atlas.clone(),
                    transform: Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
                    ..SpriteSheetBundle::default()
                });
            });
    });
}

fn update(mut query: Query<(&Elevator, &mut State, &Transform, &mut Velocity)>) {
    for (elevator, mut state, transform, mut velocity) in query.iter_mut() {
        let is_forward_positive = elevator.end_y >= elevator.start.y;
        let new_state = match &*state {
            State::MovingForward => {
                // Past the end location
                if is_forward_positive && elevator.end_y <= transform.translation.y 
                    || elevator.end_y >= transform.translation.y
                {
                    State::MovingBackward
                } else {
                    State::MovingForward
                }
            }
            State::MovingBackward => {
                if is_forward_positive && elevator.start.y >= transform.translation.y
                    || elevator.end_y <= transform.translation.y {
                    State::MovingForward
                } else {
                    State::MovingBackward
                }
            }
            State::Stopped => State::Stopped,
        };
        *state = new_state;


        let multiplier = match *state {
            State::MovingForward => 1.0,
            State::MovingBackward => -1.0,
            State::Stopped => 0.0,
        };
        let base_elevator_velocity = Vec2 { x: 0.0, y: 5.5 };
        velocity.linvel =
            base_elevator_velocity * multiplier * if is_forward_positive { 1.0 } else { -1.0 };
    }
}
