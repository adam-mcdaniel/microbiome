use rand::Rng;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color};
use ggez::event::{self, EventHandler};
use microbiome::*;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

const CAMERA_WIDTH: f64 = 1.0 / 8.0;
const CAMERA_HEIGHT: f64 = 1.0 / 8.0;

fn main() {
    let mut window_setup = ggez::conf::WindowSetup::default();
    window_setup.title = "Microbiome🦠".to_string();

    // Make a Context.
    let (ctx, event_loop) = ContextBuilder::new("petridish", "Adam McDaniel")
        .window_setup(window_setup)
        .build()
        .expect("Could not create ggez context!");

    event::run(ctx, event_loop, MyGame::default());
}

struct MyGame {
    game_over: bool,
    won: bool,
    // Your state here...
    last_update: std::time::Instant,
    this_player: Player,
    world: World,
}

impl Default for MyGame {
    fn default() -> MyGame {
        let name: [char; 32] = ['a'; 32];
        let color = microbiome::Color::rgb(random().abs(), random().abs(), random().abs());
        
        let mut world = World::new();
        let player = world.create_new_player(name, color);
        for _ in 0..20 {
            let pos = Position(random(), random());
            world.add_entity(Entity::Cell(Cell::new(Mass::default() * 500.0, pos, Direction::from_radians(random() * std::f64::consts::PI), Speed::default(), Some(player))));
        }
    
        let camera_pos = world.get_camera_position(&player);
    
        for _ in 0..40 {
            let name: [char; 32] = ['b'; 32];
            let color = microbiome::Color::rgb(random().abs(), random().abs(), random().abs());
            let enemy = world.create_new_player(name, color);
            let pos = Position(random(), random());
            let local_range = 0.05 + random() * 0.05;
            let scale = 15.0 + random() * 10.0;
            for _ in 0..rand::thread_rng().gen_range(2..10) {
                // Add random enemy cells
                
                let local_pos = pos + Position(random() * local_range, random() * local_range);
    
                world.add_entity(Entity::Cell(Cell::new(Mass::default() * scale.powi(2) * (1.0 + random() * 0.1) * 100.0, local_pos, Direction::from_radians(random() * std::f64::consts::PI), Speed::default(), Some(enemy))));
            }
        }
    
        world.get_player_cells_mut(&player).into_iter().for_each(|cell| {
            cell.set_position(Position(random() * 0.1, random() * 0.1));
        });
    
    
        for _ in 0..2000 {
            let pos = Position(random(), random());
            world.add_entity(Entity::Food(Food::new(Mass::default() * (random() + 2.0), pos)));
            // eprintln!("food added at {:?}", pos);
        }
    
        for _ in 0..30 {
            let pos = Position(random(), random());
            for _ in 0..50 {
                let local_pos = pos + Position(random() * 0.05, random() * 0.05);
                world.add_entity(Entity::Food(Food::new(Mass::default() * (random() + 3.0), local_pos)));
                // eprintln!("food added at {:?}", pos);
            }
            // eprintln!("food added at {:?}", pos);
        }
    
        let name: [char; 32] = ['a'; 32];
        let color = microbiome::Color::rgb(random().abs(), random().abs(), random().abs());
        
        let mut world = World::new();
        let player = world.create_new_player(name, color);
        for _ in 0..10 {
            let pos = Position(random(), random());
            world.add_entity(Entity::Cell(Cell::new(Mass::default() * 50.0, pos, Direction::from_radians(random() * std::f64::consts::PI), Speed::default(), Some(player))));
        }
    
        let camera_pos = world.get_camera_position(&player);
    
        for _ in 0..10 {
            let name: [char; 32] = ['b'; 32];
            let color = microbiome::Color::rgb(random().abs(), random().abs(), random().abs());
            let enemy = world.create_new_player(name, color);
            let pos = Position(random(), random());
            let local_range = 0.05 + random() * 0.05;
            for _ in 0..rand::thread_rng().gen_range(2..10) {
                // Add random enemy cells
                
                let local_pos = pos + Position(random() * local_range, random() * local_range);
    
                world.add_entity(Entity::Cell(Cell::new(Mass::default() * 500.0, local_pos, Direction::from_radians(random() * std::f64::consts::PI), Speed::default(), Some(enemy))));
            }
        }
    
        world.get_player_cells_mut(&player).into_iter().for_each(|cell| {
            cell.set_position(Position(random() * 0.1, random() * 0.1));
        });
    
    
        for _ in 0..2000 {
            let pos = Position(random(), random());
            world.add_entity(Entity::Food(Food::new(Mass::default() * (random() + 2.0), pos)));
            // eprintln!("food added at {:?}", pos);
        }
    
        for _ in 0..20 {
            let pos = Position(random(), random());
            for _ in 0..50 {
                let local_pos = pos + Position(random() * 0.05, random() * 0.05);
                world.add_entity(Entity::Food(Food::new(Mass::default() * (random() + 2.0), local_pos)));
                // eprintln!("food added at {:?}", pos);
            }
            // eprintln!("food added at {:?}", pos);
        }
    
        // Create an instance of your event handler.
        // Usually, you should provide it with the Context object to
        // use when setting your game up.
        MyGame {
            game_over: false,
            won: false,
            last_update: std::time::Instant::now(),
            this_player: player,
            world,
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.game_over {
            // Detect spacebar to restart
            if ctx.keyboard.is_key_just_pressed(ggez::input::keyboard::KeyCode::Space) {
                // Reset the game
                *self = MyGame::default();
            }
            return Ok(());
        }

        // Update code here...
        // Get mouse position
        let mouse_pos = ctx.mouse.position();
        let mouse_x = mouse_pos.x;
        let mouse_y = mouse_pos.y;

        // Position of the center of the screen
        let center_x = 400.0;
        let center_y = 300.0;

        // Get the distance between the mouse and the center of the screen
        let distance = (((mouse_x - center_x).powf(2.0) + (mouse_y - center_y).powf(2.0)).sqrt()) as f64;

        // Get the angle of the mouse
        let direction = Direction::from_radians((mouse_y - center_y).atan2(mouse_x - center_x).into());

        let speed = Speed::default() * (distance / 100.0);

        // Get the velocity of the player
        self.this_player.set_velocity(direction, speed);
        self.world.set_controls(&self.this_player, direction, speed);

        // Target the player with median total mass

        let highest_median_mass = self.world.get_players().into_iter().map(|player| {
            let mut cells = self.world.get_player_cells(&player);
            cells.sort_by(|a, b| {
                a.get_mass().partial_cmp(&b.get_mass()).unwrap_or(std::cmp::Ordering::Equal)
            });
            cells[cells.len()/2].get_mass().to_area()
        }).max_by(|a, b| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)).unwrap();

        let avg_population = self.world.get_players().into_iter().map(|player| {
            self.world.get_player_cells(&player).len()
        }).sum::<usize>() as f64 / self.world.get_players().len() as f64;

        let highest_population = self.world.get_players().into_iter().map(|player| {
            self.world.get_player_cells(&player).len()
        }).max_by(|a, b| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)).unwrap() as f64;

