use std::{
    collections::{HashMap, HashSet},
    env,
};

use bevy::{
    app::AppExit, prelude::*, render::primitives::Aabb, sprite::collide_aabb::collide,
    window::PrimaryWindow,
};
use rand::prelude::*;

use lerp::*;

const PIXELS_PER_METER: f32 = 100.0;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    App::new()
        .add_plugins(DefaultPlugins)
        // Resources
        .init_resource::<World>()
        // Startup Systems
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_stork)
        // Systems
        .add_system(move_camera)
        .add_system(move_stork)
        .add_system(spawn_tiles)
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

#[derive(Resource)]
pub struct World {
    seed: u64,
    tiles: HashMap<usize, TileType>,
}

#[derive(Copy, Clone, Debug)]
pub enum TileType {
    Countryside,
    Village,
    CityMinor,
    CityMajor,
    CityMetropolis,
}

impl Default for World {
    fn default() -> Self {
        Self {
            seed: random::<u64>(),
            tiles: HashMap::new(),
        }
    }
}

impl World {
    pub fn get_tile(&mut self, tile_nr: usize) -> TileType {
        if let Some(tile) = self.tiles.get(&tile_nr) {
            return *tile;
        }
        let micro_fluctuations = 1.5 * (tile_nr as f32).sin().powi(2);
        let minor_fluctiations = 1.0 * (tile_nr as f32 * 0.42).sin().powi(4);
        let major_fluctiations = 2.5 * (tile_nr as f32 * 0.13).sin().powi(8);
        let sum = micro_fluctuations + minor_fluctiations + major_fluctiations;
        self.tiles.insert(
            tile_nr,
            match sum {
                x if x >= 0.0 && x <= 1.0 => TileType::Countryside,
                x if x > 1.0 && x <= 2.0 => TileType::Village,
                x if x > 2.0 && x <= 3.0 => TileType::CityMinor,
                x if x > 3.0 && x <= 4.0 => TileType::CityMajor,
                x if x > 4.0 && x <= 5.0 => TileType::CityMetropolis,
                _ => panic!("Error: Tile type out of bounds!"),
            },
        );
        self.tiles[&tile_nr]
    }
}

#[derive(Component)]
pub struct Tile {
    tile_type: TileType,
    order: usize,
    collidables: Vec<Entity>,
}

pub const TILE_SIZE: f32 = 10.0; // in meters
pub const MAX_WINDOW_WIDTH: f32 = 2000.0; // in pixels

// Spawns tiles when the camera moves into a region where there are no tyles yet
pub fn spawn_tiles(
    mut commands: Commands,
    mut world: ResMut<World>,
    mut tile_query: Query<(&Tile, Entity)>,
    camera: Query<&Transform, With<Camera>>,
    asset_server: Res<AssetServer>,
) {
    // get the camera
    let camera = camera.get_single().unwrap();
    // Find all tile positions that could be visible
    let right_bound = camera.translation.x + MAX_WINDOW_WIDTH; // The right bound of the camera in pixel coordinates
    let left_bound = camera.translation.x - MAX_WINDOW_WIDTH; // The left bound of the camera in pixel coordinates
    let right_tile = (right_bound / PIXELS_PER_METER / TILE_SIZE).ceil() as usize; // The right bound of the camera in tile coordinates
    let left_tile = (left_bound / PIXELS_PER_METER / TILE_SIZE).floor() as usize; // The left bound of the camera in tile coordinates
    // Check all tiles. Despawn those that are too far away. Spawn the missing tiles.
    let mut present_tiles = HashSet::new();
    for (tile, entity) in tile_query.iter_mut() {
        let tile_nr = tile.order;
        if tile.order < left_tile {
            // Don't despawn tiles to the right
            commands.entity(entity).despawn();
            for collidable in tile.collidables.iter() {
                commands.entity(*collidable).despawn();
            }
        } else {
            present_tiles.insert(tile_nr);
        }
    }
    for tile_nr in left_tile..=right_tile {
        if !present_tiles.contains(&tile_nr) {
            let tyle_type = world.get_tile(tile_nr);
            let collidables = spawn_collidables(&mut commands, tyle_type, tile_nr, &asset_server);
            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_translation(Vec3::new(
                        tile_nr as f32 * TILE_SIZE * PIXELS_PER_METER,
                        0.5 * TILE_SIZE * PIXELS_PER_METER,
                        -1.0,
                    )),
                    texture: match tyle_type {
                        TileType::Countryside => asset_server.load("sprites/tiles/countryside.png"),
                        TileType::Village => asset_server.load("sprites/tiles/village.png"),
                        TileType::CityMinor => asset_server.load("sprites/tiles/city_minor.png"),
                        TileType::CityMajor => asset_server.load("sprites/tiles/city_major.png"),
                        TileType::CityMetropolis => {
                            asset_server.load("sprites/tiles/city_metropolis.png")
                        }
                    },
                    ..Default::default()
                },
                Tile {
                    tile_type: tyle_type,
                    order: tile_nr,
                    collidables,
                },
            ));
        }
    }
}

