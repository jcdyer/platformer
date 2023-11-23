
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::camera::ScalingMode,
    window::{PrimaryWindow, WindowResolution},
};
use bevy_rapier2d::prelude::{ NoUserData, RapierPhysicsPlugin, };

mod animation;
mod level;
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
        .add_systems(Startup, (configure_window,))
        .add_plugins((
            plugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            player::PlayerPlugin,
            animation::AnimationPlugin,
            level::LevelPlugin
        ))
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
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
