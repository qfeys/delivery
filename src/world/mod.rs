pub mod collidables;
pub mod tiles;

use bevy::prelude::*;

use rand::prelude::*;
use std::collections::HashMap;

use self::tiles::TileType;

#[derive(Resource)]
pub struct World {
    seed: u64,
    tiles: HashMap<usize, TileType>,
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
