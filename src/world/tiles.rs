use bevy::prelude::*;

use std::collections::HashSet;

use crate::PIXELS_PER_METER;

use super::{collidables::spawn_collidables, World};

#[derive(Component)]
pub struct Tile {
    tile_type: TileType,
    order: usize,
    collidables: Vec<Entity>,
}

#[derive(Copy, Clone, Debug)]
pub enum TileType {
    Countryside,
    Village,
    CityMinor,
    CityMajor,
    CityMetropolis,
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
