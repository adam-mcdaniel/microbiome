use core::ops::{Neg, Add, Sub, Mul};
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};
use rand::Rng;
use std::collections::HashMap;

// Get random number from -1.0 to 1.0
pub fn random() -> f64 {
    rand::thread_rng().gen_range(-1.0..1.0)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct World {
    pub players: Vec<Player>,
    pub entities: HashMap<ID, Entity>,
}

impl World {
    pub fn new() -> World {
        World {
            players: Vec::new(),
            entities: HashMap::new(),
        }
    }

    pub fn create_new_player(&mut self, name: [char; 32], color: Color) -> Player {
        let player = Player::new(name, ID::new(), color);
        let mut cell = Cell::default();
        cell.set_player(player);
        // Add the player to the world
        
        self.players.push(player);
        self.add_entity(Entity::Cell(cell));
        // Create a new cell for the player
        player
    }

    pub fn get_camera_position(&self, player: &Player) -> Position {
        let player_cells = self.get_player_cells(player);
        let player_positions = player_cells.iter().map(|cell| cell.get_position() * cell.get_mass().to_area()).collect::<Vec<_>>();
        let total_mass = player_cells.iter().map(|cell| cell.get_mass().to_area()).sum::<f64>();
        Position::average(&player_positions) * (1.0 / (total_mass / player_cells.len() as f64))
    }

    pub fn set_controls(&mut self, player: &Player, direction: Direction, speed: Speed) {
        self.players.iter_mut().find(|p| p.get_id() == player.get_id()).map(|p| {
            p.direction = direction;
            p.speed = speed;
        });

        for cell in self.get_player_cells_mut(player) {
            cell.set_velocity(direction, speed);
        }
    }

    pub fn get_players(&self) -> Vec<&Player> {
        self.players.iter().collect()
    }

    pub fn get_player(&self, id: ID) -> Option<&Player> {
        self.players.iter().find(|player| player.get_id() == id)
    }

    pub fn get_cells(&self) -> Vec<&Cell> {
        self.entities.values().filter_map(|entity| {
            if let Entity::Cell(cell) = entity {
                Some(cell)
            } else {
                None
            }
        }).collect()
    }

    pub fn mitosis(&mut self, player: &Player) {
        let mut new_cells = Vec::new();
        let mut cells = self.get_player_cells_mut(player);
        cells.sort_by(|a, b| a.get_mass().to_area().partial_cmp(&b.get_mass().to_area()).unwrap());
        cells.reverse();
        let num_cells = cells.len();
        for cell in cells.iter_mut().filter(|c| c.age > 8.0).take(num_cells / 2 + 2) {
            new_cells.push(cell.mitosis());
        }

        new_cells.retain(|cell| cell.get_mass().to_area() > 0.0);
        // Limit to 256 cells
        new_cells.truncate(256);

        for cell in new_cells {
            self.add_entity(Entity::Cell(cell));
        }
    }

    pub fn get_cells_mut(&mut self) -> Vec<&mut Cell> {
        self.entities.values_mut().filter_map(|entity| {
            if let Entity::Cell(cell) = entity {
                Some(cell)
            } else {
                None
            }
        }).collect()
    }

    pub fn get_entity(&self, id: ID) -> Option<&Entity> {
        self.entities.get(&id)
    }

    pub fn get_entities(&self) -> Vec<(ID, &Entity)> {
        self.entities.iter().map(|(id, entity)| (*id, entity)).collect()
    }

    pub fn get_entity_mut(&mut self, id: ID) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    pub fn get_entities_mut(&mut self) -> Vec<(ID, &mut Entity)> {
        self.entities.iter_mut().map(|(id, entity)| (*id, entity)).collect()
    }

    pub fn remove_entity(&mut self, id: ID) -> Option<Entity> {
        let result = self.entities.remove(&id);
        // eprintln!("removing entity: {:?} => {:?}", id, &result);
        result
    }

    pub fn remove_player(&mut self, id: ID) {
        self.players.retain(|player| player.get_id() != id);
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(ID::new(), entity);
    }

    pub fn update_entity(&mut self, id: ID, new: Entity) {
        if let Some(old) = self.entities.get_mut(&id) {
            *old = new;
        }
    }

    pub fn get_player_cells(&self, player: &Player) -> Vec<&Cell> {
        let player_id = player.get_id();
        self.get_cells().into_iter().filter(|cell| {
            if let Some(player) = cell.get_player() {
                player.get_id() == player_id
            } else {
                false
            }
        }).collect()
    }

    pub fn get_player_cells_mut(&mut self, player: &Player) -> Vec<&mut Cell> {
        let player_id = player.get_id();
        self.get_cells_mut().into_iter().filter(|cell| {
            if let Some(player) = cell.get_player() {
                player.get_id() == player_id
            } else {
                false
            }
        }).collect()
    }

    pub fn player_from_id(&self, id: ID) -> Option<&Player> {
        self.players.iter().find(|player| player.get_id() == id)
    }

    pub fn tick(&mut self, seconds_since_last_tick: f64) {
        let entities = self.get_entities().into_iter().map(|(a, b)| (a, *b)).collect::<Vec<_>>();
        for (id, entity) in entities {
            match entity {
                Entity::Cell(mut cell) => {
                    cell.tick(id, seconds_since_last_tick, self);
                    self.update_entity(id, Entity::Cell(cell));
                }
                Entity::Food(mut food) => {
                    food.tick(id, seconds_since_last_tick, self);
                    self.update_entity(id, Entity::Food(food));
                }
                Entity::Wall(_) => {}
            }
        }

        let players = self.get_players().into_iter().map(|player| *player).collect::<Vec<_>>();
        for player in players {
            if self.get_player_cells(&player).len() == 0 {
                self.remove_player(player.get_id());
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Entity {
    Cell(Cell),
    Food(Food),
    Wall(Wall),
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Wall {
    position: Position,
    radius: f64,
}

impl Wall {
    pub fn new(position: Position, radius: f64) -> Wall {
        Wall {
            position,
            radius,
        }
    }

    pub fn get_position(&self) -> Position {
        self.position
    }

    pub fn get_radius(&self) -> f64 {
        self.radius
    }

    pub fn check_collisions(&self, cell: &mut Cell) {
        let Position(x, y) = self.position;
        let Position(x1, y1) = cell.get_position();
        let radius = cell.get_mass().to_radius();

        let distance = ((x - x1).powi(2) + (y - y1).powi(2)).sqrt();
        if distance < radius + self.radius {
            let direction = Direction::from_positions(self.position, cell.get_position());
            let direction = -direction;
            cell.move_towards(&direction, distance - radius - self.radius);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Player {
    name: [char; 32],
    id: ID,
    score: u32,
    color: Color,
    direction: Direction,
    speed: Speed,
}

impl Player {
    pub fn new(name: [char; 32], player_id: ID, color: Color) -> Player {
        Player {
            name,
            id: player_id,
            score: 0,
            color,
            direction: Direction::from_degrees(random() * 360.0),
            speed: Speed::default() * random(),
        }
    }

    pub fn set_velocity(&mut self, direction: Direction, speed: Speed) {
        self.direction = direction;
        self.speed = speed;
    }

    pub fn get_name(&self) -> String {
        self.name.iter().collect()
    }

    pub fn get_id(&self) -> ID {
        self.id
    }

    pub fn get_score(&self) -> u32 {
        self.score
    }

    pub fn get_color(&self) -> Color {
        self.color
    }

    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    pub fn get_speed(&self) -> Speed {
        self.speed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn rgb(r: f64, g: f64, b: f64) -> Color {
        Color {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        }
    }

    pub fn get_red(&self) -> u8 {
        self.r
    }

    pub fn get_green(&self) -> u8 {
        self.g
    }

    pub fn get_blue(&self) -> u8 {
        self.b
    }

    pub fn hsv(h: f64, s: f64, v: f64) -> Color {
        let h = h % 360.0;
        let s = s.max(0.0).min(1.0);
        let v = v.max(0.0).min(1.0);

        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        if h < 60.0 {
            Color::rgb(c + m, x + m, m)
        } else if h < 120.0 {
            Color::rgb(x + m, c + m, m)
        } else if h < 180.0 {
            Color::rgb(m, c + m, x + m)
        } else if h < 240.0 {
            Color::rgb(m, x + m, c + m)
        } else if h < 300.0 {
            Color::rgb(x + m, m, c + m)
        } else {
            Color::rgb(c + m, m, x + m)
        }
    }

    pub fn to_rgb(&self) -> (f64, f64, f64) {
        let Color { r, g, b } = self;
        (f64::from(*r) / 255.0, f64::from(*g) / 255.0, f64::from(*b) / 255.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct ID(u32);

static mut NEXT_ID: u32 = 0;

impl ID {
    pub fn new() -> ID {
        unsafe {
            let id = NEXT_ID;
            NEXT_ID += 1;
            ID(id)
        }
    }

    pub fn to_number(&self) -> u32 {
        let ID(id) = self;
        *id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Cell {
    // The mass of the cell
    mass: Mass,
    // The position of the cell
    position: Position,
    // The direction of the cell
    direction: Direction,
    // The speed of the cell
    speed: Speed,
    // The cell's owner
    player: Option<Player>,
    // The cell's age
    age: f64,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            mass: Mass::default() * 10.0,
            position: Position(random(), random()),
            direction: Direction::from_radians(random() * std::f64::consts::PI),
            speed: Speed::default(),
            player: None,
            age: 0.0,
        }
    }
}

impl Cell {
    // Create a new cell
    pub fn new(mass: Mass, position: Position, direction: Direction, speed: Speed, player: Option<Player>) -> Cell {
        Cell {
            mass,
            position,
            direction,
            speed,
            player,
            age: 0.0,
        }
    }

    pub fn set_player(&mut self, player: Player) {
        self.player = Some(player);
    }

    // Get the mass of the cell
    pub fn get_mass(&self) -> Mass {
        self.mass
    }

    pub fn get_radius(&self) -> f64 {
        self.mass.to_radius()
    }

    // Eat a food
    pub fn eat(&mut self, food: &Food) {
        self.mass = self.mass + food.to_mass();
    }

    // Get the position of the cell
    pub fn get_position(&self) -> Position {
        self.position
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    // Get the direction of the cell
    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    // Move the cell
    pub fn move_towards(&mut self, direction: &Direction, distance: f64) {
        self.position = self.position.move_towards(direction, distance);
    }

    // Move the cell
    pub fn move_away(&mut self, direction: &Direction, distance: f64) {
        self.position = self.position.move_away(direction, distance);
    }

    // Get the speed of the cell
    pub fn get_speed(&self) -> Speed {
        self.speed
    }

    // Get the player of the cell
    pub fn get_player(&self) -> Option<&Player> {
        self.player.as_ref()
    }

    pub fn get_player_id(&self) -> Option<ID> {
        self.player.as_ref().map(|player| player.get_id())
    }

    pub fn can_swallow_cell(&self, other: &Cell) -> bool {
        if other.player == self.player && (self.age < 5.0 || other.age < 5.0) {
            return false;
        }

        // Check if the cell is bigger than the other cell
        self.get_mass().to_area() > other.get_mass().to_area() * 1.1
            // Check if the cell is close enough to the other cell
            && self.get_position().distance_to(other.get_position()) < self.get_mass().to_radius() + other.get_radius() * (if other.player == self.player { 4.0 / 5.0 } else { 2.0 / 3.0 })
    }

    pub fn can_swallow_food(&self, food: &Food) -> bool {
        // Check if the cell is bigger than the food
        self.get_position().distance_to(food.get_position()) < self.get_mass().to_radius() + food.to_radius()
    }

    pub fn eat_food(&mut self, food: &Food) {
        self.mass = self.mass + food.to_mass();
    }

    pub fn eat_cell(&mut self, cell: &Cell) {
        self.mass = self.mass + cell.get_mass();
    }

    // Live a tick
    pub fn tick(&mut self, my_id: ID, seconds_since_last_tick: f64, world: &mut World) {
        // Move the cell
        self.update_controls(world);
        self.move_towards(&self.get_direction(), self.mass.calculate_slowness(self.get_speed()).to_distance(seconds_since_last_tick));

        self.apply_friction(seconds_since_last_tick);

        // Check if the cell is out of bounds
        let Position(mut x, mut y) = self.get_position();
        x = x.max(-1.0).min(1.0);
        y = y.max(-1.0).min(1.0);
        self.position = Position(x, y);

        let mut eaten_ids = Vec::new();
        for (id, entity) in world.get_entities_mut() {
            if id == my_id {
                continue;
            }
            match entity {
                Entity::Food(food) => {
                    if self.can_swallow_food(food) {
                        self.eat_food(food);
                        eaten_ids.push(id);
                    }
                }
                Entity::Cell(cell) => {
                    if self.can_swallow_cell(cell) {
                        self.eat_cell(cell);
                        eaten_ids.push(id);
                    }
                }
                Entity::Wall(wall) => {
                    // Check if the cell is out of bounds
                    wall.check_collisions(self);
                }
            }
        }

        for id in eaten_ids {
            world.remove_entity(id);
        }

        // Count number of food available
        let mut num_food = 0;
        for (_, entity) in world.get_entities() {
            if let Entity::Food(_) = entity {
                num_food += 1;
            }
        }

        if num_food < 1000 {
            for _ in 0..(10.0 * seconds_since_last_tick).abs().round() as usize {
                world.add_entity(Entity::Food(Food::default()));
            }
        }

        self.age += seconds_since_last_tick;
        self.mass = self.mass * (1.0 - 0.03 * seconds_since_last_tick);
    }

    // Make the cell follow the player's controls
    pub fn update_controls(&mut self, world: &World) {
        // Get the current speed and direction from the player
        if let Some(id) = self.get_player_id() {
            if let Some(player) = world.player_from_id(id) {
                // Update the cell's direction
                self.set_velocity(player.get_direction(), player.get_speed());
            }
        }
    }

    pub fn set_velocity(&mut self, direction: Direction, speed: Speed) {
        self.direction = direction;
        self.speed = speed;
    }

    pub fn apply_friction(&mut self, seconds_since_last_tick: f64) {
        self.speed = self.speed * (1.0 - 0.1 * seconds_since_last_tick);
    }

    pub fn mitosis(&mut self) -> Cell {
        let mass = self.get_mass();
        let radius = mass.to_radius();
        let area = mass.to_area();
        let position = self.get_position();
        let direction = self.get_direction();
        let speed = self.get_speed();
        let player = self.get_player().cloned();

        let mut cell = Cell::new(Mass::from_area(area / 2.0), position, direction, speed * 2.0, player);
        cell.move_towards(&direction, radius * 1.5);
        self.mass = Mass::from_area(area / 2.0);
        self.speed = speed * -2.0;
        self.move_away(&direction, radius * 1.5);
        self.age = 0.0;
        cell
    }
}


#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Food {
    mass: Mass,
    position: Position,
}

impl Default for Food {
    fn default() -> Self {
        Food {
            // Create random food
            mass: Mass::default() * (random().abs() + 1.0) * 10.0,
            position: Position(random(), random()),
        }
    }
}

impl Food {
    pub fn new(mass: Mass, position: Position) -> Food {
        Food {
            mass,
            position,
        }
    }

    pub fn to_mass(&self) -> Mass {
        self.mass
    }

    pub fn to_radius(&self) -> f64 {
        self.mass.to_radius()
    }

    pub fn get_position(&self) -> Position {
        self.position
    }
    
    pub fn tick(&mut self, my_id: ID, seconds_since_last_tick: f64, _world: &mut World) {
        // Grow the food with compound interest
        self.mass = self.mass * (1.0 + 0.01 * seconds_since_last_tick);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Mass(pub f64);

impl Default for Mass {
    fn default() -> Self {
        Mass::from_radius(1.0 / 1024.0)
    }
}

impl Mass {
    pub fn calculate_slowness(&self, speed: Speed) -> Speed {
        let default_mass = Self::default().0;
        
        // Bigger cells are slower
        let slowness = (default_mass / self.0).sqrt();

        // The speed is multiplied by the slowness
        speed * 5.0 * slowness
    }

    pub fn to_radius(&self) -> f64 {
        let Mass(mass) = self;
        mass.sqrt()
    }

    pub fn from_radius(radius: f64) -> Mass {
        Mass(radius.powi(2))
    }

    pub fn to_area(&self) -> f64 {
        std::f64::consts::PI * self.0
    }

    pub fn from_area(area: f64) -> Mass {
        Mass(area / std::f64::consts::PI)
    }
}

impl Add for Mass {
    type Output = Mass;

    fn add(self, other: Mass) -> Self::Output {
        let Mass(mass1) = self;
        let Mass(mass2) = other;
        Mass(mass1 + mass2)
    }
}

impl Mul<f64> for Mass {
    type Output = Mass;

    fn mul(self, other: f64) -> Self::Output {
        let Mass(mass) = self;
        Mass(mass * other)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Position(pub f64, pub f64);

impl Position {
    pub fn average(positions: &[Position]) -> Position {
        let mut x = 0.0;
        let mut y = 0.0;
        for Position(x1, y1) in positions {
            x += x1;
            y += y1;
        }
        let len = positions.len() as f64;
        Position(x / len, y / len)
    }

    pub fn get_x(&self) -> f64 {
        let Position(x, _) = self;
        *x
    }

    pub fn get_y(&self) -> f64 {
        let Position(_, y) = self;
        *y
    }

    pub fn direction_to(&self, other: Position) -> Direction {
        Direction::from_positions(*self, other)
    }

    pub fn distance_to(&self, other: Position) -> f64 {
        let Position(x1, y1) = self;
        let Position(x2, y2) = other;
        ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
    }

    pub fn move_towards(&self, direction: &Direction, distance: f64) -> Position {
        let (x, y) = direction.to_vector();
        let Position(x1, y1) = self;
        Position(x1 + x * distance, y1 + y * distance)
    }

    pub fn move_away(&self, direction: &Direction, distance: f64) -> Position {
        let (x, y) = direction.to_vector();
        let Position(x1, y1) = self;
        Position(x1 - x * distance, y1 - y * distance)
    }

    pub fn project_onto(&self, width: f64, height: f64) -> (f64, f64) {
        let Position(x, y) = self;
        let x = (x + 1.0) * width / 2.0;
        let y = (y + 1.0) * height / 2.0;
        (x, y)
    }
}

impl From<(f64, f64)> for Position {
    fn from((x, y): (f64, f64)) -> Self {
        Position(x, y)
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, other: Position) -> Self::Output {
        let Position(x1, y1) = self;
        let Position(x2, y2) = other;
        Position(x1 + x2, y1 + y2)
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(self, other: Position) -> Self::Output {
        let Position(x1, y1) = self;
        let Position(x2, y2) = other;
        Position(x1 - x2, y1 - y2)
    }
}

impl Mul for Position {
    type Output = Position;

    fn mul(self, other: Position) -> Self::Output {
        let Position(x1, y1) = self;
        let Position(x2, y2) = other;
        Position(x1 * x2, y1 * y2)
    }
}

impl Mul<f64> for Position {
    type Output = Position;

    fn mul(self, other: f64) -> Self::Output {
        let Position(x, y) = self;
        Position(x * other, y * other)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Direction(pub f64);

impl Default for Direction {
    fn default() -> Self {
        Direction(0.0)
    }
}

impl Direction {
    pub fn rotate_by_degrees(&self, degrees: f64) -> Direction {
        let Direction(angle) = self;
        Direction(angle + degrees.to_radians())
    }

    pub fn rotate_by_radians(&self, radians: f64) -> Direction {
        let Direction(angle) = self;
        Direction(angle + radians)
    }

    pub fn from_degrees(degrees: f64) -> Direction {
        Direction(degrees.to_radians())
    }

    pub fn to_degrees(&self) -> f64 {
        let Direction(angle) = self;
        angle.to_degrees()
    }

    pub fn from_radians(radians: f64) -> Direction {
        Direction(radians % (2.0 * std::f64::consts::PI))
    }

    pub fn to_radians(&self) -> f64 {
        let Direction(angle) = self;
        *angle % (2.0 * std::f64::consts::PI)
    }

    pub fn to_vector(&self) -> (f64, f64) {
        let Direction(angle) = self;
        let x = angle.cos();
        let y = angle.sin();
        (x, y)
    }

    pub fn from_vector(x: f64, y: f64) -> Direction {
        Direction(y.atan2(x) % (2.0 * std::f64::consts::PI))
    }

    pub fn from_positions(from: Position, to: Position) -> Direction {
        let Position(x1, y1) = from;
        let Position(x2, y2) = to;
        Direction::from_vector(x2 - x1, y2 - y1)
    }

    pub fn turn(&self, angle: f64) -> Direction {
        let Direction(current_angle) = self;
        Direction((current_angle + angle) % (2.0 * std::f64::consts::PI))
    }

    pub fn x_component(&self) -> f64 {
        let Direction(angle) = self;
        angle.cos()
    }

    pub fn y_component(&self) -> f64 {
        let Direction(angle) = self;
        angle.sin()
    }
}

impl Neg for Direction {
    type Output = Direction;

    fn neg(self) -> Self::Output {
        let Direction(angle) = self;
        Direction(-angle % (2.0 * std::f64::consts::PI))
    }
}

impl Add for Direction {
    type Output = Direction;

    fn add(self, other: Direction) -> Self::Output {
        let Direction(angle1) = self;
        let Direction(angle2) = other;
        Direction((angle1 + angle2) % (2.0 * std::f64::consts::PI))
    }
}

impl Mul<f64> for Direction {
    type Output = Direction;

    fn mul(self, other: f64) -> Self::Output {
        let Direction(angle) = self;
        Direction((angle * other) % (2.0 * std::f64::consts::PI))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Speed(pub f64);

impl Default for Speed {
    fn default() -> Self {
        Speed(0.025)
    }
}

impl Speed {
    pub fn to_vector(&self, direction: &Direction) -> (f64, f64) {
        let (x, y) = direction.to_vector();
        let Speed(speed) = self;
        (x * speed, y * speed)
    }

    pub fn from_vector(x: f64, y: f64) -> Speed {
        Speed((x.powi(2) + y.powi(2)).sqrt())
    }

    pub fn to_speed(&self) -> f64 {
        let Speed(speed) = self;
        *speed
    }

    pub fn from_speed(speed: f64) -> Speed {
        Speed(speed)
    }

    pub fn to_distance(&self, time: f64) -> f64 {
        let Speed(speed) = self;
        speed * time
    }
}

impl Mul<f64> for Speed {
    type Output = Speed;

    fn mul(self, other: f64) -> Self::Output {
        let Speed(speed) = self;
        Speed(speed * other)
    }
}