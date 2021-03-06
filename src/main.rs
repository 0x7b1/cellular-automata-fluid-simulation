use std::fs::File;
use std::io::{BufReader, BufRead, Read};
use std::path::Path;
use rand::Rng;
use std::time::Instant;

use minifb::{
    Key,
    WindowOptions,
    Window,
    MouseMode,
    MouseButton,
    KeyRepeat,
    CursorStyle,
};
use vek::Vec2;
use std::ops::Sub;

const WIDTH: usize = 300;
const HEIGHT: usize = 300;

const FRAME_DELAY: u64 = 0;
const MIN_FLOW: f32 = 0.01;
const MAX_MASS: f32 = 10.0;
const MAX_COMPRESS: f32 = 0.02;
const MIN_MASS: f32 = 0.0001;
const MIN_DRAW: f32 = 0.01;
const MAX_DRAW: f32 = 1.1;
const MAX_SPEED: f32 = 1.0;

#[derive(Clone, PartialEq)]
enum Color {
    Black,
    White,
    Red,
    Yellow,
    Green,
    Cyan,
    Purple,
    Blue,
    Desert,
}

impl Color {
    pub fn get_hex(&self) -> u32 {
        use Color::*;

        match self {
            Black => 0x000000,
            White => 0xffffff,
            Red => 0xff0000,
            Yellow => 0xffff00,
            Green => 0x00ff00,
            Cyan => 0x00ffff,
            Blue => 0x0000ff,
            Purple => 0xff00ff,
            Desert => 0xccae62,
        }
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
enum Cell {
    Water,
    Ground,
    Air,
}

impl Cell {
    fn empty() -> Self {
        Cell::Air
    }
}

#[derive(Copy, Clone)]
struct Widget {
    active: bool,
    element: Cell,
}

impl Widget {
    fn new(element: Cell) -> Self {
        Widget {
            active: false,
            element,
        }
    }

    fn toggle(&mut self) {
        self.active = !self.active;
    }

    fn get_color(&self) -> Color {
        match self.active {
            true => Color::Red,
            false => Color::Yellow,
        }
    }
}

struct World {
    water: Box<Vec<i32>>,
    energy: Box<Vec<i32>>,
    ground: Box<Vec<i32>>,

    mass: Box<Vec<Vec<f32>>>,
    new_mass: Box<Vec<Vec<f32>>>,
    blocks: Box<Vec<Vec<Cell>>>,

    widgets: Vec<Widget>,
    selected_element: Cell,
}

pub fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
    if val < min {
        min
    } else {
        if val > max {
            max
        } else {
            val
        }
    }
}

pub fn lerp_range(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

#[test]
fn test_lerp() {
    assert_eq!(lerp_range(5.0, 0.0, 10.0, 0.0, 100.0), 50.0);
}

impl World {
    fn new() -> Self {
        let mut this = Self {
            water: Box::new(vec![0; WIDTH]),
            energy: Box::new(vec![0; WIDTH]),
            ground: Box::new(vec![0; WIDTH]),

            mass: Box::new(vec![vec![0.0; WIDTH]; HEIGHT]),
            new_mass: Box::new(vec![vec![0.0; WIDTH]; HEIGHT]),
            blocks: Box::new(vec![vec![Cell::empty(); WIDTH]; HEIGHT]),

            widgets: Vec::new(),
            selected_element: Cell::Ground,
        };

        this.widgets.push(Widget::new(Cell::Ground));
        this.widgets.push(Widget::new(Cell::Water));
        this.select_element(Cell::Ground);

        this
    }

    fn get_stable_state(&self, total_mass: f32) -> f32 {
        if total_mass <= 1.0 {
            1.0
        } else if total_mass < 2.0 * MAX_MASS + MAX_COMPRESS {
            (MAX_MASS.powi(2) + total_mass * MAX_COMPRESS) / (MAX_MASS + MAX_COMPRESS)
        } else {
            (total_mass + MAX_COMPRESS) / 2.0
        }
    }

    fn clear_map(&mut self) {
        self.water = Box::new(vec![0; WIDTH]);
        self.energy = Box::new(vec![0; WIDTH]);
        self.ground = Box::new(vec![0; WIDTH]);
        self.mass = Box::new(vec![vec![0.0; WIDTH]; HEIGHT]);
        self.new_mass = Box::new(vec![vec![0.0; WIDTH]; HEIGHT]);
        self.blocks = Box::new(vec![vec![Cell::empty(); WIDTH]; HEIGHT]);
    }

    fn tick(&mut self) {
        let mut flow = 0.0;
        let mut blocks = self.blocks.clone();
        let mass = self.mass.clone();
        let mut new_mass = self.new_mass.clone();
        // let mut new_mass = [[0.0; WIDTH]; HEIGHT];
        let mut remaining_mass;

        // Calculate and apply flow for each block
        for x in 0..WIDTH - 1 {
            for y in 0..HEIGHT - 1 {
                // Skip inert ground blocks
                if blocks[x][y] == Cell::Ground {
                    continue;
                }

                // Custom push-only flow
                flow = 0.0;
                remaining_mass = mass[x][y];

                if remaining_mass <= 0.0 {
                    continue;
                }

                // The block below this one
                if blocks[x][y + 1] != Cell::Ground {
                    flow = self.get_stable_state(remaining_mass + mass[x][y + 1]) - mass[x][y + 1];
                    if flow > MIN_FLOW {
                        flow *= 0.8; // leads to smoother flow
                    }

                    flow = clamp(flow, 0.0, remaining_mass.min(MAX_SPEED));

                    new_mass[x][y] -= flow;
                    new_mass[x][y + 1] += flow;
                    remaining_mass -= flow;
                }

                if remaining_mass <= 0.0 {
                    continue;
                }

                // Left
                if blocks[x - 1][y] != Cell::Ground {
                    // Equalize the amount of water in this block and its neighbor
                    flow = (mass[x][y] - mass[x - 1][y]) / 4.0;
                    if flow > MIN_FLOW {
                        flow *= 0.8;
                    }
                    flow = clamp(flow, 0.0, remaining_mass);

                    new_mass[x][y] -= flow;
                    new_mass[x - 1][y] += flow;
                    remaining_mass -= flow;
                }

                if remaining_mass <= 0.0 {
                    continue;
                }

                // Right
                if blocks[x + 1][y] != Cell::Ground {
                    flow = (mass[x][y] - mass[x + 1][y]) / 4.0;
                    if flow > MIN_FLOW {
                        flow *= 0.8;
                    }

                    flow = clamp(flow, 0.0, remaining_mass);

                    new_mass[x][y] -= flow;
                    new_mass[x + 1][y] += flow;
                    remaining_mass -= flow;
                }

                if remaining_mass <= 0.0 {
                    continue;
                }

                // Up. Only compressed water flows upwards
                if blocks[x][y - 1] != Cell::Ground {
                    flow = remaining_mass - self.get_stable_state(remaining_mass + mass[x][y - 1]);
                    if flow >= MIN_FLOW {
                        flow *= 0.8;
                    }

                    flow = clamp(flow, 0.0, remaining_mass.min(MAX_SPEED));

                    new_mass[x][y] -= flow;
                    new_mass[x][y - 1] += flow;
                    remaining_mass -= flow;
                }
            }
        }

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                // Skip ground blocks
                if blocks[x][y] == Cell::Ground {
                    continue;
                }

                // Flag/unflag water blocks
                if mass[x][y] > MIN_MASS {
                    blocks[x][y] = Cell::Water;
                } else {
                    blocks[x][y] = Cell::Air;
                }
            }
        }

        // remove any water that has left the map
        for x in 0..WIDTH {
            new_mass[x][0] = 0.0;
            new_mass[x][HEIGHT - 1] = 0.0;
        }

        for y in 0..HEIGHT {
            new_mass[0][y] = 0.0;
            new_mass[WIDTH - 1][y] = 0.0;
        }

        self.mass = new_mass.clone();
        self.new_mass = new_mass.clone();
        self.blocks = blocks;
    }

