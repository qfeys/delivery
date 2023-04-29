use bevy::{prelude::*, window::PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Startup Systems
        .add_startup_system(spawn_camera)
        .add_system(spawn_stork)
        //.add_system(move_stork.system())
        .run();
}


pub fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}


#[derive(Component)]
struct Stork {
    position: Vec2,
    speed: f32,
    direction: f32, // in degrees, 90 is to the right, 0 is down, -90 is to the left
}

pub fn spawn_stork(
    mut commands: Commands,
    window_querry: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
){
    let window = window_querry.get_single().unwrap();
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
            texture: asset_server.load("sprites\\stork.png"),
            ..default()
        },
        Stork {
            position: Vec2::new(0.0, 0.0),
            speed: 100.0,
            direction: 90.0,
        }
    ));
}