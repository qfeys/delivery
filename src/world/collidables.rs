use bevy::{app::AppExit, prelude::*, sprite::collide_aabb::collide};

use crate::{stork::Stork, PIXELS_PER_METER};

use super::{tiles::TILE_SIZE, TileType};

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
                    pos + Vec2::new(0.0, HOUSE_2_SIZE.y / 2.0)
                        + Vec2::new(tile_nr as f32 * TILE_SIZE * PIXELS_PER_METER, 0.0),
                    asset_server,
                ));
            }
            for pos in house_1_locations {
                collidables.push(spawn_collidable(
                    commands,
                    CollidableType::House1,
                    pos + Vec2::new(0.0, HOUSE_1_SIZE.y / 2.0)
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