    fn draw_element(&mut self, x: usize, y: usize) {
        match self.selected_element {
            Cell::Water => {
                self.blocks[x][y] = Cell::Water;
                // self.mass[x][y] = MAX_MASS;
                self.mass[x][y] = MAX_MASS * 10.0;
                self.mass[x + 1][y] = MAX_MASS * 10.0;
                self.mass[x + 2][y] = MAX_MASS * 10.0;
                self.mass[x - 1][y] = MAX_MASS * 10.0;
                self.mass[x - 2][y] = MAX_MASS * 10.0;
            }
            Cell::Ground => {
                self.blocks[x][y] = Cell::Ground;
                self.blocks[x + 1][y] = Cell::Ground;
                self.blocks[x - 1][y] = Cell::Ground;
                self.blocks[x][y + 1] = Cell::Ground;
                self.blocks[x][y - 1] = Cell::Ground;
            }
            Cell::Air => {
                self.mass[x][y] = 0.0;
                self.mass[x + 1][y] = 0.0;
                self.mass[x - 1][y] = 0.0;
                self.mass[x][y + 1] = 0.0;
                self.mass[x][y - 1] = 0.0;
            }
            _ => (),
        }
    }

    fn get_water_color(&self, mut mass: f32) -> u32 {
        mass = clamp(mass, MIN_MASS, MAX_MASS);
        let mut g = 50.0;
        let mut r = 50.0;
        let mut b;

        if (mass < 1.0) {
            b = lerp_range(mass, 0.01, 1.0, 255.0, 200.0);
            r = lerp_range(mass, 0.01, 1.0, 240.0, 50.0);
            r = clamp(r, 50.0, 240.0);
            g = r;
        } else {
            b = lerp_range(mass, 1.0, 1.1, 90.0, 140.0);
        }

        (1 << 24) + ((r as u32) << 16) + ((g as u32) << 8) + b as u32
    }

