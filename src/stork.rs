use bevy::{prelude::*, window::PrimaryWindow};

use super::PIXELS_PER_METER;

#[derive(Component)]
pub struct Stork {
    pub position: Vec2,
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
        if stork.direction.abs() > 175.0 {
            stork.direction = -stork.direction.signum() * 180.0;
        }
    }
    if keyboard_input.pressed(KeyCode::Down) {
        stork.direction -= TURN_SPEED * stork.direction.signum() * time.delta_seconds();
        if stork.direction.abs() < 5.0 {
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
