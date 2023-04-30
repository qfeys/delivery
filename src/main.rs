mod camera;
mod stork;
mod world;

use std::env;

use bevy::prelude::*;
use camera::*;
use stork::{move_stork, spawn_stork};
use world::tiles::spawn_tiles;

const PIXELS_PER_METER: f32 = 100.0;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    App::new()
        .add_plugins(DefaultPlugins)
        // Resources
        .init_resource::<world::World>()
        // Startup Systems
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_stork)
        // Systems
        .add_system(move_camera)
        .add_system(move_stork)
        .add_system(spawn_tiles)
        .run();
}