        let players = self.world.get_players().into_iter().map(|player| *player).collect::<Vec<_>>();
          
        let controls = players.par_iter().map(|player| {
            let avg_position = self.world.get_camera_position(&player);
            let target_player = {
                let mut players = self.world.get_players().into_iter().map(|player| *player).filter(|p| p != player).collect::<Vec<_>>();

                players.sort_by(|a, b| {
                    // let smallest_a_cell = self.world.get_player_cells(&a).into_iter().min_by(|a, b| a.get_mass().partial_cmp(&b.get_mass()).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
                    // let smallest_b_cell = self.world.get_player_cells(&b).into_iter().min_by(|a, b| a.get_mass().partial_cmp(&b.get_mass()).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
                    let a_pos = self.world.get_camera_position(&a);
                    let b_pos = self.world.get_camera_position(&b);
                    let a_mass = self.world.get_player_cells(&a).into_iter().map(|cell| cell.get_mass().to_area()).sum::<f64>() / a_pos.distance_to(avg_position).powi(1);
                    let b_mass = self.world.get_player_cells(&b).into_iter().map(|cell| cell.get_mass().to_area()).sum::<f64>() / b_pos.distance_to(avg_position).powi(1);
                    a_mass.partial_cmp(&b_mass).unwrap_or(std::cmp::Ordering::Equal)
                });
                // players[players.len() / 2]
                // if players.len() < 6 {
                //     players[players.len() - 1]
                // } else {
                //     players[players.len()/2]
                // }
                players[0]
            };

            if self.world.get_player_cells(&target_player).is_empty() {
                return None;
            }

            let smallest_cell = self.world.get_player_cells(&target_player).into_iter().min_by(|a, b| a.get_mass().partial_cmp(&b.get_mass()).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
            let largest_cell = self.world.get_player_cells(&target_player).into_iter().max_by(|a, b| a.get_mass().partial_cmp(&b.get_mass()).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
            let smallest_cell_pos = smallest_cell.get_position();
            let smallest_cell_mass = smallest_cell.get_mass().to_area();
            let largest_cell_mass = largest_cell.get_mass().to_area();
             
            if *player != self.this_player {
                // Get smallest cell for this player
                let enemy_cell = self.world.get_player_cells(&player).into_iter().max_by(|a, b| a.get_mass().partial_cmp(&b.get_mass()).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
                let enemy_cell_pos = enemy_cell.get_position();
                let enemy_cell_mass = enemy_cell.get_mass().to_area();
                let speed = Speed::default() * 3.5;
                let smallest_enemy_cell = self.world.get_player_cells(&player).into_iter().min_by(|a, b| a.get_mass().partial_cmp(&b.get_mass()).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
                let smallest_enemy_cell_mass = smallest_enemy_cell.get_mass().to_area();

                

                let closest_enemy_cell = self.world.get_player_cells(&player).into_iter().min_by(|a, b| {
                    let a_distance = a.get_position().distance_to(smallest_cell_pos);
                    let b_distance = b.get_position().distance_to(smallest_cell_pos);
                    a_distance.partial_cmp(&b_distance).unwrap_or(std::cmp::Ordering::Equal)
                }).unwrap();

                let avg_position = self.world.get_camera_position(&player);


                let mut weighted_directions = self.world.get_entities().par_iter().filter_map(|(_, entity)| {
                    match entity {
                        Entity::Food(food) => {
                            // Some(food.get_position() * ((food_mulitplier * food.to_mass().to_area()).powi(2) / food.get_position().distance_to(avg_position).powi(1)))
                            // Return direction towards food
                            let food_mass = food.to_mass().to_area();
                            let food_pos = food.get_position();
                            let food_distance = food_pos.distance_to(avg_position);
                            
                            let is_close = food_distance < food.to_mass().to_radius() * 200.0;
                            if is_close {
                                let closest_cell = self.world.get_player_cells(&player).into_iter().min_by(|a, b| {
                                    let a_distance = a.get_position().distance_to(food_pos);
                                    let b_distance = b.get_position().distance_to(food_pos);
                                    a_distance.partial_cmp(&b_distance).unwrap_or(std::cmp::Ordering::Equal)
                                }).unwrap();
                                Some((5.0 * food_mass / food_distance.powi(2), closest_cell.get_position().direction_to(food_pos), false))

                            } else {
                                Some((3.0 * food_mass / food_distance.powi(2), avg_position.direction_to(food_pos), false))
                            }
                        },
                        Entity::Cell(cell) => {
                            if cell.get_player() == Some(&player) {
                                return None;
                            }
                            let cell_pos = cell.get_position();
                            let cell_mass = cell.get_mass().to_area();
                            
                            // Get closest cell to this cell
                            let closest_cell = self.world.get_player_cells(&player).into_iter().min_by(|a, b| {
                                let a_distance = a.get_position().distance_to(cell_pos);
                                let b_distance = b.get_position().distance_to(cell_pos);
                                a_distance.partial_cmp(&b_distance).unwrap_or(std::cmp::Ordering::Equal)
                            }).unwrap();
                            let closest_cell_mass = closest_cell.get_mass().to_area();
                            let closest_cell_pos = closest_cell.get_position();
                            let cell_distance = cell.get_position().distance_to(closest_cell_pos);
                            let is_close = cell_distance < cell.get_mass().to_radius().max(closest_cell.get_mass().to_radius()) * 32.0;
                            if !is_close {
                                return None;
                            }

                            if cell_mass < closest_cell_mass * 0.3 {
                                // Some(cell.get_position() * (cell.get_mass().to_area().powi(4) / cell.get_position().distance_to(closest_cell.get_position()).powi(2)))
                                // Some(avg_position.move_towards(&avg_position.direction_to(cell.get_position()), cell.get_mass().to_area().powi(4) / cell.get_position().distance_to(closest_cell.get_position()).powi(1)))
                                Some((10.0 * cell_mass / cell_distance.powi(2), closest_cell_pos.direction_to(cell.get_position()), false))
                            } else if cell_mass < closest_cell_mass * 0.6 {
                                // Some(cell.get_position() * (cell.get_mass().to_area().powi(3) / cell.get_position().distance_to(closest_cell.get_position()).powi(2)))
                                // Some(avg_position.move_towards(&avg_position.direction_to(cell.get_position()), cell.get_mass().to_area().powi(3) / cell.get_position().distance_to(closest_cell.get_position()).powi(1)))
                            // } else if cell.get_mass().to_area() < closest_cell_mass * 0.8 {
                                // Some(cell.get_position() * (cell.get_mass().to_area().powi(2) / cell.get_position().distance_to(closest_cell.get_position()).powi(2)))
                                // Some(avg_position.move_towards(&avg_position.direction_to(cell.get_position()), cell.get_mass().to_area().powi(2) / cell.get_position().distance_to(closest_cell.get_position()).powi(1)))
                                Some((8.0 * cell_mass / cell_distance.powi(2), closest_cell_pos.direction_to(cell.get_position()), false))
                            // Detect if close to a large, threatening cell
                            } else if cell.get_mass().to_area() < closest_cell_mass {
                                // Some(cell.get_position() * (cell.get_mass().to_area().powi(3) / cell.get_position().distance_to(closest_cell.get_position()).powi(2)))
                                // Some(avg_position.move_towards(&avg_position.direction_to(cell.get_position()), cell.get_mass().to_area().powi(3) / cell.get_position().distance_to(closest_cell.get_position()).powi(1)))
                            // } else if cell.get_mass().to_area() < closest_cell_mass * 0.8 {
                                // Some(cell.get_position() * (cell.get_mass().to_area().powi(2) / cell.get_position().distance_to(closest_cell.get_position()).powi(2)))
                                // Some(avg_position.move_towards(&avg_position.direction_to(cell.get_position()), cell.get_mass().to_area().powi(2) / cell.get_position().distance_to(closest_cell.get_position()).powi(1)))
                                Some((6.0 * cell_mass / cell_distance.powi(2), closest_cell_pos.direction_to(cell.get_position()), false))
                            // Detect if close to a large, threatening cell
                            } else if cell.get_mass().to_area() > closest_cell_mass {
                                // food_mulitplier = food_mulitplier.powi(2);
                                // Some(avg_position.move_away(&avg_position.direction_to(cell.get_position()), 0.))
                                // None
                                // let direction_to_threat = cell.get_position().direction_to(avg_position);
                                // let heading = Direction::from_radians(direction_to_threat.to_radians() * (1.0 + random() * 0.2));
                                // Some(avg_position.move_towards(&heading, (cell.get_mass().to_area().powi(2) + closest_cell_mass.powi(2)) / cell.get_position().distance_to(closest_cell.get_position()).powi(2)))
                                Some((-10.0 * closest_cell_mass / cell_distance.powi(2), closest_cell_pos.direction_to(cell_pos), true))
                            } else {
                                None
                            }
                        },
                        _ => None,
                    }
                }).collect::<Vec<_>>();

                // Calculate target position based on weighted average
                // let weights: Vec<_> = weighted_directions.iter().map(|(weight, _)| *weight).collect();
                // let sum_of_weights = weights.iter().sum::<f64>();
                // let directions = weighted_directions.iter().map(|(_, direction)| direction).collect::<Vec<_>>();

                let mut x = 0.0;
                let mut y = 0.0;
            
                for (weight, direction, _) in &weighted_directions {
                    x += weight * direction.x_component();
                    y += weight * direction.y_component();
                }
            
                // Calculate the average angle
                let target_direction = Direction::from_vector(x, y);

                // println!("Targeting direction {:?}", target_direction);

                // let highest_avg_mass = self.world.get_players().into_iter().map(|player| {
                //     let avg_mass = self.world.get_player_cells(&player).into_iter().map(|cell| cell.get_mass().to_area()).sum::<f64>() / self.world.get_player_cells(&player).len() as f64;
                //     avg_mass
                // }).max_by(|a, b| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)).unwrap();

                // // Decide whether to perform mitosis
                // let median_mass = {
                //     let mut cells = self.world.get_player_cells(&player);
                //     cells.sort_by(|a, b| {
                //         a.get_mass().partial_cmp(&b.get_mass()).unwrap_or(std::cmp::Ordering::Equal)
                //     });
                //     cells[cells.len()/2].get_mass().to_area()
                // };

                // let highest_population = self.world.get_players().into_iter().map(|player| {
                //     self.world.get_player_cells(&player).len()
                // }).max_by(|a, b| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)).unwrap() as f64;
                // let median_population = self.world.get_players().into_iter().map(|player| {
                //     self.world.get_player_cells(&player).len()
                // }).nth(self.world.get_players().len() / 2).unwrap() as f64;

                // let direction = avg_position.direction_to(target_pos);
                
                // // let direction = Direction::from_radians(player.get_direction().to_radians() * (1.0 + random() * 0.5));
                // println!("Randomly moving player");
                // if smallest_cell_mass < enemy_cell_mass * 0.5 && closest_enemy_cell.get_position().distance_to(avg_position) < 0.1 {
                //     // Eat the enemy
                //     // let avg_position = self.world.get_camera_position(&player);
                //     let direction = closest_enemy_cell.get_position().direction_to(smallest_cell_pos);
                //     println!("Pointing to enemy at {:?}", smallest_cell_pos);
                //     self.world.set_controls(&player, direction, speed);
                //     // Count cells
                //     let num_cells = self.world.get_player_cells(&player).len();
                //     // Calculate average mass
                //     let avg_mass = self.world.get_player_cells(&player).into_iter().map(|cell| cell.get_mass().to_area()).sum::<f64>() / num_cells as f64;
                //     if avg_mass > Mass::default().to_area() * 100.0 && num_cells < 4 {
                //         self.world.mitosis(&player);
                //     } else if largest_cell_mass < smallest_enemy_cell_mass * 0.5 && num_cells < 8 && closest_enemy_cell_mass < smallest_enemy_cell_mass * 0.5 {
                //         self.world.mitosis(&player);
                //     }
                // } else if smallest_cell_mass < smallest_enemy_cell_mass && closest_enemy_cell.get_position().distance_to(avg_position) < 0.1 {
                //     // Run away
                //     // let avg_position = self.world.get_camera_position(&player);
                //     let direction = closest_enemy_cell.get_position().direction_to(smallest_cell_pos);
                //     println!("Pointing to enemy at {:?}", smallest_cell_pos);
                //     self.world.set_controls(&player, direction, speed);
                // } else {
                //     // Point towards food
                //     let food = self.world.get_entities().into_iter().filter_map(|(id, entity)| {
                //         match entity {
                //             Entity::Food(food) => {
                //                 Some((id, food))
                //             },
                //             _ => None,
                //         }
                //     }).min_by(|(_, a), (_, b)| {
                //         let a_distance = a.get_position().distance_to(avg_position) * a.to_mass().to_area();
                //         let b_distance = b.get_position().distance_to(avg_position) * b.to_mass().to_area();
                //         a_distance.partial_cmp(&b_distance).unwrap_or(std::cmp::Ordering::Equal)
                //     }).map(|(id, food)| {
                //         (id, food.get_position())
                //     });

                //     let direction = if let Some((_, food_pos)) = food {
                //         avg_position.direction_to(food_pos)
                //     } else {
                //         Direction::from_radians(random() * std::f64::consts::PI * 2.0)
                //     };                    
                //     // let direction = Direction::from_radians(player.get_direction().to_radians() * (1.0 + random() * 0.5));
                //     println!("Randomly moving player");
                //     self.world.set_controls(&player, direction, speed);
                // }
                return Some((player, target_direction, speed, weighted_directions.iter().map(|(_, _, should_split)| *should_split).any(|should_split| should_split)))
            } else {
                // return (self.this_player, Direction::from_radians(random() * std::f64::consts::PI * 2.0), Speed::default())
                return None
            }
        }).collect::<Vec<_>>();
        for (player, target_direction, speed, should_split) in controls.into_iter().filter(|control| control.is_some()).map(|control| control.unwrap()) {
            self.world.set_controls(&player, target_direction, speed);
            
            let my_population = self.world.get_player_cells(&player).len() as f64;
            // Calculate average mass
            // let my_avg_mass = self.world.get_player_cells(&player).into_iter().map(|cell| cell.get_mass().to_area()).sum::<f64>() / my_population;
            let my_median_mass = {
                let mut cells = self.world.get_player_cells(&player);
                cells.sort_by(|a, b| {
                    a.get_mass().partial_cmp(&b.get_mass()).unwrap_or(std::cmp::Ordering::Equal)
                });
                cells[cells.len()/2].get_mass().to_area()
            };

            if should_split && my_population < 512.0 && (my_median_mass.log10() / highest_median_mass.log10() > 2.0 || my_population.log10() / highest_population.log10() < 0.5) {
                self.world.mitosis(&player);
            }
        }
        // self.world.set_controls(&player, target_direction, speed);

        // detect spacebar
        if ctx.keyboard.is_key_just_released(ggez::input::keyboard::KeyCode::Space) {
            // Split the player
            self.world.mitosis(&self.this_player);
        }

        let median_cell_mass = {
            let mut cells = self.world.get_player_cells(&self.this_player);
            cells.sort_by(|a, b| {
                a.get_mass().partial_cmp(&b.get_mass()).unwrap_or(std::cmp::Ordering::Equal)
            });
            cells[cells.len()/5].get_mass().to_area()
        };

        // Update the world
        // let seconds_since_last_update = ctx.time.delta().as_secs_f64();
        // let seconds_since_last_update = 0.0;
        let seconds_since_last_update = self.last_update.elapsed().as_secs_f64() * 2.0 * median_cell_mass.log10() / Mass::default().to_area().log10();
        self.last_update = std::time::Instant::now();
        self.world.tick(seconds_since_last_update);



        // Detect win
        if self.world.get_players().len() == 1 && self.world.get_player_cells(&self.this_player).len() > 0 {
            self.won = true;
            self.game_over = true;
        }

        if self.won {
            return Ok(());
        }

        // Detect game over
        if self.world.get_cells().len() == 0 {
            self.game_over = true;
            self.won = false;
        }

        if self.world.get_player_cells(&self.this_player).len() == 0 {
            self.game_over = true;
            self.won = false;
        }

        if self.game_over {
            return Ok(());
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        let (screen_width, screen_height) = (800.0_f64, 600.0_f64);
        if self.won {
            // Game over
            let text = graphics::Text::new("You Win!");
            let text_x = screen_width as f32 / 2.0;
            let text_y = screen_height as f32 / 2.0;
            let color = Color::from_rgb(0, 0, 0);
            let text_pos = graphics::DrawParam::default().dest([text_x, text_y]).color(color);
            canvas.draw(&text, text_pos);
            return canvas.finish(ctx);
        }

        if self.game_over {
            // Game over
            let text = graphics::Text::new("Game Over");
            let text_x = screen_width as f32 / 2.0;
            let text_y = screen_height as f32 / 2.0;
            let color = Color::from_rgb(0, 0, 0);
            let text_pos = graphics::DrawParam::default().dest([text_x, text_y]).color(color);
            canvas.draw(&text, text_pos);
            return canvas.finish(ctx);
        }
        
        // Draw code here...
        let player = self.this_player;

        // Screen size is 800x600
        // Center of the screen is 400x300

        let smallest_cell = self.world.get_player_cells(&player).into_iter().max_by(|a, b| a.get_mass().partial_cmp(&b.get_mass()).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
        let smallest_cell_radius = smallest_cell.get_mass().to_radius();
        let default_cell_radius = Mass::default().to_radius();
        
        // let (camera_width, camera_height) = (smallest_cell_radius * 2.0 * 1.5, smallest_cell_radius * 2.0 * 1.5);
        // let (camera_width, camera_height) = (screen_width / (smallest_cell_radius / default_cell_radius), screen_height / (smallest_cell_radius / default_cell_radius));
        let (camera_width, camera_height) = (screen_width, screen_height);
        
        let (world_width, world_height) = (camera_width / (CAMERA_WIDTH * (smallest_cell_radius / default_cell_radius).sqrt()), camera_height / (CAMERA_HEIGHT * (smallest_cell_radius / default_cell_radius).sqrt()));

        let camera_pos = self.world.get_camera_position(&player);
        let (mut camera_x, mut camera_y) = camera_pos.project_onto(world_width, world_height);
        camera_x -= camera_width / 2.0;
        camera_y -= camera_height / 2.0;
        // Draw all the entities
        let entities = self.world.get_entities();


        // Draw grid lines
        let primary_color = Color::from_rgb(0, 0, 0);
        let alt_color = Color::from_rgb(0, 0, 255);
        
        for y in 0..256 {
            let color = if y % 16 == 0 {
                alt_color
            } else {
                primary_color
            };

            // Draw horizontal line across world
            let world_y = y as f64 / 256.0 * world_height;
            let (x1, y1) = (0.0, world_y);
            let (x2, y2) = (world_width, world_y);
            let (x1, y1) = (x1 - camera_x, y1 - camera_y);
            let (x2, y2) = (x2 - camera_x, y2 - camera_y);
            let line = graphics::Mesh::new_line(ctx, &[[x1 as f32, y1 as f32], [x2 as f32, y2 as f32]], 1.0, color)?;
            canvas.draw(&line, graphics::DrawParam::default());
        }

        for x in 0..256 {
            let color = if x % 16 == 0 {
                alt_color
            } else {
                primary_color
            };

            // Draw vertical line across world
            let world_x = x as f64 / 256.0 * world_width;
            let (x1, y1) = (world_x, 0.0);
            let (x2, y2) = (world_x, world_height);
            let (x1, y1) = (x1 - camera_x, y1 - camera_y);
            let (x2, y2) = (x2 - camera_x, y2 - camera_y);
            let line = graphics::Mesh::new_line(ctx, &[[x1 as f32, y1 as f32], [x2 as f32, y2 as f32]], 1.0, color)?;
            canvas.draw(&line, graphics::DrawParam::default());
        }

        for (id, entity) in entities {
            // eprintln!("id: {:?}", id);
            // Draw the entity
            // Get the position and radius

            match entity {
                Entity::Cell(cell) => {
                    // Get the color
                    let color: microbiome::Color = cell.get_player().unwrap().get_color();
                    // eprintln!("color: {:?}", color);
                    let ggez_color = Color::from_rgb(color.get_red(), color.get_green(), color.get_blue());

                    // Get radius and position
                    let radius = cell.get_mass().to_radius() * world_width / 2.0;
                    let (x, y) = cell.get_position().project_onto(world_width, world_height);
                    // let pos = cell.get_position();
                    
                    // println!("cell pos {}, {}", pos.0, pos.1);
                    // println!("  radius {}", cell.get_mass().to_radius());

                    // If out of bounds, don't draw
                    if x < camera_x || x > camera_x + camera_width || y < camera_y || y > camera_y + camera_height {
                        continue;
                    }
                    
                    // Draw the player
                    let circle = graphics::Mesh::new_circle(ctx, graphics::DrawMode::fill(), [(x - camera_x) as f32, (y - camera_y) as f32], radius as f32, 0.1, ggez_color)?;
                    canvas.draw(&circle, graphics::DrawParam::default());
                },
                Entity::Food(food) => {
                    // Get the color
                    let color = microbiome::Color::rgb(1.0, 0.0, 0.0);
                    let ggez_color = Color::from_rgb(color.get_red(), color.get_green(), color.get_blue());

                    let radius = food.to_mass().to_radius() * world_width / 2.0;
                    let (x, y) = food.get_position().project_onto(world_width, world_height);
                    // let pos = food.get_position();

                    if x < camera_x || x > camera_x + camera_width || y < camera_y || y > camera_y + camera_height {
                        continue;
                    }
                    // println!("food pos {}, {}", pos.0, pos.1);

                    // Draw the food
                    let circle = graphics::Mesh::new_circle(ctx, graphics::DrawMode::fill(), [(x - camera_x) as f32, (y - camera_y) as f32], radius as f32, 0.1, ggez_color)?;
                    // let circle = graphics::Mesh::new_circle(ctx, graphics::DrawMode::fill(), [x as f32, y as f32], radius as f32, 0.1, ggez_color)?;
                    canvas.draw(&circle, graphics::DrawParam::default());
                },
                Entity::Wall(wall) => {
                    let color = microbiome::Color::rgb(0.0, 0.0, 0.0);
                    let ggez_color = Color::from_rgb(color.get_red(), color.get_green(), color.get_blue());

                    let radius = wall.get_radius() * world_width;
                    let (x, y) = wall.get_position().project_onto(world_width, world_height);

                    if x < camera_x || x > camera_x + camera_width || y < camera_y || y > camera_y + camera_height {
                        continue;
                    }

                    // Draw the virus
                    let circle = graphics::Mesh::new_circle(ctx, graphics::DrawMode::fill(), [(x - camera_x) as f32, (y - camera_y) as f32], radius as f32, 0.1, ggez_color)?;
                    // let circle = graphics::Mesh::new_circle(ctx, graphics::DrawMode::fill(), [x as f32, y as f32], radius as f32, 0.1, ggez_color)?;
                    canvas.draw(&circle, graphics::DrawParam::default());
                },
            }
        }

        // eprintln!("done drawing");
        canvas.finish(ctx)
    }
}