    fn render(&self, buff: &mut [u32]) {
        self.render_simulation(buff);
        self.render_widgets(buff);
    }

    fn render_simulation(&self, buff: &mut [u32]) {
        let blocks = self.blocks.clone();
        let mass = self.mass.clone();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let current_cell = self.blocks[x][y];

                buff[y * WIDTH + x] = match current_cell {
                    // Cell::Water => Color::Blue.get_hex(),
                    Cell::Water => self.get_water_color(mass[x][y]),
                    // Cell::Water => Color::Red.get_hex(),
                    Cell::Air => Color::Black.get_hex(),
                    Cell::Ground => Color::Desert.get_hex(),
                    _ => 0,
                }
            }
        }
    }

    fn render_widgets(&self, buff: &mut [u32]) {
        let widgets = self.widgets.clone();
        let mut x: usize = 5;
        let y: usize = 5;
        let square_length: usize = 5;
        let padding: usize = 5;

        for w in widgets {
            self.render_rectangle(buff, Vec2::new(x, y), square_length, w.get_color());
            x += square_length + padding;
        }
    }

    fn render_rectangle(&self, buff: &mut [u32], point: Vec2<usize>, length: usize, color: Color) {
        for y in point.y..point.y + length {
            for x in point.x..point.x + length {
                buff[y * WIDTH + x] = color.get_hex();
            }
        }
    }

    fn select_element(&mut self, cell_element: Cell) {
        self.selected_element = cell_element;

        for mut w in self.widgets.iter_mut() {
            if w.element == cell_element {
                w.active = true;
            } else {
                w.active = false;
            }
        }
    }

    fn rotate_canvas_anticlockwise(&mut self) {
        let mut blocks = self.blocks.clone();

        // Processing each block one by one
        for i in 0..WIDTH / 2 {

            // Processing elements in group of 4 in the current square
            for j in i..HEIGHT - i - 1 {
                // Storing current cell in a temporal variable
                let tmp_block = blocks[i][j];

                // Move values from right to top
                blocks[i][j] = blocks[j][HEIGHT - 1 - i];

                // Move values from bottom to right
                blocks[j][HEIGHT - 1 - i] = blocks[HEIGHT - 1 - i][HEIGHT - 1 - j];

                // Move values from left to bottom
                blocks[HEIGHT - 1 - i][HEIGHT - 1 - j] = blocks[HEIGHT - 1 - j][i];

                // Assign temporal to left
                blocks[HEIGHT - 1 - j][i] = tmp_block;
            }
        }

        self.blocks = blocks;
    }

    fn rotate_canvas_clockwise(&mut self) {
        let mut blocks = self.blocks.clone();

        // Traverse each cycle
        for i in 0..WIDTH / 2 {
            for j in i..HEIGHT - i - 1 {
                // Swap elements of each cycle in clockwise direction
                let tmp_block = blocks[i][j];
                blocks[i][j] = blocks[HEIGHT - 1 - j][i];
                blocks[HEIGHT - 1 - j][i] = blocks[HEIGHT - 1 - i][HEIGHT - 1 - j];
                blocks[HEIGHT - 1 - i][HEIGHT - 1 - j] = blocks[j][HEIGHT - 1 - i];
                blocks[j][HEIGHT - 1 - i] = tmp_block;
            }
        }

        self.blocks = blocks;
    }

    fn count_neighbours(&self, map: [[bool; WIDTH]; HEIGHT], x: usize, y: usize) -> i32 {
        let mut count_n = 0;

        for ii in 0..3 {
            for jj in 0..3 {
                let i = ii as i32 - 1;
                let j = jj as i32 - 1;

                if i == 0 && j == 0 {
                    continue;
                }

                let n_x = x as i32 + i;
                let n_y = y as i32 + j;

                if n_x < 0 || n_y < 0 || n_x >= WIDTH as i32 || n_y >= HEIGHT as i32 {
                    count_n += 1;
                } else if map[n_x as usize][n_y as usize] {
                    count_n += 1;
                }
            }
        }

        count_n
    }

    fn do_cave_generation_step(&self, old_map: [[bool; WIDTH]; HEIGHT]) -> [[bool; WIDTH]; HEIGHT] {
        let mut new_map = [[false; WIDTH]; HEIGHT];
        let death_limit = 3;
        let birth_limit = 4;
        // let birth_limit = 3;

        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                let nbs = self.count_neighbours(old_map, i, j);
                if old_map[i][j] {
                    if nbs < death_limit {
                        new_map[i][j] = false;
                    } else {
                        new_map[i][j] = true;
                    }
                } else {
                    if nbs > birth_limit {
                        new_map[i][j] = true;
                    } else {
                        new_map[i][j] = false;
                    }
                }
            }
        }

        new_map
    }

    fn initialize_cave(&self) -> [[bool; WIDTH]; HEIGHT] {
        let mut cave_map = [[false; WIDTH]; HEIGHT];
        let chance_to_start_alive = 0.35;

        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                let chance: f64 = rand::thread_rng().gen();
                if (chance < chance_to_start_alive) {
                    cave_map[i][j] = true;
                    // blocks[i][j] = Cell::Ground;
                }
            }
        }

        cave_map
    }

    fn generate_map(&mut self) {
        self.clear_map();
        let mut blocks = self.blocks.clone();
        let mut cave_map = self.initialize_cave();

        for _ in 0..3 {
            cave_map = self.do_cave_generation_step(cave_map);
        }

        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                if cave_map[i][j] {
                    blocks[i][j] = Cell::Ground;
                } else {
                    blocks[i][j] = Cell::Air;
                }
            }
        }

        self.blocks = blocks;
    }
}

