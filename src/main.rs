use std::fs::File;
use std::io::{BufReader, BufRead, Read};
use std::path::Path;

use minifb::{
    Key,
    WindowOptions,
    Window,
    MouseMode,
    MouseButton,
    KeyRepeat,
};
use vek::Vec2;

const WIDTH: usize = 100;
const HEIGHT: usize = 100;
const N_NEIGHBORS: u8 = 8;
const DELTA: f32 = 1.0;

const MIN_FLOW: f32 = 0.01;
const MAX_MASS: f32 = 1.0;
const MAX_COMPRESS: f32 = 0.02;
const MIN_MASS: f32 = 0.0001;
const MIN_DRAW: f32 = 0.01;
const MAX_DRAW: f32 = 1.1;
const MAX_SPEED: f32 = 1.0;

#[derive(Clone)]
enum Elements {
    Brick,
    Water,
    Empty,
}

#[derive(Clone)]
enum LifeMode {
    Cross,
    Circle,
    Fish,
    Flower,
    Galaxy,
}

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
        }
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
enum CellElement {
    Empty,
    Wall,
    Solid,

    Water,
    Ground,
    Air,
}

#[derive(Copy, Clone)]
struct Cell {
    velocity: Vec2<f32>,
    population: f32,
    spread: f32,
    conductivity: f32,

    element: CellElement,

    // mass: f32,
    // new_mass: f32,

    // living: bool,
    // solid: bool,
}

struct CellWater {}

struct CellAir {}

struct CellGround {}

impl Cell {
    fn empty() -> Self {
        Cell {
            velocity: Vec2::zero(),
            population: 0.0,
            spread: 0.0,
            conductivity: 1.0,

            element: CellElement::Empty,
        }
    }

    fn set_element(&mut self, element: CellElement) {
        match element {
            CellElement::Empty => {
                self.velocity = Vec2::zero();
                self.population = 0.0;
            }
            CellElement::Wall => {
                self.velocity = Vec2::zero();
                self.population = 0.0;
                self.conductivity = 0.0;
                self.population = 400.0;
            }
            CellElement::Water => {
                self.velocity = Vec2::new(0.0, -0.9);
                self.spread = 0.5;
                self.population = 250.0;
            }
            _ => {}
        }
    }
}

struct World {
    water: [i32; WIDTH],
    energy: [i32; WIDTH],
    ground: [i32; WIDTH],

    mass: [[f32; WIDTH]; HEIGHT],
    new_mass: [[f32; WIDTH]; HEIGHT],
    blocks: [[Cell; WIDTH]; HEIGHT],
}