#[derive(Component)]
pub struct Collidable {
    collidable_type: CollidableType,
}

pub enum CollidableType {
    House1,
    House2,
    House3,
}

const SIZE_STORK: Vec2 = Vec2 { x: 64.0, y: 32.0 };
const HOUSE_1_SIZE: Vec2 = Vec2 { x: 156.0, y: 62.0 };
const HOUSE_2_SIZE: Vec2 = Vec2 { x: 120.0, y: 144.0 };
const HOUSE_3_SIZE: Vec2 = Vec2 { x: 152.0, y: 310.0 };

pub fn stork_hit_collidable(
    mut stork_query: Query<&Transform, With<Stork>>,
    collidable_query: Query<(&Collidable, &Transform)>,
    mut app_exit_event_writer: EventWriter<AppExit>,
) {
    let stork_transf = stork_query.get_single_mut().unwrap();

    for (collidable, collidable_transf) in collidable_query.iter() {
        if collide(
            stork_transf.translation,
            SIZE_STORK,
            collidable_transf.translation,
            match collidable.collidable_type {
                CollidableType::House1 => HOUSE_1_SIZE,
                CollidableType::House2 => HOUSE_2_SIZE,
                CollidableType::House3 => HOUSE_3_SIZE,
            },
        )
        .is_some()
        {
            println!("Collided!");
            println!("Game Over!");
            app_exit_event_writer.send(AppExit);
        }
    }
}