fn cpu_rendering() {
    let mut world = World::new();

    let mut buff = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new(
        "CA Water Simulation",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: minifb::Scale::X2,
            ..WindowOptions::default()
        },
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(FRAME_DELAY)));
    window.set_cursor_style(CursorStyle::Crosshair);

    // let mut fps_now = Instant::now();
    // let mut fps_time_diff;
    // let mut fps_counter = 0;
    // let mut tick_time;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.get_keys_pressed(KeyRepeat::No).map(|keys| {
            for t in keys {
                match t {
                    Key::Key1 => world.select_element(Cell::Ground),
                    Key::Key2 => world.select_element(Cell::Water),
                    Key::E => world.rotate_canvas_anticlockwise(),
                    Key::R => world.rotate_canvas_clockwise(),
                    Key::N => world.generate_map(),
                    Key::C => world.clear_map(),
                    _ => (),
                }
            }
        }).unwrap();

        if window.get_mouse_down(MouseButton::Left) {
            window.get_mouse_pos(MouseMode::Discard).map(|(x, y)| {
                world.draw_element(x as usize, y as usize);
            });
        }

        // window.get_scroll_wheel().map(|scroll| {
        //     println!("-> {:?}", scroll);
        // });
        // tick_time = Instant::now();

        world.tick();
        world.render(&mut buff);

        window
            .update_with_buffer(&buff, WIDTH, HEIGHT)
            .unwrap();

        // println!("Tick Time: {}", tick_time.elapsed().as_micros());

        // FPS Calculation
        // fps_counter += 1;
        // fps_time_diff = fps_now.elapsed().as_secs();

        // if fps_time_diff > 1 {
            // println!("FPS: {}", fps_counter as f64 / fps_time_diff as f64);
            // fps_counter = 0;
            // fps_now = Instant::now();
        // }
    }
}

fn gpu_rendering() {
    if let Err(failure) = automata_sandbox::run_simulation() {
        eprintln!("Application failed: {}", failure);
    }
}

fn main() {
    // cpu_rendering();
    gpu_rendering();
}
