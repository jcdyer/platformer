use std::time::{Duration, Instant};

use bevy::{
    prelude::{
        AssetServer, Assets, BuildChildren, Commands, Component, Entity, EventReader, Handle,
        Image, Input, KeyCode, Plugin, Query, Res, ResMut, Startup, Transform, Update, Vec2, Vec3,
        With, Without,
    },
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
};
use bevy_rapier2d::prelude::{
    ActiveEvents, Collider, CollisionEvent, Damping, LockedAxes, RigidBody, Velocity,
};

use crate::animation::Animation;

const SPRITESHEET: &str = "Spritesheets/spritesheet_players.png";
const SPRITESHEET_COLS: usize = 7;
const SPRITESHEET_ROWS: usize = 8;
const SPRITE_TILE_WIDTH: f32 = 128.;
const SPRITE_TILE_HEIGHT: f32 = 256.;

const SPRITE_IDX_GREEN_STAND: usize = 5;
const SPRITE_IDX_GREEN_WALK_0: usize = 11;
const SPRITE_IDX_GREEN_WALK_1: usize = 18;

const WALK_CYCLE_DELAY: Duration = Duration::from_millis(120);
const RUN_CYCLE_DELAY: Duration = Duration::from_millis(40);

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub last_running: Instant,
}

impl Player {
    fn is_running(&self) -> bool {
        self.last_running.elapsed() < Duration::from_millis(100)
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                player_jumps,
                jump_reset,
                check_reset_game,
                player_movement,
                apply_movement_animation,
                apply_idle_sprite,
                update_direction,
                update_sprite_direction,
            ),
        );
    }
}

#[derive(Debug, Component)]
pub struct Jumper {
    pub jump_impulse: f32,
    pub is_jumping: bool,
}

pub fn setup(
    mut commands: Commands,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    server: Res<AssetServer>,
) {
    let image_handle: Handle<Image> = server.load(SPRITESHEET);
    let texture_atlas = TextureAtlas::from_grid(
        image_handle,
        Vec2::new(SPRITE_TILE_WIDTH, SPRITE_TILE_HEIGHT),
        SPRITESHEET_COLS,
        SPRITESHEET_ROWS,
        None,
        None,
    );

    let atlas_handle = atlases.add(texture_atlas);
    let collider = Collider::cuboid(0.5, 1.0);
    let mut sprite = TextureAtlasSprite::new(SPRITE_IDX_GREEN_STAND);
    sprite.custom_size = Some(Vec2::new(1.0, 2.0));
    commands
        .spawn(SpriteSheetBundle {
            sprite,
            texture_atlas: atlas_handle,
            transform: Transform::from_translation(Vec3::new(0.0, 10.0, 1.0)),
            ..SpriteSheetBundle::default()
        })
        .insert(Direction::Right)
        .insert(Player {
            speed: 5.5,
            last_running: Instant::now() - Duration::from_secs_f32(1.0),
        })
        .insert(RigidBody::Dynamic)
        .insert(Damping {
            linear_damping: 0.5,
            angular_damping: 1.0,
        })
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(collider)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Velocity {
            linvel: Vec2::new(0., 0.),
            angvel: 0.0,
        })
        .insert(Jumper {
            jump_impulse: 50.0,
            is_jumping: false,
        })
        .with_children(|children| {
            children.spawn(crate::new_camera_2d());
        });
}
pub fn player_jumps(
    keyboard_input: Res<Input<KeyCode>>,
    mut players: Query<(&mut Jumper, &mut Velocity), With<Player>>,
) {
    for (mut jumper, mut velocity) in players.iter_mut() {
        if keyboard_input.pressed(KeyCode::Space) && !jumper.is_jumping {
            velocity.linvel = Vec2::new(velocity.linvel.x, jumper.jump_impulse);
            jumper.is_jumping = true;
        }
    }
}

fn jump_reset(
    mut query: Query<(Entity, &mut Jumper)>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.iter() {
        for (entity, mut jumper) in query.iter_mut() {
            set_jumping_false_if_touching_floor(&entity, &mut jumper, collision_event);
        }
    }
}

fn set_jumping_false_if_touching_floor(
    entity: &Entity,
    jumper: &mut Jumper,
    collision_event: &CollisionEvent,
) {
    if let CollisionEvent::Started(e1, e2, _) = collision_event {
        if entity == e1 || entity == e2 {
            jumper.is_jumping = false;
        }
    }
}

fn check_reset_game(mut query: Query<(&Player, &mut Velocity, &mut Transform)>) {
    for (_, mut velocity, mut transform) in query.iter_mut() {
        if transform.translation.y < -200.0 {
            transform.translation = Vec3::new(0.0, 10.0, 1.0);
            velocity.linvel.x = 0.0;
            velocity.linvel.y = 0.0;
        }
    }
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut players: Query<(&mut Player, &mut Velocity)>,
) {
    for (mut player, mut velocity) in players.iter_mut() {
        if keyboard_input.pressed(KeyCode::B) {
            player.last_running = Instant::now()
        }
        let running_coeff = if player.is_running() { 3.0 } else { 1.0 };
        if keyboard_input.pressed(KeyCode::Left) {
            velocity.linvel.x = -player.speed * running_coeff;
        } else if keyboard_input.pressed(KeyCode::Right) {
            velocity.linvel.x = player.speed * running_coeff;
        }
    }
}


fn apply_movement_animation(
    mut commands: Commands,
    query: Query<(Entity, &Player, &Velocity), Without<Animation>>,
) {
    if query.is_empty() {
        return;
    }

    let (entity, player, velocity) = query.single();
    if velocity.linvel.x != 0.0 && velocity.linvel.y == 0.0 {
        let delay = if player.is_running() {
            RUN_CYCLE_DELAY
        } else {
            WALK_CYCLE_DELAY
        };
        commands.entity(entity).insert(Animation::new(
            &[SPRITE_IDX_GREEN_WALK_0, SPRITE_IDX_GREEN_WALK_1],
            delay,
        ));
    }
}

fn apply_idle_sprite(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Velocity,
        &mut TextureAtlasSprite,
    )>,
) {
    if query.is_empty() {
        return;
    }
    let (player, velocity, mut sprite) = query.single_mut();
    if velocity.linvel.x == 0.0 {
        commands.entity(player).remove::<Animation>();
        sprite.index = SPRITE_IDX_GREEN_STAND;
    }
}

#[derive(Component)]
enum Direction {
    Left,
    Right,
}

fn update_direction(
    mut commands: Commands,
    query: Query<(Entity, &Velocity)>,
) {
    if query.is_empty() {
        return;
    }

    let (player, velocity) = query.single();

    if velocity.linvel.x > 0.0 {
        commands.entity(player).insert(Direction::Right);
    } else if velocity.linvel.x < 0.0 {
        commands.entity(player).insert(Direction::Left);
    }
}

fn update_sprite_direction(mut query: Query<(&mut TextureAtlasSprite, &Direction)>) {
    if query.is_empty() {
        return;
    }

    let (mut sprite, direction) = query.single_mut();

    match direction {
        Direction::Right => sprite.flip_x = false,
        Direction::Left => sprite.flip_x = true,
    }
}
