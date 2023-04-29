use bevy::{prelude::*, window::PrimaryWindow};

use lerp::*;

const PIXELS_PER_METER: f32 = 100.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Startup Systems
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_stork)
        // Systems
        .add_system(move_camera)
        .add_system(move_stork)
        .run();
}

pub fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, window.height() / 2.0, 0.0),
        ..default()
    });
}

// When the Stork enters the first or last 20% of the screen, start moving the camera
// When the stork enters the last 10% of the screen, move the camera at the same speed as the stork
pub fn move_camera(
    time: Res<Time>,
    mut query_camera: Query<&mut Transform, With<Camera2d>>,
    query_stork: Query<&Stork>,
    query_window: Query<&Window, With<PrimaryWindow>>,
) {
    let mut camera = query_camera.get_single_mut().unwrap();
    let stork = query_stork.get_single().unwrap();
    let window = query_window.get_single().unwrap();
    let stork_pixel_pos = stork.position * PIXELS_PER_METER; // The position of the stork in pixel coordinates
    let right_bound = camera.translation.x + 0.5 * window.width() * camera.scale.x; // The right bound of the camera in pixel coordinates
    let left_bound = camera.translation.x - 0.5 * window.width() * camera.scale.x; // The left bound of the camera in pixel coordinates
                                                                                   // The position of the stork as a fraction of the place between the bounds, with respect to the camera
    let relative_stork_pos = (stork_pixel_pos.x - left_bound) / (right_bound - left_bound);
    match relative_stork_pos {
        /* x if x < 0.1 => {
            // Desired camera position to have the stork at 11%
            let desired_camera_pos = stork_pixel_pos.x + 0.39 * (right_bound - left_bound);
            camera.translation.x = desired_camera_pos;
        }
        x if x > 0.9 => {
            // Desired camera position to have the stork at 89%
            let desired_camera_pos = stork_pixel_pos.x - 0.39 * (right_bound - left_bound);
            println!("Desired camera pos: {}", desired_camera_pos);
            camera.translation.x = desired_camera_pos;
        } */
        x if x < 0.2 => {
            // Desired camera position to have the stork at a position between the current pos and 20%
            // Closer to 20%, we go for current pos, closer to 10 %, we go for 20% pos
            let current_camera_pos = camera.translation.x;
            let camera_pos_20 = stork_pixel_pos.x + 0.3 * (right_bound - left_bound);
            let ratio = (0.2 - x) / 0.1; // is 0 at 20% and 1 at 10%
            let desired_camera_pos = current_camera_pos.lerp(camera_pos_20, ratio * time.delta_seconds());
            camera.translation.x = desired_camera_pos;
        }
        x if x > 0.8 => {
            // Desired camera position to have the stork at a position between the current pos and 80%
            // Closer to 80%, we go for current pos, closer to 90 %, we go for 80% pos
            let current_camera_pos = camera.translation.x;
            let camera_pos_80 = stork_pixel_pos.x - 0.3 * (right_bound - left_bound);
            let ratio = (x - 0.8) / 0.1; // is 0 at 80% and 1 at 90%
            let desired_camera_pos = current_camera_pos.lerp(camera_pos_80, ratio*time.delta_seconds());
            camera.translation.x = desired_camera_pos;
        }
        _ => {}
    }
}

#[derive(Component)]
pub struct Stork {
    position: Vec2,
    speed: f32,
    direction: f32, // in degrees, 90 is to the right, 0 is down, -90 is to the left
}

// spawn the sork at the origin, and spawn the sprite.
pub fn spawn_stork(
    mut commands: Commands,
    window_querry: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_querry.get_single().unwrap();
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
            texture: asset_server.load("sprites\\stork.png"),
            ..default()
        },
        Stork {
            position: Vec2::new(0.0, 3.0),
            speed: 2.0,
            direction: 100.0,
        },
    ));
}

const AIR_RESISTANCE: f32 = 0.1;
const GRAVITY: f32 = 9.81 * 0.2;
const STALL_SPEED: f32 = 1.0;

const MAX_ACCELLEATION: f32 = 10.0;
const TOP_ACCELERATION_SPEED: f32 = 4.0;
const TURN_SPEED: f32 = 100.0;

pub fn move_stork(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Stork, &mut Transform)>,
) {
    let (mut stork, mut transf) = query.get_single_mut().unwrap();
    let mut speed = stork.speed;

    // apply power on keyboard input (shift)
    // At speed 0, the acceleration is maximum. At speed TOP_ACCELERATION_SPEED, the acceleration is 0
    // At speeds beyond TOP_ACCELERATION_SPEED, the acceleration is negative
    if keyboard_input.pressed(KeyCode::LShift) {
        let acceleration = MAX_ACCELLEATION * (1.0 - speed / TOP_ACCELERATION_SPEED);
        speed += acceleration * time.delta_seconds();
    }

    // on using the up and down arrows, change the direction
    if keyboard_input.pressed(KeyCode::Up) {
        stork.direction += TURN_SPEED * stork.direction.signum() * time.delta_seconds();
        if stork.direction.abs() > 180.0 {
            stork.direction = -stork.direction.signum() * 180.0;
        }
    }
    if keyboard_input.pressed(KeyCode::Down) {
        stork.direction -= TURN_SPEED * stork.direction.signum() * time.delta_seconds();
        if stork.direction.abs() > 0.0 {
            stork.direction = -stork.direction.signum() * 0.0;
        }
    }

    // apply gravity. Change the speed based on conservations of energy
    let delta_h = (stork.direction.abs() - 90.0).to_radians().sin() * speed * time.delta_seconds();
    speed = (speed.powi(2) - 2.0 * (delta_h * GRAVITY)).sqrt();
    assert!(!speed.is_nan(), "Error: Speed is NaN!");
    // apply air resistance
    speed -= speed * AIR_RESISTANCE * time.delta_seconds();
    // turn downward if speed is below stall speed
    if speed < STALL_SPEED {
        let dir = stork.direction.signum();
        let mut elevation = stork.direction.abs();
        // use a curve to make the turn more gradual
        // At the stall speed, the turn speed is 0, at speed 0, the turn speed is infinite
        let turnspeed = 20.0 / (speed / STALL_SPEED).powi(2);
        println!("Speed ratio: {}, turnspeed: {}", speed / STALL_SPEED, turnspeed);
        elevation -= turnspeed * time.delta_seconds();
        stork.direction = elevation * dir as f32;
    }
    let dir = stork.direction - 90.0; // coorrection so 0 is to the right, so the math is as in the unit circle
    stork.position +=
        Vec2::new(dir.to_radians().cos(), dir.to_radians().sin()) * speed * time.delta_seconds();
    transf.translation = Vec3::new(stork.position.x, stork.position.y, 0.0) * PIXELS_PER_METER;
    transf.rotation = Quat::from_rotation_z(dir.to_radians());
    stork.speed = speed;
}