impl World {
    fn new() -> Self {
        let mut this = Self {
            energy: [0; WIDTH],
            water: [0; WIDTH],
            ground: [10; WIDTH],

            mass: [[0.0; WIDTH]; HEIGHT],
            new_mass: [[0.0; WIDTH]; HEIGHT],
            blocks: [[Cell::empty(); WIDTH]; HEIGHT],
        };

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

    fn tick(&mut self) {
        let mut flow = 0.0;
        let mut remaining_mass = 0.0;
        let mut blocks = self.blocks.clone();
        let mut mass = self.mass.clone();
        let mut new_mass = self.new_mass.clone();

        for x in 1..WIDTH {
            for y in 1..HEIGHT {
                // Skip inert ground blocks
                if blocks[x][y].element == CellElement::Ground {
                    continue;
                }

                // Custom push-only flow
                flow = 0.0;
                remaining_mass = mass[x][y];

                if remaining_mass <= 0.0 {
                    continue;
                }

                // The block bellow this one
                if blocks[x][y + 1].element != CellElement::Ground {
                    flow = self.get_stable_state(remaining_mass + mass[x][y + 1]) - mass[x][y + 1];
                    if flow > MIN_FLOW {
                        flow *= 0.5; // leads to smoother flow
                    }
                    flow = if flow <= 0.0 {
                        0.0
                    } else if flow >= remaining_mass.min(MAX_SPEED) {
                        remaining_mass.min(MAX_SPEED)
                    } else {
                        flow
                    };

                    new_mass[x][y] -= flow;
                    new_mass[x][y + 1] += flow;
                    remaining_mass -= flow;
                }

                if remaining_mass <= 0.0 {
                    continue;
                }

                // Left
                if blocks[x - 1][y].element != CellElement::Ground {
                    flow = mass[x][y] - mass[x - 1][y] / 4.0;
                    if flow > MIN_FLOW {
                        flow *= 0.5;
                    }
                    flow = if flow <= 0.0 {
                        0.0
                    } else if flow >= remaining_mass {
                        remaining_mass
                    } else {
                        flow
                    };

                    new_mass[x][y] -= flow;
                    new_mass[x - 1][y] += flow;
                    remaining_mass -= flow;
                }

                if remaining_mass <= 0.0 {
                    continue;
                }

                // Right
                if blocks[x + 1][y].element != CellElement::Ground {
                    flow = mass[x][y] - mass[x + 1][y] / 4.0;
                    if flow > MIN_FLOW {
                        flow *= 0.5;
                    }

                    flow = if flow <= 0.0 {
                        0.0
                    } else if flow >= remaining_mass {
                        remaining_mass
                    } else {
                        flow
                    };

                    new_mass[x][y] -= flow;
                    new_mass[x + 1][y] += flow;
                    remaining_mass -= flow;
                }

                if remaining_mass <= 0.0 {
                    continue;
                }

                // Up. Only compressed water flows upwards
                if blocks[x][y - 1].element != CellElement::Ground {
                    flow = remaining_mass - self.get_stable_state(remaining_mass + mass[x][y - 1]);
                    if flow > MIN_FLOW {
                        flow *= 0.5;
                    }

                    flow = if flow <= 0.0 {
                        0.0
                    } else if flow >= remaining_mass.min(MAX_SPEED) {
                        remaining_mass.min(MAX_SPEED)
                    } else {
                        flow
                    };

                    new_mass[x][y] -= flow;
                    new_mass[x][y - 1] += flow;
                    remaining_mass -= flow;
                }
            }
        }

        for x in 1..WIDTH {
            for y in 1..HEIGHT {
                if blocks[x][y].element == CellElement::Ground {
                    continue;
                }

                if mass[x][y] > MIN_MASS {
                    blocks[x][y].element = CellElement::Water;
                } else {
                    blocks[x][y].element = CellElement::Air;
                }
            }
        }

        self.mass = new_mass;
        self.blocks = blocks;
    }

    fn spawn_water(&mut self, x: usize, y: usize) {
        self.mass[x][y] = 10.0;
        self.blocks[x][y].element = CellElement::Water;
    }

    fn spawn_ground(&mut self, x: usize, y: usize) {
        self.blocks[x][y].element = CellElement::Ground;
    }

    fn render_line(&self, buff: &mut [u32], x: usize, mut y0: usize, mut y1: usize, color: Color) {
        let j: usize = 0;
        y0 = HEIGHT - y0;
        y1 = HEIGHT - y1;

        while y0 > y1 {
            y0 -= 1;
            buff[y0 as usize * WIDTH + x as usize] = color.get_hex();
        }
    }

    fn troubleshoor_renderer(&self, buff: &mut [u32]) {
        self.render_line(buff, 10, 0, 10, Color::Red);
        self.render_line(buff, 10, 10, 50, Color::Blue);
    }

    fn get_water_color(&self, mass: f32) -> u32 {
        Color::Blue.get_hex()
    }

    fn render(&self, buff: &mut [u32]) {
        let blocks = self.blocks.clone();
        let mass = self.mass.clone();

        for y in 1..HEIGHT {
            for x in 1..WIDTH {
                let current_cell = self.blocks[x][y];
                buff[y * WIDTH + x] = match current_cell.element {
                    CellElement::Water => self.get_water_color(mass[x][y]),
                    CellElement::Air => Color::Black.get_hex(),
                    CellElement::Ground => Color::Yellow.get_hex(),
                    _ => 0,
                }
            }
        }
    }
}

fn main() {
    let mut world = World::new();

    let mut buff = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new(
        "CA",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: minifb::Scale::X4,
            ..WindowOptions::default()
        },
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600 * 2)));

    while window.is_open() && !window.is_key_down(Key::Q) {
        if window.get_mouse_down(MouseButton::Left) {
            window.get_mouse_pos(MouseMode::Discard).map(|(x, y)| {
                world.spawn_water(x as usize, y as usize);
            });
        }

        if window.get_mouse_down(MouseButton::Right) {
            window.get_mouse_pos(MouseMode::Discard).map(|(x, y)| {
                world.spawn_ground(x as usize, y as usize);
            });
        }

        world.tick();
        world.render(&mut buff);

        window
            .update_with_buffer(&buff, WIDTH, HEIGHT)
            .unwrap();
    }
}