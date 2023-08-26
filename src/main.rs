use std::{fs::File, io::BufRead};

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::camera::ScalingMode,
    window::{PrimaryWindow, WindowResolution},
};
use bevy_rapier2d::prelude::{Collider, NoUserData, RapierPhysicsPlugin, RigidBody};

mod animation;
mod player;

fn main() {
    let plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(800.0, 480.0),
            title: "Platformer!".into(),
            ..Window::default()
        }),
        ..WindowPlugin::default()
    });
    App::new()
        .add_systems(Startup, (configure_window, spawn_floors))
        .add_plugins((
            plugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            player::PlayerPlugin,
            animation::AnimationPlugin,
        ))
        .run();
}

fn spawn_floors(commands: Commands, atlases: ResMut<Assets<TextureAtlas>>, server: Res<AssetServer>) {
    spawn_floors_inner(commands, atlases, server).unwrap()
}

fn spawn_floors_inner(
    mut commands: Commands,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    server: Res<AssetServer>,
)-> anyhow::Result<()> {
    let ground_spritesheet: Handle<Image> = server.load("Spritesheets/spritesheet_ground.png");

    let ground_atlas =
        TextureAtlas::from_grid(ground_spritesheet, Vec2::new(128., 128.), 7, 16, None, Some(Vec2::new(0.0, 1.0)));
    let ground_atlas = atlases.add(ground_atlas);
    let mut floors = vec![];
    let file = std::io::BufReader::new(File::open("floors.txt").unwrap());
    for line in file.lines() {
        let line = line.unwrap();
        let (x,y, len, left, right, middle) = match &line.split(',').collect::<Vec<_>>()[..] {
            [x, y, len, tiles @ ..] => {
                #[allow(clippy::get_first)]
                let left = tiles.get(0).map(|tile| tile.parse()).transpose()?;
                let right = tiles.get(1).map(|tile| tile.parse()).transpose()?;
                let middle = tiles.get(2).map(|tile| tile.parse()).transpose()?;
                (x.parse()?,y.parse()?,len.parse()?, left,right ,middle)
            }
            _ => panic!(),
        };
        floors.push((Vec2::new(x, y), len, left, right, middle));
    }

    for (start, length, left, right, middle) in floors {
        spawn_floor(&mut commands, ground_atlas.clone(), start, length, left, right, middle)
    }
    Ok(())
}

fn spawn_floor(
    commands: &mut Commands,
    ground_atlas: Handle<TextureAtlas>,
    start: Vec2,
    length: f32,
    left: Option<usize>,
    right:Option<usize>,
    middle: Option<usize>,
) {
    const GROUND_TEXTURE_INDEX_MIDDLE: usize = 82;
    const GROUND_TEXTURE_INDEX_LEFT: usize = 48;
    const GROUND_TEXTURE_INDEX_RIGHT: usize = 41;
    const GROUND_TEXTURE_INDEX_ALONE: usize = 38;


    let mut x_offset = 0.;
    let collider = Collider::cuboid(length * 0.5, 0.5);
    while x_offset < length {

    let mut ground_sprite = if length == 1.0 {
        TextureAtlasSprite::new(left.unwrap_or(GROUND_TEXTURE_INDEX_ALONE))
    } else if x_offset == 0.0 {

        TextureAtlasSprite::new(left.unwrap_or(GROUND_TEXTURE_INDEX_LEFT))
    } else if length - x_offset <= 1.0 {
        TextureAtlasSprite::new(right.unwrap_or(GROUND_TEXTURE_INDEX_RIGHT))
    } else {
        TextureAtlasSprite::new(middle.unwrap_or(GROUND_TEXTURE_INDEX_MIDDLE))
    };
    ground_sprite.custom_size = Some(Vec2::new(1.0, 1.0));

        let location = Vec2::new(start.x + x_offset + 0.5, start.y);
        let translation = location.extend(1.0);

        commands.spawn(SpriteSheetBundle {
            sprite: ground_sprite.clone(),
            texture_atlas: ground_atlas.clone(),
            transform: Transform::from_translation(translation),
            ..SpriteSheetBundle::default()
        });
        x_offset += 1.0;
    }
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(length, 1.0)),
                ..Sprite::default()
            },
            transform: Transform::from_translation(Vec3::new(start.x + length / 2., start.y, 1.0)),
            ..SpriteBundle::default()
        })
        .insert(RigidBody::Fixed)
        .insert(collider);
}

fn configure_window(mut query: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = query.get_single_mut() {
        window.title = "Platformer!".into();
    }
}

fn new_camera_2d() -> Camera2dBundle {
    let mut cam2d = Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.2, 0.3)),
        },
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal(10.0),
            ..OrthographicProjection::default()
        },
        ..Camera2dBundle::default()
    };
    cam2d.transform.scale = Vec3::new(4.0, 4.0, 1.0);
    println!("{:?}", cam2d.transform);
    cam2d
}
