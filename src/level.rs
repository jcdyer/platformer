use std::fs::File;

use bevy::{
    prelude::{
        AssetServer, Assets, BuildChildren, Children, Color, Commands, Component,
        DespawnRecursiveExt, Entity, Handle, Image, Input, KeyCode, Plugin, Query, Res, ResMut,
        SpatialBundle, Startup, Transform, Update, Vec2, Vec3, Without,
    },
    sprite::{Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
};
use bevy_rapier2d::prelude::{Collider, RigidBody};

use crate::player::Player;

pub struct LevelPlugin;

#[derive(Debug, Component)]
pub struct Level {
    idx: u8,
}
#[derive(Debug, Component)]
pub struct Ready;

fn setup_level(mut commands: Commands) {
    commands
        .spawn(SpatialBundle::default())
        .insert(Level { idx: 0 });
    //.insert(Shader::from_glsl("shaders/bg.glsl", ShaderStage::Fragment, ));
}

#[derive(serde::Deserialize)]
struct WorldDefinition {
    levels: Vec<LevelDefinition>,
}

#[derive(serde::Deserialize)]
struct LevelDefinition {
    // id: String,
    // name: String,
    features: Vec<Feature>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "data")]
enum Feature {
    Floor(Vec<FloorDefinition>),
    Exit(Vec<ExitDefinition>),
    Elevator(Vec<ElevatorDefinition>),
}

#[derive(serde::Deserialize)]
struct ExitDefinition {
    location: Vec2,
}
#[derive(serde::Deserialize, Debug)]
pub struct ElevatorDefinition {
    pub start_location: Vec2,
    pub end_y: f32,
    //pub path: tg::Line,
    pub control: ElevatorControl,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ElevatorControl {
    Constant,
    Switches { locations: (Vec2, Vec2) },
}

#[derive(serde::Deserialize)]
struct FloorDefinition {
    loc: Vec2,
    length: f32,
    left: Option<usize>,
    right: Option<usize>,
    middle: Option<usize>,
}

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup_level)
            .add_systems(Update, (exit_level, spawn_level));
    }
}

#[derive(Component)]
struct Exit {
    destination: u8,
}

fn spawn_level(
    query: Query<(Entity, &Level), Without<Children>>,
    mut commands: Commands,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    server: Res<AssetServer>,
) {
    if query.is_empty() {
        return;
    }
    let (entity, level) = query.single();
    let mut file = std::io::BufReader::new(File::open("world.yml").unwrap());
    let world_definition: WorldDefinition = serde_yaml::from_reader(&mut file).unwrap();
    match world_definition.levels.get(level.idx as usize) {
        Some(level_definition) => {
            spawn_level_features(
                &mut commands,
                &mut atlases,
                entity,
                level,
                level_definition,
                &server,
            );
        }
        None => spawn_win_screen(),
    }
}

fn spawn_win_screen() {}

fn spawn_level_features(
    commands: &mut Commands,
    atlases: &mut Assets<TextureAtlas>,
    level_entity: Entity,
    level: &Level,
    level_definition: &LevelDefinition,
    server: &AssetServer,
) {
    let tile_spritesheet: Handle<Image> = server.load("Spritesheets/spritesheet_tiles.png");
    let tile_atlas = TextureAtlas::from_grid(
        tile_spritesheet,
        Vec2::new(128., 128.),
        5,
        16,
        None,
        Some(Vec2::new(0.0, 1.0)),
    );
    let tile_atlas = atlases.add(tile_atlas);

    let ground_spritesheet: Handle<Image> = server.load("Spritesheets/spritesheet_ground.png");

    let ground_atlas = TextureAtlas::from_grid(
        ground_spritesheet,
        Vec2::new(128., 128.),
        7,
        16,
        None,
        Some(Vec2::new(0.0, 1.0)),
    );
    let ground_atlas = atlases.add(ground_atlas);

    for feature in &level_definition.features {
        match feature {
            Feature::Exit(exits) => {
                spawn_exits(commands, &tile_atlas, level_entity, level.idx, exits)
            }
            Feature::Floor(floors) => {
                spawn_floors(commands, floors, level_entity, &ground_atlas).unwrap()
            }
            Feature::Elevator(elevators) => {
                spawn_elevators(commands, &ground_atlas, level_entity, elevators)
            }
        }
    }
}

fn spawn_elevators(
    commands: &mut Commands,
    ground_atlas: &Handle<TextureAtlas>,
    level_entity: Entity,
    elevators: &[ElevatorDefinition],
) {
    for elevator in elevators {
        super::elevator::setup(commands, ground_atlas, level_entity, elevator);
    }
}

