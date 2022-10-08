// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{
    prelude::*,
    render::camera::{RenderTarget, ScalingMode},
};

mod boids;
mod enemies;
mod gold;
mod hex;
mod palette;
mod tower;
mod tutorial;

use crate::hex::HexPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(boids::BoidsPlugin)
        .add_plugin(enemies::EnemyPlugin)
        .add_plugin(gold::GoldPlugin)
        .add_plugin(HexPlugin)
        .add_plugin(tower::TowerPlugin)
        .add_plugin(tutorial::TutorialPlugin)
        .insert_resource(MouseWorldPos(Vec2::ONE * 10000.0))
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            title: "Gold Collex Hex Defense".to_string(),
            canvas: Some("#bevy".to_owned()),
            ..Default::default()
        })
        .add_event::<StartSpawningEnemiesEvent>()
        .add_startup_system(setup)
        .add_system(update_mouse_position)
        .run();
}

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;

struct MouseWorldPos(Vec2);

pub struct StartSpawningEnemiesEvent;

fn setup(mut commands: Commands) {
    //commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical(720.),
            ..default()
        },
        // default is 0.4 all
        // camera_2d: Camera2d {
        //     clear_color: ClearColorConfig::Custom(Color::rgb(0.278, 0.247, 0.202))
        // },
        ..default()
    });
    // commands.spawn_bundle(SpriteBundle {
    //     texture: asset_server.load("icon.png"),
    //     transform: Transform {
    //         translation: Vec3::new(500.0, 0.0, 0.0),
    //         ..default()
    //     },
    //     ..Default::default()
    // });
}

fn update_mouse_position(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut mouse_pos: ResMut<MouseWorldPos>,
) {
    let (camera, camera_transform) = q_camera.single();

    let win = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    if let Some(screen_pos) = win.cursor_position() {
        let window_size = Vec2::new(win.width() as f32, win.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coords)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        mouse_pos.0 = world_pos;
    }
}
