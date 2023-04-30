use bevy::{prelude::*, window::PrimaryWindow};
use lerp::Lerp;

use super::{stork::Stork, PIXELS_PER_METER};

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
        x if x < 0.2 => {
            // Desired camera position to have the stork at a position between the current pos and 20%
            // Closer to 20%, we go for current pos, closer to 10 %, we go for 20% pos
            let current_camera_pos = camera.translation.x;
            let camera_pos_20 = stork_pixel_pos.x + 0.3 * (right_bound - left_bound);
            let ratio = (0.2 - x) / 0.1; // is 0 at 20% and 1 at 10%
            let desired_camera_pos =
                current_camera_pos.lerp(camera_pos_20, ratio * time.delta_seconds());
            camera.translation.x = desired_camera_pos;
        }
        x if x > 0.8 => {
            // Desired camera position to have the stork at a position between the current pos and 80%
            // Closer to 80%, we go for current pos, closer to 90 %, we go for 80% pos
            let current_camera_pos = camera.translation.x;
            let camera_pos_80 = stork_pixel_pos.x - 0.3 * (right_bound - left_bound);
            let ratio = (x - 0.8) / 0.1; // is 0 at 80% and 1 at 90%
            let desired_camera_pos =
                current_camera_pos.lerp(camera_pos_80, ratio * time.delta_seconds());
            camera.translation.x = desired_camera_pos;
        }
        _ => {}
    }
}