fn spawn_exits(
    commands: &mut Commands,
    tile_atlas: &Handle<TextureAtlas>,
    level_entity: Entity,
    level_index: u8,
    exits: &[ExitDefinition],
) {
    const DOOR_BOTTOM_SPRITE_INDEX: usize = 48;
    const DOOR_TOP_SPRITE_INDEX: usize = 43;
    const EXIT_SIGN_SPRITE_INDEX: usize = 39;

    let mut door_bottom_sprite = TextureAtlasSprite::new(DOOR_BOTTOM_SPRITE_INDEX);
    door_bottom_sprite.custom_size = Some(Vec2::new(1.0, 1.0));

    let mut door_top_sprite = TextureAtlasSprite::new(DOOR_TOP_SPRITE_INDEX);
    door_top_sprite.custom_size = Some(Vec2::new(1.0, 1.0));

    let mut exit_sign_sprite = TextureAtlasSprite::new(EXIT_SIGN_SPRITE_INDEX);
    exit_sign_sprite.custom_size = Some(Vec2::new(1.0, 1.0));

    for exit in exits {
        let door_bottom_location = exit.location;
        let door_bottom_translation = door_bottom_location.extend(1.0);

        let door_top_translation = Vec3::new(
            door_bottom_translation.x,
            door_bottom_translation.y + 1.0,
            door_bottom_translation.z,
        );

        let exit_sign_translation = Vec3::new(
            door_bottom_translation.x + 2.0,
            door_bottom_translation.y,
            2.0,
        );

        commands.entity(level_entity).with_children(|children| {
            children
                .spawn(SpriteSheetBundle {
                    sprite: door_bottom_sprite.clone(),
                    texture_atlas: tile_atlas.clone(),
                    transform: Transform::from_translation(door_bottom_translation),
                    ..SpriteSheetBundle::default()
                })
                .insert(Exit {
                    destination: level_index + 1,
                });
            children.spawn(SpriteSheetBundle {
                sprite: door_top_sprite.clone(),
                texture_atlas: tile_atlas.clone(),
                transform: Transform::from_translation(door_top_translation),
                ..SpriteSheetBundle::default()
            });
            children.spawn(SpriteSheetBundle {
                sprite: exit_sign_sprite.clone(),
                texture_atlas: tile_atlas.clone(),
                transform: Transform::from_translation(exit_sign_translation),
                ..SpriteSheetBundle::default()
            });
        });
    }
}

fn exit_level(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    exit_query: Query<(&Exit, &Transform)>,
    mut player_query: Query<(&Player, &mut Transform), Without<Exit>>,
    mut level_query: Query<(Entity, &mut Level)>,
) {
    if keyboard_input.pressed(KeyCode::Up) {
        if player_query.is_empty() {
            return;
        }
        if level_query.is_empty() {
            return;
        }
        let (_, mut player_transform) = player_query.single_mut();
        for (exit, exit_transform) in exit_query.iter() {
            if (player_transform.translation.x - exit_transform.translation.x).abs() < 0.5
                && (player_transform.translation.y - exit_transform.translation.y).abs() < 0.5
            {
                println!("Go to level: {}", exit.destination);
                player_transform.translation = Vec3::new(0.0, 10.0, 1.0);
                let (level_entity, mut level) = level_query.single_mut();
                level.idx = exit.destination;
                commands.entity(level_entity).despawn_descendants();
                commands.entity(level_entity).clear_children();
                println!("level: {level:?}");
            }
        }
    }
}

fn spawn_floors(
    commands: &mut Commands,
    floors: &[FloorDefinition],
    level_entity: Entity,
    ground_atlas: &Handle<TextureAtlas>,
) -> anyhow::Result<()> {
    for floor in floors {
        spawn_floor_onto(level_entity, commands, ground_atlas.clone(), floor)
    }
    Ok(())
}

fn spawn_floor_onto(
    entity: Entity,
    commands: &mut Commands,
    ground_atlas: Handle<TextureAtlas>,
    floor: &FloorDefinition,
) {
    const GROUND_TEXTURE_INDEX_MIDDLE: usize = 82;
    const GROUND_TEXTURE_INDEX_LEFT: usize = 48;
    const GROUND_TEXTURE_INDEX_RIGHT: usize = 41;
    const GROUND_TEXTURE_INDEX_ALONE: usize = 38;

    let mut x_offset = 0.;
    let collider = Collider::cuboid(floor.length * 0.5, 0.5);

    let mut entity_builder = commands.entity(entity);
    while x_offset < floor.length {
        let mut ground_sprite = if floor.length == 1.0 {
            TextureAtlasSprite::new(floor.left.unwrap_or(GROUND_TEXTURE_INDEX_ALONE))
        } else if x_offset == 0.0 {
            TextureAtlasSprite::new(floor.left.unwrap_or(GROUND_TEXTURE_INDEX_LEFT))
        } else if floor.length - x_offset <= 1.0 {
            TextureAtlasSprite::new(floor.right.unwrap_or(GROUND_TEXTURE_INDEX_RIGHT))
        } else {
            TextureAtlasSprite::new(floor.middle.unwrap_or(GROUND_TEXTURE_INDEX_MIDDLE))
        };
        ground_sprite.custom_size = Some(Vec2::new(1.0, 1.0));

        let location = Vec2::new(floor.loc.x + x_offset + 0.5, floor.loc.y);
        let translation = location.extend(1.0);

        entity_builder.with_children(|children| {
            children.spawn(SpriteSheetBundle {
                sprite: ground_sprite.clone(),
                texture_atlas: ground_atlas.clone(),
                transform: Transform::from_translation(translation),
                ..SpriteSheetBundle::default()
            });
        });
        x_offset += 1.0;
    }
    entity_builder.with_children(|children| {
        children
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(1.0, 0.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(floor.length, 1.0)),
                    ..Sprite::default()
                },
                transform: Transform::from_translation(Vec3::new(
                    floor.loc.x + floor.length / 2.,
                    floor.loc.y,
                    1.0,
                )),
                ..SpriteBundle::default()
            })
            .insert(RigidBody::Fixed)
            .insert(collider);
    });
}

#[cfg(test)]
mod tests {
    use super::WorldDefinition;
    use std::{fs::File, io::BufReader};

    #[test]
    fn deserialize_world() {
        let mut file = BufReader::new(File::open("world.yml").unwrap());
        let _world_definition: WorldDefinition = serde_yaml::from_reader(&mut file).unwrap();
    }
}