pub fn spawn_collidables(
    commands: &mut Commands,
    tile_type: TileType,
    tile_nr: usize,
    asset_server: &AssetServer,
) -> Vec<Entity> {
    let mut collidables = Vec::new();
    match tile_type {
        TileType::Countryside => {}
        TileType::Village => {
            // one house of type house_1 will spawn on one of three possible positions
            let house_1_pos = match tile_nr % 3 {
                0 => Vec2::new(-300.0, HOUSE_1_SIZE.y / 2.0),
                1 => Vec2::new(-150.0, HOUSE_1_SIZE.y / 2.0),
                2 => Vec2::new(200.0, HOUSE_1_SIZE.y / 2.0),
                _ => panic!("Error: Tile number out of bounds!"),
            };
            collidables.push(spawn_collidable(
                commands,
                CollidableType::House1,
                house_1_pos + Vec2::new(tile_nr as f32 * TILE_SIZE * PIXELS_PER_METER, 0.0),
                asset_server,
            ));
        }
        TileType::CityMinor => {
            // one house of type house_1 and one of type house_2 will spawn on one of four possible positions
            let combination = tile_nr % 12;
            let (option1, option2) = match combination {
                0 => (0, 1),
                1 => (0, 2),
                2 => (0, 3),
                3 => (1, 0),
                4 => (1, 2),
                5 => (1, 3),
                6 => (2, 0),
                7 => (2, 1),
                8 => (2, 3),
                9 => (3, 0),
                10 => (3, 1),
                11 => (3, 2),
                _ => panic!("Error: Tile number out of bounds!"),
            };
            let house_1_pos = match option1 {
                0 => Vec2::new(-300.0, HOUSE_1_SIZE.y / 2.0),
                1 => Vec2::new(-150.0, HOUSE_1_SIZE.y / 2.0),
                2 => Vec2::new(200.0, HOUSE_1_SIZE.y / 2.0),
                3 => Vec2::new(350.0, HOUSE_1_SIZE.y / 2.0),
                _ => panic!("Error: Tile number out of bounds!"),
            };
            let house_2_pos = match option2 {
                0 => Vec2::new(-300.0, HOUSE_2_SIZE.y / 2.0),
                1 => Vec2::new(-150.0, HOUSE_2_SIZE.y / 2.0),
                2 => Vec2::new(200.0, HOUSE_2_SIZE.y / 2.0),
                3 => Vec2::new(350.0, HOUSE_2_SIZE.y / 2.0),
                _ => panic!("Error: Tile number out of bounds!"),
            };
            collidables.push(spawn_collidable(
                commands,
                CollidableType::House1,
                house_1_pos + Vec2::new(tile_nr as f32 * TILE_SIZE * PIXELS_PER_METER, 0.0),
                asset_server,
            ));
            collidables.push(spawn_collidable(
                commands,
                CollidableType::House2,
                house_2_pos + Vec2::new(tile_nr as f32 * TILE_SIZE * PIXELS_PER_METER, 0.0),
                asset_server,
            ));
        }
        TileType::CityMajor => {
            // Three houses, at least 1 of type house_2, the others of type house_2 or house_1
            // The locations are chosen from 5 options
            // we will use a greedy algoritm to choose the locations, with as seed the tile nr
            let mut options = vec![
                Vec2::new(-400.0, 0.0),
                Vec2::new(-150.0, 0.0),
                Vec2::new(000.0, 0.0),
                Vec2::new(200.0, 0.0),
                Vec2::new(350.0, 0.0),
            ];
            let amount_of_house_2 = (tile_nr % 3) + 1;
            let mut house_2_locations = Vec::new();
            for _ in 0..amount_of_house_2 {
                let index = (tile_nr % options.len()) as usize;
                house_2_locations.push(options.remove(index));
            }
            let mut house_1_locations = Vec::new();
            for _ in 0..(3 - amount_of_house_2) {
                let index = (tile_nr % options.len()) as usize;
                house_1_locations.push(options.remove(index));
            }
            for pos in house_2_locations {
                collidables.push(spawn_collidable(
                    commands,
                    CollidableType::House2,
                    pos 
                        + Vec2::new(0.0, HOUSE_2_SIZE.y / 2.0)
                        + Vec2::new(tile_nr as f32 * TILE_SIZE * PIXELS_PER_METER, 0.0),
                    asset_server,
                ));
            }
            for pos in house_1_locations {
                collidables.push(spawn_collidable(
                    commands,
                    CollidableType::House1,
                    pos 
                        + Vec2::new(0.0, HOUSE_1_SIZE.y / 2.0)
                        + Vec2::new(tile_nr as f32 * TILE_SIZE * PIXELS_PER_METER, 0.0),
                    asset_server,
                ));
            }
        }
        TileType::CityMetropolis => {
            // Four houses, one of type house_3, the others of type house_2
            // The locations are chosen from 5 options
            // we will use a greedy algoritm to choose the locations, with as seed the tile nr
            let mut options = vec![
                Vec2::new(-350.0, 0.0),
                Vec2::new(-150.0, 0.0),
                Vec2::new(000.0, 0.0),
                Vec2::new(200.0, 0.0),
                Vec2::new(400.0, 0.0),
            ];
            let house_3_location = options.remove((tile_nr % options.len()) as usize);
            let mut house_2_locations = Vec::new();
            for _ in 0..3 {
                let index = (tile_nr % options.len()) as usize;
                house_2_locations.push(options.remove(index));
            }
            collidables.push(spawn_collidable(
                commands,
                CollidableType::House3,
                house_3_location
                    + Vec2::new(0.0, HOUSE_3_SIZE.y / 2.0)
                    + Vec2::new(tile_nr as f32 * TILE_SIZE * PIXELS_PER_METER, 0.0),
                asset_server,
            ));
            for pos in house_2_locations {
                collidables.push(spawn_collidable(
                    commands,
                    CollidableType::House2,
                    pos + Vec2::new(0.0, HOUSE_2_SIZE.y / 2.0)
                        + Vec2::new(tile_nr as f32 * TILE_SIZE * PIXELS_PER_METER, 0.0),
                    asset_server,
                ));
            }
        }
    }
    collidables
}

pub fn spawn_collidable(
    commands: &mut Commands,
    collidable_type: CollidableType,
    pos: Vec2,
    asset_server: &AssetServer,
) -> Entity {
    commands
        .spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(pos.x, pos.y, -0.5)),
                texture: match collidable_type {
                    CollidableType::House1 => asset_server.load("sprites/collidables/house_1.png"),
                    CollidableType::House2 => asset_server.load("sprites/collidables/house_2.png"),
                    CollidableType::House3 => asset_server.load("sprites/collidables/house_3.png"),
                },
                ..Default::default()
            },
            Collidable { collidable_type },
        ))
        .id()
}
